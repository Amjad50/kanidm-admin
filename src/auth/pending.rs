//! In-memory store of mid-flight login conversations.
//!
//! Each browser starts at `POST /login` with a username; we create a fresh
//! `KanidmClient`, call `auth_step_init` on it, and stash the client in this
//! store under a v4 Uuid. The browser holds the Uuid in a short-lived
//! `kanidm_admin_login` cookie scoped to `Path=/login`. Each subsequent step
//! looks up the same `KanidmClient` so the in-flight `auth_session_id` that
//! kanidm_client tracks internally stays consistent across requests.
//!
//! Entries TTL after 5 minutes of idleness. Eviction is opportunistic —
//! every read scans for expired entries.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kanidm_client::KanidmClient;
use kanidm_proto::v1::{AuthAllowed, AuthMech};
use uuid::Uuid;

pub const PENDING_TTL: Duration = Duration::from_secs(300);

pub struct PendingAuth {
    /// KanidmClient is not Clone — wrap in Arc so handlers can hold an
    /// owned handle outside the store lock while making the actual
    /// `.await` call.
    pub client: Arc<KanidmClient>,
    pub ident: String,
    /// Filtered + ordered list of mechs the server told us are available
    /// for this user, captured at `auth_step_init` and rendered as the
    /// chooser. Excludes Anonymous and OAuth2Trust.
    pub available: Vec<AuthMech>,
    pub mech: Option<AuthMech>,
    pub continued: Vec<AuthAllowed>,
    /// Base64-encoded JSON of the `RequestChallengeResponse` returned by
    /// `auth_step_begin`. Populated by handlers after `fresh()` and consumed
    /// when the browser posts the signed credential back.
    pub challenge: Option<String>,
    pub return_to: String,
    pub last_touched: Instant,
}

impl PendingAuth {
    fn fresh(
        client: Arc<KanidmClient>,
        ident: String,
        available: Vec<AuthMech>,
        return_to: String,
    ) -> Self {
        Self {
            client,
            ident,
            available,
            mech: None,
            continued: Vec::new(),
            challenge: None,
            return_to,
            last_touched: Instant::now(),
        }
    }
}

pub struct PendingAuthStore {
    inner: Mutex<HashMap<Uuid, PendingAuth>>,
}

impl Default for PendingAuthStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PendingAuthStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Insert a new in-flight conversation, returning the key the browser
    /// should hold in its login cookie.
    pub fn insert(
        &self,
        client: Arc<KanidmClient>,
        ident: String,
        available: Vec<AuthMech>,
        return_to: String,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let mut guard = self.inner.lock().expect("pending store poisoned");
        sweep_expired(&mut guard);
        guard.insert(id, PendingAuth::fresh(client, ident, available, return_to));
        id
    }

    /// Mutate an entry in place, bumping its TTL. Returns whatever the
    /// closure returns. Returns None if the entry is missing or expired.
    pub fn with_mut<R>(&self, id: Uuid, f: impl FnOnce(&mut PendingAuth) -> R) -> Option<R> {
        let mut guard = self.inner.lock().expect("pending store poisoned");
        sweep_expired(&mut guard);
        let entry = guard.get_mut(&id)?;
        entry.last_touched = Instant::now();
        Some(f(entry))
    }

    /// Take ownership of the entry, removing it from the store. Used on
    /// terminal states (Success, Denied) or when the user backs out.
    pub fn take(&self, id: Uuid) -> Option<PendingAuth> {
        let mut guard = self.inner.lock().expect("pending store poisoned");
        sweep_expired(&mut guard);
        guard.remove(&id)
    }
}

fn sweep_expired(map: &mut HashMap<Uuid, PendingAuth>) {
    let now = Instant::now();
    map.retain(|_, v| now.duration_since(v.last_touched) < PENDING_TTL);
}
