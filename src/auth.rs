use std::path::PathBuf;

use anyhow::{anyhow, Result};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use kanidm_client::{KanidmClient, KanidmClientBuilder};
use kanidm_proto::v1::Entry;

use crate::config::Config;
use crate::error::AppError;
use crate::AppState;

/// Builds per-request KanidmClient instances pre-loaded with the caller's
/// session token. One factory holds the connection config; each request gets
/// its own client so per-user auth is isolated.
pub struct KanidmClientFactory {
    base_url: String,
    ca_path: Option<PathBuf>,
    accept_invalid_certs: bool,
}

impl KanidmClientFactory {
    pub fn new(cfg: &Config) -> Result<Self> {
        Ok(Self {
            base_url: cfg.kanidm_url.clone(),
            ca_path: cfg.kanidm_ca_path.as_deref().map(PathBuf::from),
            accept_invalid_certs: cfg.kanidm_accept_invalid_certs,
        })
    }

    fn builder(&self) -> Result<KanidmClientBuilder> {
        let mut builder = KanidmClientBuilder::new().address(self.base_url.clone());
        if let Some(ca) = &self.ca_path {
            // ClientError doesn't impl std::error::Error, so we can't use `?` with anyhow.
            builder = builder
                .add_root_certificate_filepath(ca.to_string_lossy().as_ref())
                .map_err(|e| anyhow!("loading CA cert {:?}: {e:?}", ca))?;
        }
        if self.accept_invalid_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }
        Ok(builder)
    }

    /// Build a client with the given bearer token already set.
    pub async fn for_token(&self, token: &str) -> Result<KanidmClient> {
        let builder = self.builder()?;
        let client = builder
            .build()
            .map_err(|e| anyhow!("building kanidm client: {e:?}"))?;
        client.set_token(token.to_string()).await;
        Ok(client)
    }
}

/// Authenticated admin user. Extracted from a request via [`FromRequestParts`].
/// The presence of this extractor in a handler signature guarantees:
///   - the request has a kanidm session cookie
///   - the token is valid (kanidm's /v1/auth/valid said yes)
///   - the user is a member of the configured admin group
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdminUser {
    pub token: String,
    pub spn: String,
    pub displayname: String,
    pub uuid: String,
}

impl AdminUser {
    pub fn spn(&self) -> &str {
        &self.spn
    }
    pub fn displayname(&self) -> &str {
        &self.displayname
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie_name = &state.config.kanidm_session_cookie;
        let token = jar
            .get(cookie_name)
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthenticated)?;

        let client = state
            .kanidm
            .for_token(&token)
            .await
            .map_err(|e| AppError::Kanidm(e.to_string()))?;

        // Validate the token; kanidm checks signature, expiry, revocation.
        client
            .auth_valid()
            .await
            .map_err(|e| AppError::Kanidm(format!("token validation failed: {e:?}")))?;

        // Pull the caller's full entry, including memberof, via whoami.
        let entry = client
            .whoami()
            .await
            .map_err(|e| AppError::Kanidm(format!("whoami failed: {e:?}")))?
            .ok_or_else(|| AppError::Kanidm("whoami returned no entry".to_string()))?;

        // Admin gate: must be in the configured admin group.
        let admin_group = &state.config.admin_group;
        if !entry_in_group(&entry, admin_group) {
            return Err(AppError::Forbidden);
        }

        let spn = attr_first(&entry, "spn").unwrap_or_default();
        let displayname = attr_first(&entry, "displayname").unwrap_or_else(|| spn.clone());
        let uuid = attr_first(&entry, "uuid").unwrap_or_default();

        Ok(AdminUser {
            token,
            spn,
            displayname,
            uuid,
        })
    }
}

/// Check if an Entry has membership in a group identified by either:
///   - its bare name (e.g. "idm_admins"), or
///   - its full SPN (e.g. "idm_admins@idm.home.amsh.dev").
/// We look at `memberof` (transitive) so nested membership counts.
fn entry_in_group(entry: &Entry, group: &str) -> bool {
    entry
        .attrs
        .get("memberof")
        .into_iter()
        .flatten()
        .chain(entry.attrs.get("directmemberof").into_iter().flatten())
        .any(|m| {
            m == group || m.split('@').next().map(|n| n == group).unwrap_or(false)
        })
}

fn attr_first(entry: &Entry, name: &str) -> Option<String> {
    entry.attrs.get(name).and_then(|v| v.first().cloned())
}
