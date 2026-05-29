//! Login flow handlers. Walks kanidm's auth state machine via `kanidm_client`,
//! stashing the in-flight `KanidmClient` in `PendingAuthStore` between
//! requests. On success, sets `session_cookie_name` and redirects to the
//! requested `return_to`.

use std::str::FromStr;

use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Form, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use kanidm_proto::v1::{AuthAllowed, AuthMech, AuthState};
use kanidm_proto::internal::UserAuthToken;
use kanidm_proto::webauthn::PublicKeyCredential;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::AppState;

const LOGIN_COOKIE: &str = "kanidm_admin_login";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(get_username).post(post_username))
        .route("/login/mech", get(get_mech).post(post_mech))
        .route("/login/password", get(get_password).post(post_password))
        .route("/login/totp", get(get_totp).post(post_totp))
        .route("/login/backup-code", get(get_backup).post(post_backup))
        .route("/login/passkey", get(get_passkey).post(post_passkey))
        .route("/login/security-key", get(get_security_key).post(post_security_key))
        .route("/login/denied", get(get_denied))
}

// ─── Templates ────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "login/01_username.html")]
struct UsernameView {
    domain_name: Option<String>,
    return_to: String,
    expired: bool,
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/02_mech.html")]
struct MechView {
    domain_name: Option<String>,
    ident: String,
    displayname: Option<String>,
    initials: String,
    choices: Vec<MechChoice>,
    error: Option<String>,
}

struct MechChoice {
    value: &'static str,
    label: &'static str,
    desc: &'static str,
    icon: &'static str,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/03_password.html")]
struct PasswordView {
    domain_name: Option<String>,
    ident: String,
    displayname: Option<String>,
    initials: String,
    show_back: bool,
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/04_totp.html")]
struct TotpView {
    domain_name: Option<String>,
    ident: String,
    displayname: Option<String>,
    initials: String,
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/05_backup_code.html")]
struct BackupCodeView {
    domain_name: Option<String>,
    ident: String,
    displayname: Option<String>,
    initials: String,
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/06_passkey.html")]
struct PasskeyView {
    domain_name: Option<String>,
    ident: String,
    displayname: Option<String>,
    initials: String,
    challenge: String,
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/07_security_key.html")]
struct SecurityKeyView {
    domain_name: Option<String>,
    ident: String,
    displayname: Option<String>,
    initials: String,
    challenge: String,
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "login/07_denied.html")]
struct DeniedView {
    domain_name: Option<String>,
    message: String,
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

fn initials_of(name: &str) -> String {
    crate::views::initials_for_login(name)
}

/// Read the login-cookie Uuid out of the jar, if present and parseable.
fn pending_id(jar: &CookieJar) -> Option<Uuid> {
    jar.get(LOGIN_COOKIE)
        .and_then(|c| Uuid::from_str(c.value()).ok())
}

fn build_login_cookie(value: String, state: &AppState) -> Cookie<'static> {
    Cookie::build((LOGIN_COOKIE, value))
        .path("/login")
        .http_only(true)
        .secure(!state.config.dev_insecure_cookies)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::minutes(5))
        .build()
}

fn clear_login_cookie(state: &AppState) -> Cookie<'static> {
    Cookie::build((LOGIN_COOKIE, ""))
        .path("/login")
        .http_only(true)
        .secure(!state.config.dev_insecure_cookies)
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build()
}

fn build_session_cookie(token: String, state: &AppState) -> Cookie<'static> {
    let expiry = decode_uat_expiry(&token);
    let mut b = Cookie::build((state.config.session_cookie_name.clone(), token))
        .path("/")
        .http_only(true)
        .secure(!state.config.dev_insecure_cookies)
        .same_site(SameSite::Lax);
    if let Some(exp) = expiry {
        b = b.expires(exp);
    }
    b.build()
}

fn decode_uat_expiry(jws: &str) -> Option<OffsetDateTime> {
    let mut parts = jws.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _sig = parts.next()?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let uat: UserAuthToken = serde_json::from_slice(&bytes).ok()?;
    uat.expiry
}

/// `return_to` arrives via query string on /login and via hidden form field
/// on POST. Reject anything that isn't a same-origin path: must start with
/// '/', not '//', under 512 bytes, no control characters.
fn sanitize_return_to(raw: Option<&str>) -> String {
    let s = raw.unwrap_or("/");
    if s.len() > 512
        || !s.starts_with('/')
        || s.starts_with("//")
        || s.chars().any(|c| c.is_control())
    {
        return "/".to_string();
    }
    s.to_string()
}

fn redirect_with_cookies(target: &str, jar: CookieJar) -> Response {
    (jar, Redirect::to(target)).into_response()
}

fn discard_pending_and_redirect(state: &AppState, jar: CookieJar, target: &str) -> Response {
    if let Some(id) = pending_id(&jar) {
        let _ = state.pending.take(id);
    }
    let jar = jar.add(clear_login_cookie(state));
    redirect_with_cookies(target, jar)
}

/// Filter, dedupe, and order mechs for the chooser. Anonymous + OAuth2Trust
/// removed; strongest-first display.
fn presentable_mechs(mut mechs: Vec<AuthMech>) -> Vec<AuthMech> {
    mechs.retain(|m| !matches!(m, AuthMech::Anonymous | AuthMech::OAuth2Trust));
    mechs.sort_by_key(|m| match m {
        AuthMech::Passkey => 0,
        AuthMech::PasswordSecurityKey => 1,
        AuthMech::PasswordTotp => 2,
        AuthMech::PasswordBackupCode => 3,
        AuthMech::Password => 4,
        _ => 99,
    });
    mechs
}

fn mech_to_choice(mech: &AuthMech) -> Option<MechChoice> {
    match mech {
        AuthMech::Passkey => Some(MechChoice {
            value: "passkey",
            label: "Passkey",
            desc: "Use a registered authenticator (Touch ID, YubiKey, etc.)",
            icon: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m21 2-9.6 9.6"/><circle cx="7.5" cy="15.5" r="5.5"/><path d="m15.5 7.5 3 3L22 7l-3-3"/></svg>"#,
        }),
        AuthMech::PasswordSecurityKey => Some(MechChoice {
            value: "passwordsecuritykey",
            label: "Password and security key",
            desc: "Your password plus a hardware security key",
            icon: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="10" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>"#,
        }),
        AuthMech::PasswordTotp => Some(MechChoice {
            value: "passwordmfa",
            label: "Password and TOTP",
            desc: "Your password plus a one-time code from your authenticator app",
            icon: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="16" r="1"/><rect x="3" y="10" width="18" height="12" rx="2"/><path d="M7 10V7a5 5 0 0 1 10 0v3"/></svg>"#,
        }),
        AuthMech::PasswordBackupCode => Some(MechChoice {
            value: "passwordbackupcode",
            label: "Password and backup code",
            desc: "Your password plus a recovery backup code",
            icon: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m4.93 4.93 4.24 4.24"/><path d="m14.83 9.17 4.24-4.24"/><path d="m14.83 14.83 4.24 4.24"/><path d="m9.17 14.83-4.24 4.24"/><circle cx="12" cy="12" r="4"/></svg>"#,
        }),
        AuthMech::Password => Some(MechChoice {
            value: "password",
            label: "Password",
            desc: "Just your password",
            icon: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="10" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>"#,
        }),
        _ => None,
    }
}

fn mech_from_form(v: &str) -> Option<AuthMech> {
    match v {
        "passkey" => Some(AuthMech::Passkey),
        "passwordsecuritykey" => Some(AuthMech::PasswordSecurityKey),
        "passwordmfa" => Some(AuthMech::PasswordTotp),
        "passwordbackupcode" => Some(AuthMech::PasswordBackupCode),
        "password" => Some(AuthMech::Password),
        _ => None,
    }
}

/// Where to send the user after `auth_step_begin(mech)`. kanidm asks for the
/// non-password factor FIRST on every multi-factor mech (see
/// server/lib/src/idm/authsession/mod.rs CredHandler::PasswordMfa flow),
/// then routes to the password page after the factor is verified.
fn route_for_mech(mech: &AuthMech) -> &'static str {
    match mech {
        AuthMech::Passkey => "/login/passkey",
        AuthMech::Password => "/login/password",
        AuthMech::PasswordTotp => "/login/totp",
        AuthMech::PasswordBackupCode => "/login/backup-code",
        AuthMech::PasswordSecurityKey => "/login/security-key",
        _ => "/login",
    }
}

/// Map an `AuthState::Continue(allowed)` response to the next page. After
/// the non-password factor verifies, kanidm always returns
/// `Continue([Password])` — see authsession/mod.rs lines 518, 619, 714.
fn route_for_next_allowed(allowed: &[AuthAllowed]) -> Result<&'static str, String> {
    if allowed.iter().any(|a| matches!(a, AuthAllowed::Password)) {
        Ok("/login/password")
    } else if allowed.iter().any(|a| matches!(a, AuthAllowed::Totp)) {
        Ok("/login/totp")
    } else if allowed.iter().any(|a| matches!(a, AuthAllowed::BackupCode)) {
        Ok("/login/backup-code")
    } else if allowed.iter().any(|a| matches!(a, AuthAllowed::Passkey(_))) {
        Err("Passkey login isn't wired up yet. Use a different sign-in method.".to_string())
    } else if allowed.iter().any(|a| matches!(a, AuthAllowed::SecurityKey(_))) {
        Err("Security-key login isn't wired up yet. Use a different sign-in method.".to_string())
    } else {
        Err("This account requires a credential we don't support yet.".to_string())
    }
}

fn extract_domain(state: &AppState) -> Option<String> {
    url::Url::parse(&state.config.kanidm_url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}

// ─── Step 1: username (screen 01) ─────────────────────────────────────────────

#[derive(Deserialize)]
struct UsernameQuery {
    return_to: Option<String>,
    expired: Option<String>,
    err: Option<String>,
}

async fn get_username(
    State(state): State<AppState>,
    Query(q): Query<UsernameQuery>,
    jar: CookieJar,
) -> Response {
    // Stale pending? clear it so a re-entry starts fresh.
    let jar = if let Some(id) = pending_id(&jar) {
        let _ = state.pending.take(id);
        jar.add(clear_login_cookie(&state))
    } else {
        jar
    };
    let view = UsernameView {
        domain_name: extract_domain(&state),
        return_to: sanitize_return_to(q.return_to.as_deref()),
        expired: q.expired.is_some(),
        error: q.err,
    };
    (jar, view).into_response()
}

#[derive(Deserialize)]
struct UsernameForm {
    ident: String,
    return_to: Option<String>,
}

async fn post_username(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<UsernameForm>,
) -> Response {
    let ident = form.ident.trim().to_string();
    if ident.is_empty() {
        return render_username_error(&state, jar, "Enter your username.");
    }
    let return_to = sanitize_return_to(form.return_to.as_deref());

    let client = match state.kanidm.anonymous() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = ?e, "could not build anonymous kanidm client");
            return render_username_error(&state, jar, "Unable to reach the identity server.");
        }
    };

    let mechs = match client.auth_step_init(&ident).await {
        Ok(m) => m,
        Err(e) => {
            tracing::debug!(ident = %ident, error = ?e, "auth_step_init failed");
            return render_username_error(&state, jar, "Unable to start authentication. Check the username and try again.");
        }
    };

    let presentable = presentable_mechs(mechs.into_iter().collect());
    if presentable.is_empty() {
        return render_username_error(&state, jar, "No supported sign-in mechanisms are available for this account.");
    }

    // Clear any prior pending entry.
    let jar = if let Some(id) = pending_id(&jar) {
        let _ = state.pending.take(id);
        jar
    } else {
        jar
    };

    let id = state
        .pending
        .insert(client, ident.clone(), presentable.clone(), return_to);
    let jar = jar.add(build_login_cookie(id.to_string(), &state));

    // Auto-skip the chooser when only one mech is available.
    if presentable.len() == 1 {
        let only = presentable[0].clone();
        state.pending.with_mut(id, |p| p.mech = Some(only.clone()));
        return match call_begin(&state, id, only).await {
            Ok(target) => redirect_with_cookies(target, jar),
            Err(msg) => discard_pending_and_redirect(
                &state,
                jar,
                &format!("/login/denied?msg={}", urlencode(&msg)),
            ),
        };
    }

    redirect_with_cookies("/login/mech", jar)
}

fn render_username_error(state: &AppState, jar: CookieJar, msg: &str) -> Response {
    let view = UsernameView {
        domain_name: extract_domain(state),
        return_to: "/".to_string(),
        expired: false,
        error: Some(msg.to_string()),
    };
    (StatusCode::OK, jar, view).into_response()
}

// ─── Step 2: mech chooser (screen 02) ─────────────────────────────────────────

async fn get_mech(State(state): State<AppState>, jar: CookieJar) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(view) = state.pending.with_mut(id, |p| MechView {
        domain_name: extract_domain(&state),
        ident: p.ident.clone(),
        displayname: None,
        initials: initials_of(&p.ident),
        choices: p.available.iter().filter_map(mech_to_choice).collect(),
        error: None,
    }) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    view.into_response()
}

#[derive(Deserialize)]
struct MechForm {
    mech: String,
}

async fn post_mech(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<MechForm>,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(mech) = mech_from_form(&form.mech) else {
        return Redirect::to("/login/mech").into_response();
    };

    state.pending.with_mut(id, |p| p.mech = Some(mech.clone()));

    match call_begin(&state, id, mech.clone()).await {
        Ok(target) => Redirect::to(target).into_response(),
        Err(msg) => discard_pending_and_redirect(
            &state,
            jar,
            &format!("/login/denied?msg={}", urlencode(&msg)),
        ),
    }
}

/// Call `auth_step_begin` on the pending entry's client; on success, route
/// the user to the matching step page.
async fn call_begin(state: &AppState, id: Uuid, mech: AuthMech) -> Result<&'static str, String> {
    let Some(client) = state.pending.with_mut(id, |p| std::sync::Arc::clone(&p.client)) else {
        return Err("Session expired. Sign in again.".to_string());
    };

    match client.auth_step_begin(mech.clone()).await {
        Ok(allowed) => {
            // If the first allowed factor is a WebAuthn challenge, encode it
            // using STANDARD base64 (not URL_SAFE) to match kanidm's own
            // server/core/src/https/views/login.rs:954-961.
            let challenge = allowed.iter().find_map(|a| match a {
                AuthAllowed::Passkey(chal) | AuthAllowed::SecurityKey(chal) => {
                    serde_json::to_vec(chal)
                        .ok()
                        .map(|data| STANDARD.encode(data))
                }
                _ => None,
            });
            state.pending.with_mut(id, |p| {
                p.continued = allowed;
                p.challenge = challenge;
            });
            Ok(route_for_mech(&mech))
        }
        Err(e) => {
            tracing::debug!(error = ?e, ?mech, "auth_step_begin failed");
            Err("This sign-in method isn't available for your account.".to_string())
        }
    }
}

// ─── Step 3: password (screen 03) ─────────────────────────────────────────────

#[derive(Deserialize)]
struct StepQuery {
    err: Option<String>,
}

async fn get_password(
    State(state): State<AppState>,
    Query(q): Query<StepQuery>,
    jar: CookieJar,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(view) = state.pending.with_mut(id, |p| PasswordView {
        domain_name: extract_domain(&state),
        ident: p.ident.clone(),
        displayname: None,
        initials: initials_of(&p.ident),
        show_back: p.mech.is_some(), // chooser was visited
        error: q.err.clone(),
    }) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    view.into_response()
}

#[derive(Deserialize)]
struct PasswordForm {
    password: String,
}

async fn post_password(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<PasswordForm>,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(client) = state.pending.with_mut(id, |p| std::sync::Arc::clone(&p.client)) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };

    match client.auth_step_password(&form.password).await {
        Ok(resp) => handle_terminal_or_continue(state, jar, id, resp.state, "/login/password"),
        Err(e) => {
            tracing::debug!(error = ?e, "auth_step_password call failed");
            Redirect::to("/login/password?err=Wrong+password.").into_response()
        }
    }
}

// ─── Step 4: TOTP (screen 04) ─────────────────────────────────────────────────

async fn get_totp(
    State(state): State<AppState>,
    Query(q): Query<StepQuery>,
    jar: CookieJar,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(view) = state.pending.with_mut(id, |p| TotpView {
        domain_name: extract_domain(&state),
        ident: p.ident.clone(),
        displayname: None,
        initials: initials_of(&p.ident),
        error: q.err.clone(),
    }) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    view.into_response()
}

#[derive(Deserialize)]
struct TotpForm {
    totp: String,
}

async fn post_totp(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<TotpForm>,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let totp_value: u32 = match form.totp.trim().parse() {
        Ok(v) => v,
        Err(_) => return Redirect::to("/login/totp?err=Enter+a+6-digit+code.").into_response(),
    };
    let Some(client) = state.pending.with_mut(id, |p| std::sync::Arc::clone(&p.client)) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    match client.auth_step_totp(totp_value).await {
        Ok(resp) => handle_terminal_or_continue(state, jar, id, resp.state, "/login/totp"),
        Err(e) => {
            tracing::debug!(error = ?e, "auth_step_totp call failed");
            Redirect::to("/login/totp?err=Incorrect+code.").into_response()
        }
    }
}

// ─── Step 5: backup code (screen 05) ──────────────────────────────────────────

async fn get_backup(
    State(state): State<AppState>,
    Query(q): Query<StepQuery>,
    jar: CookieJar,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(view) = state.pending.with_mut(id, |p| BackupCodeView {
        domain_name: extract_domain(&state),
        ident: p.ident.clone(),
        displayname: None,
        initials: initials_of(&p.ident),
        error: q.err.clone(),
    }) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    view.into_response()
}

#[derive(Deserialize)]
struct BackupForm {
    code: String,
}

async fn post_backup(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<BackupForm>,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(client) = state.pending.with_mut(id, |p| std::sync::Arc::clone(&p.client)) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    match client.auth_step_backup_code(form.code.trim()).await {
        Ok(resp) => handle_terminal_or_continue(state, jar, id, resp.state, "/login/backup-code"),
        Err(e) => {
            tracing::debug!(error = ?e, "auth_step_backup_code call failed");
            Redirect::to("/login/backup-code?err=Backup+code+rejected.").into_response()
        }
    }
}

// ─── Step 6: passkey (screen 06) ─────────────────────────────────────────────

async fn get_passkey(
    State(state): State<AppState>,
    Query(q): Query<StepQuery>,
    jar: CookieJar,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(view) = state.pending.with_mut(id, |p| {
        p.challenge.as_deref().map(|ch| PasskeyView {
            domain_name: extract_domain(&state),
            ident: p.ident.clone(),
            displayname: None,
            initials: initials_of(&p.ident),
            challenge: ch.to_string(),
            error: q.err.clone(),
        })
    }) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    let Some(view) = view else {
        return Redirect::to("/login").into_response();
    };
    view.into_response()
}

#[derive(Deserialize)]
struct CredForm {
    cred: String,
}

async fn post_passkey(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<CredForm>,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let pkc: Box<PublicKeyCredential> = match serde_json::from_str(&form.cred) {
        Ok(v) => v,
        Err(_) => {
            return Redirect::to(&format!(
                "/login/passkey?err={}",
                urlencode("Your device returned an unexpected response.")
            ))
            .into_response();
        }
    };
    let Some(client) = state.pending.with_mut(id, |p| std::sync::Arc::clone(&p.client)) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    match client.auth_step_passkey_complete(pkc).await {
        Ok(resp) => handle_terminal_or_continue(state, jar, id, resp.state, "/login/passkey"),
        Err(e) => {
            tracing::debug!(error = ?e, "auth_step_passkey_complete failed");
            Redirect::to(&format!(
                "/login/passkey?err={}",
                urlencode("Passkey verification failed. Try again.")
            ))
            .into_response()
        }
    }
}

// ─── Step 6b: security key (screen 07_security_key) ──────────────────────────

async fn get_security_key(
    State(state): State<AppState>,
    Query(q): Query<StepQuery>,
    jar: CookieJar,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let Some(view) = state.pending.with_mut(id, |p| {
        p.challenge.as_deref().map(|ch| SecurityKeyView {
            domain_name: extract_domain(&state),
            ident: p.ident.clone(),
            displayname: None,
            initials: initials_of(&p.ident),
            challenge: ch.to_string(),
            error: q.err.clone(),
        })
    }) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    let Some(view) = view else {
        return Redirect::to("/login").into_response();
    };
    view.into_response()
}

async fn post_security_key(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<CredForm>,
) -> Response {
    let Some(id) = pending_id(&jar) else {
        return Redirect::to("/login").into_response();
    };
    let pkc: Box<PublicKeyCredential> = match serde_json::from_str(&form.cred) {
        Ok(v) => v,
        Err(_) => {
            return Redirect::to(&format!(
                "/login/security-key?err={}",
                urlencode("Your device returned an unexpected response.")
            ))
            .into_response();
        }
    };
    let Some(client) = state.pending.with_mut(id, |p| std::sync::Arc::clone(&p.client)) else {
        let jar = jar.add(clear_login_cookie(&state));
        return redirect_with_cookies("/login?expired=1", jar);
    };
    match client.auth_step_securitykey_complete(pkc).await {
        Ok(resp) => {
            handle_terminal_or_continue(state, jar, id, resp.state, "/login/security-key")
        }
        Err(e) => {
            tracing::debug!(error = ?e, "auth_step_securitykey_complete failed");
            Redirect::to(&format!(
                "/login/security-key?err={}",
                urlencode("Security key verification failed. Try again.")
            ))
            .into_response()
        }
    }
}

// ─── Step 7: denied (screen 07) ──────────────────────────────────────────────

#[derive(Deserialize)]
struct DeniedQuery {
    msg: Option<String>,
}

async fn get_denied(
    State(state): State<AppState>,
    Query(q): Query<DeniedQuery>,
    jar: CookieJar,
) -> Response {
    let jar = if let Some(id) = pending_id(&jar) {
        let _ = state.pending.take(id);
        jar.add(clear_login_cookie(&state))
    } else {
        jar
    };
    let view = DeniedView {
        domain_name: extract_domain(&state),
        message: q
            .msg
            .unwrap_or_else(|| "Sign-in was denied. Try again or contact your administrator.".to_string()),
    };
    (jar, view).into_response()
}

// ─── Terminal handling — Success / Denied / Continue dispatch ─────────────────

/// After a cred step, branch on the returned `AuthState`:
///   - Success → plant session cookie, clear pending, redirect to return_to.
///   - Denied  → discard, redirect to /login/denied with the server's reason.
///   - Continue(allowed) → route to /login/totp or /login/backup-code, or
///     deny if the next allowed is something we don't implement.
///   - Choose  → unexpected here; treat as protocol error.
fn handle_terminal_or_continue(
    state: AppState,
    jar: CookieJar,
    id: Uuid,
    auth_state: AuthState,
    current_path: &str,
) -> Response {
    match auth_state {
        AuthState::Success(token) => {
            let pending = state.pending.take(id);
            let return_to = pending
                .as_ref()
                .map(|p| p.return_to.clone())
                .unwrap_or_else(|| "/".to_string());
            let jar = jar
                .remove(Cookie::from(LOGIN_COOKIE))
                .add(clear_login_cookie(&state))
                .add(build_session_cookie(token, &state));
            redirect_with_cookies(&return_to, jar)
        }
        AuthState::Denied(reason) => discard_pending_and_redirect(
            &state,
            jar,
            &format!("/login/denied?msg={}", urlencode(&reason)),
        ),
        AuthState::Continue(allowed) => match route_for_next_allowed(&allowed) {
            Ok(target) => {
                state.pending.with_mut(id, |p| p.continued = allowed);
                Redirect::to(target).into_response()
            }
            Err(msg) => discard_pending_and_redirect(
                &state,
                jar,
                &format!("/login/denied?msg={}", urlencode(&msg)),
            ),
        },
        AuthState::Choose(_) => {
            tracing::warn!(path = %current_path, "unexpected AuthState::Choose after cred step");
            Redirect::to("/login/denied?msg=Unexpected+protocol+state.").into_response()
        }
    }
}

fn urlencode(s: &str) -> String {
    // Tiny ad-hoc form-encoder for the denied-message query parameter.
    // Only encodes the few characters that would break a URL — good enough
    // since the message is rendered server-side after re-decoding.
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ' ' => out.push('+'),
            c if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') => out.push(c),
            c => {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).bytes() {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}
