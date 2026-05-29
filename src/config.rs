use anyhow::Result;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Address to bind the HTTP server to. Default 127.0.0.1:3000.
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    /// URL of the kanidm server (e.g. https://idm.home.amsh.dev).
    pub kanidm_url: String,

    /// Path to CA cert for kanidm, if it uses a self-signed cert. Optional.
    #[serde(default)]
    pub kanidm_ca_path: Option<String>,

    /// Skip TLS verification entirely. Dev only — never in production.
    #[serde(default)]
    pub kanidm_accept_invalid_certs: bool,

    /// Name of the cookie we set on successful login. Distinct from kanidm's
    /// own `bearer` cookie so the two never collide on a shared parent domain.
    #[serde(default = "default_session_cookie")]
    pub session_cookie_name: String,

    /// Group SPN that grants access to the admin panel. Users not in this group
    /// see "Forbidden".
    #[serde(default = "default_admin_group")]
    pub admin_group: String,

    /// Where to find static assets (CSS, JS bundles).
    #[serde(default = "default_static_dir")]
    pub static_dir: String,

    /// Drop the `Secure` cookie flag so login works on plain http://localhost
    /// in dev. NEVER true in production.
    #[serde(default)]
    pub dev_insecure_cookies: bool,
}

fn default_bind_addr() -> String {
    "127.0.0.1:3000".to_string()
}
fn default_session_cookie() -> String {
    "kanidm_admin_session".to_string()
}
fn default_admin_group() -> String {
    "idm_admins".to_string()
}
fn default_static_dir() -> String {
    "static".to_string()
}

impl Config {
    pub fn load() -> Result<Self> {
        let cfg: Self = Figment::new()
            .merge(Toml::file("kanidm-admin-ui.toml"))
            .merge(Env::prefixed("KANIDM_ADMIN_").global())
            .extract()?;
        Ok(cfg)
    }
}
