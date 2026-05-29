use std::path::PathBuf;

use anyhow::{anyhow, Result};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use axum_htmx::HxRequest;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use kanidm_client::{KanidmClient, KanidmClientBuilder};
use kanidm_proto::internal::{UatPurpose, UserAuthToken};
use kanidm_proto::v1::Entry;
use time::OffsetDateTime;

use crate::config::Config;
use crate::error::AppError;
use crate::kanidm::entry::attr_first;
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
    /// UAT session_id — the UUID kanidm assigned to this auth token.
    /// Decoded from the JWS payload; used to destroy the current session on
    /// logout and to flag "this is you" rows in the sessions list.
    pub session_id: Option<String>,
    pub signed_in_at: Option<OffsetDateTime>,
    pub session_expires_at: Option<OffsetDateTime>,
    /// True when the session has active ReadWrite privileges (not expired).
    pub privileged: bool,
    /// When the ReadWrite privilege window expires (None if read-only or unknown).
    pub privileged_until: Option<OffsetDateTime>,
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let HxRequest(is_htmx) = HxRequest::from_request_parts(parts, state)
            .await
            .unwrap_or(HxRequest(false));
        let cookie_name = &state.config.kanidm_session_cookie;
        let token = jar
            .get(cookie_name)
            .map(|c| c.value().to_string())
            .ok_or_else(|| AppError::Unauthenticated {
                kanidm_url: state.config.kanidm_url.clone(),
                is_htmx,
            })?;

        let client = state
            .kanidm
            .for_token(&token)
            .await
            .map_err(|e| AppError::Kanidm(e.to_string()))?;

        // auth_valid returning Err means the session is no longer accepted by
        // kanidm — surface it as Unauthenticated so HTMX requests get the
        // reauth modal trigger instead of a generic 502.
        if let Err(e) = client.auth_valid().await {
            tracing::debug!(error = ?e, "auth_valid rejected; treating as unauthenticated");
            return Err(AppError::Unauthenticated {
                kanidm_url: state.config.kanidm_url.clone(),
                is_htmx,
            });
        }

        // Pull the caller's full entry, including memberof, via whoami.
        let entry = client
            .whoami()
            .await
            .map_err(|e| AppError::Kanidm(format!("whoami failed: {e:?}")))?
            .ok_or_else(|| AppError::Kanidm("whoami returned no entry".to_string()))?;

        // Admin gate: must be in the configured admin group.
        let admin_group = &state.config.admin_group;
        if !entry_in_group(&entry, admin_group) {
            return Err(AppError::Forbidden {
                admin_group: admin_group.clone(),
            });
        }

        let spn = attr_first(&entry, "spn").unwrap_or_default();
        let displayname = attr_first(&entry, "displayname").unwrap_or_else(|| spn.clone());
        let uuid = attr_first(&entry, "uuid").unwrap_or_default();

        let (session_id, signed_in_at, session_expires_at, privileged, privileged_until) =
            match parse_uat_payload(&token) {
                Some(uat) => {
                    let now = OffsetDateTime::now_utc();
                    let (privileged, privileged_until) = match &uat.purpose {
                        UatPurpose::ReadWrite { expiry: Some(exp) } if now < *exp => {
                            (true, Some(*exp))
                        }
                        _ => (false, None),
                    };
                    (
                        Some(uat.session_id.to_string()),
                        Some(uat.issued_at),
                        uat.expiry,
                        privileged,
                        privileged_until,
                    )
                }
                None => {
                    tracing::warn!(spn = ?spn, "could not decode UAT payload from session cookie");
                    (None, None, None, false, None)
                }
            };

        Ok(AdminUser {
            token,
            spn,
            displayname,
            uuid,
            session_id,
            signed_in_at,
            session_expires_at,
            privileged,
            privileged_until,
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

/// Decode the payload segment of a JWS compact-serialised token without
/// verifying the signature. The token has already been validated by
/// `auth_valid()`, so signature re-verification here would be redundant.
///
/// JWS compact serialisation format: `header.payload.signature` where each
/// segment is base64url-no-pad encoded.
fn parse_uat_payload(jws: &str) -> Option<UserAuthToken> {
    let mut parts = jws.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _signature = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&bytes).ok()
}
