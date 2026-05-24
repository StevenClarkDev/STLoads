use axum::http::{HeaderMap, header};
use chrono::Utc;
use db::auth::{
    UserRecord, delete_personal_access_token_by_hash, delete_personal_access_tokens_for_user,
    find_personal_access_token_by_hash, find_user_by_id, insert_personal_access_token,
    list_permission_names_for_role, touch_personal_access_token,
};
use db::organizations::{list_permission_keys_for_organization_role, primary_membership_for_user};
use domain::auth::{
    AccountStatus, Permission, ROLE_PERMISSION_CONTRACTS, UserRole, role_descriptors,
};
use sha2::{Digest, Sha256};
use shared::{AuthSessionState, AuthSessionUser};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct ResolvedSession {
    pub user: UserRecord,
    pub session: AuthSessionState,
}

pub async fn resolve_session_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<ResolvedSession>, String> {
    match bearer_token_from_headers(headers) {
        Some(token) => resolve_session_from_token(state, &token).await,
        None => Ok(None),
    }
}

pub async fn resolve_session_from_token(
    state: &AppState,
    token: &str,
) -> Result<Option<ResolvedSession>, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(None);
    };

    let Some(parsed_token) = parse_session_token(token) else {
        return Ok(None);
    };

    let token_hash = hash_session_token(parsed_token.secret);
    let Some(token_record) =
        find_personal_access_token_by_hash(pool, parsed_token.prefix, &token_hash)
            .await
            .map_err(|error| format!("token lookup failed: {}", error))?
    else {
        return Ok(None);
    };

    if token_record
        .expires_at
        .map(|expires_at| expires_at <= Utc::now().naive_utc())
        .unwrap_or(false)
    {
        let _ = delete_personal_access_token_by_hash(pool, parsed_token.prefix, &token_hash).await;
        return Ok(None);
    }

    let Some(user) = find_user_by_id(pool, token_record.tokenable_id)
        .await
        .map_err(|error| format!("user lookup failed: {}", error))?
    else {
        return Ok(None);
    };

    let _ = touch_personal_access_token(pool, token_record.id).await;
    let mut session = build_session_state(state, &user).await;
    if token_abilities_include(token_record.abilities.as_deref(), "mfa_verified")
        && !session
            .permissions
            .iter()
            .any(|permission| permission == "mfa_verified")
    {
        session.permissions.push("mfa_verified".into());
        session
            .notes
            .push("This session completed privileged MFA.".into());
    }

    Ok(Some(ResolvedSession { user, session }))
}

pub async fn issue_session_token(state: &AppState, user: &UserRecord) -> Result<String, String> {
    issue_session_token_with_mfa(state, user, false).await
}

pub async fn issue_session_token_with_mfa(
    state: &AppState,
    user: &UserRecord,
    mfa_verified: bool,
) -> Result<String, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Err("database is unavailable for login".into());
    };

    let token_prefix = session_token_part();
    let token_secret = format!("{}{}", session_token_part(), session_token_part());
    let token_hash = hash_session_token(&token_secret);
    let bearer_token = format!("stl_{}.{}", token_prefix, token_secret);
    let mut permissions = permission_keys_for_user(state, user).await;
    if mfa_verified
        && !permissions
            .iter()
            .any(|permission| permission == "mfa_verified")
    {
        permissions.push("mfa_verified".into());
    }
    let abilities = serde_json::to_string(&permissions).ok();
    let expires_at = Some(Utc::now().naive_utc() + chrono::Duration::days(14));

    delete_personal_access_tokens_for_user(pool, user.id)
        .await
        .map_err(|error| format!("token rotation failed: {}", error))?;

    insert_personal_access_token(
        pool,
        user.id,
        "rust-session",
        &token_prefix,
        &token_hash,
        abilities.as_deref(),
        expires_at,
    )
    .await
    .map_err(|error| format!("token insert failed: {}", error))?;

    Ok(bearer_token)
}

pub async fn revoke_session_token(state: &AppState, token: &str) -> Result<u64, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(0);
    };

    let Some(parsed_token) = parse_session_token(token) else {
        return Ok(0);
    };
    let token_hash = hash_session_token(parsed_token.secret);

    delete_personal_access_token_by_hash(pool, parsed_token.prefix, &token_hash)
        .await
        .map_err(|error| format!("token delete failed: {}", error))
}

pub async fn build_session_state(state: &AppState, user: &UserRecord) -> AuthSessionState {
    let permissions = permission_keys_for_user(state, user).await;
    let primary_membership = if let Some(pool) = state.pool.as_ref() {
        primary_membership_for_user(pool, user.id)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    let notes = match user.account_status() {
        Some(AccountStatus::Approved) => {
            vec!["Authenticated through the Rust session layer.".into()]
        }
        Some(status) => vec![format!(
            "Authenticated, but account status is {}.",
            account_status_label(Some(status))
        )],
        None => {
            vec!["Authenticated with an unmapped account status from the legacy schema.".into()]
        }
    };

    AuthSessionState {
        authenticated: true,
        user: Some(AuthSessionUser {
            id: user.id.max(0) as u64,
            name: user.name.clone(),
            email: user.email.clone(),
            organization_id: primary_membership
                .as_ref()
                .map(|membership| membership.organization_id.max(0) as u64),
            organization_role_key: primary_membership
                .as_ref()
                .map(|membership| membership.role_key.clone()),
            role_key: role_key(user.primary_role()),
            role_label: role_label(user.primary_role()),
            account_status_label: account_status_label(user.account_status()),
            dashboard_href: dashboard_href(user.primary_role(), user.account_status()).to_string(),
        }),
        permissions,
        notes,
    }
}

pub fn unauthenticated_session_state(note: &str) -> AuthSessionState {
    AuthSessionState {
        authenticated: false,
        user: None,
        permissions: Vec::new(),
        notes: vec![note.to_string()],
    }
}

pub fn bearer_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

pub async fn permission_keys_for_user(state: &AppState, user: &UserRecord) -> Vec<String> {
    let Some(role) = user.primary_role() else {
        return Vec::new();
    };

    if let Some(pool) = state.pool.as_ref() {
        if let Ok(mut dynamic_permissions) =
            list_permission_names_for_role(pool, i64::from(role.legacy_id())).await
        {
            if let Ok(Some(membership)) = primary_membership_for_user(pool, user.id).await {
                if let Ok(org_permissions) =
                    list_permission_keys_for_organization_role(pool, &membership.role_key).await
                {
                    dynamic_permissions.extend(org_permissions);
                    dynamic_permissions.sort();
                    dynamic_permissions.dedup();
                }
            }
            if !dynamic_permissions.is_empty() {
                return dynamic_permissions;
            }
        }
    }

    ROLE_PERMISSION_CONTRACTS
        .iter()
        .find(|contract| contract.role == role)
        .map(|contract| {
            contract
                .permissions
                .iter()
                .map(|permission| permission_key(*permission).to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn session_organization_id(session: &ResolvedSession) -> Option<i64> {
    session
        .session
        .user
        .as_ref()
        .and_then(|user| user.organization_id)
        .and_then(|value| i64::try_from(value).ok())
}

pub fn session_matches_organization(session: &ResolvedSession, organization_id: i64) -> bool {
    session_organization_id(session) == Some(organization_id)
}

pub fn role_key(role: Option<UserRole>) -> String {
    match role {
        Some(UserRole::Admin) => "admin".into(),
        Some(UserRole::Shipper) => "shipper".into(),
        Some(UserRole::Carrier) => "carrier".into(),
        Some(UserRole::Broker) => "broker".into(),
        Some(UserRole::FreightForwarder) => "freight_forwarder".into(),
        None => "unknown".into(),
    }
}

pub fn role_label(role: Option<UserRole>) -> String {
    role.map(|value| value.label().to_string())
        .unwrap_or_else(|| "Unknown".into())
}

pub fn dashboard_href(role: Option<UserRole>, status: Option<AccountStatus>) -> &'static str {
    if matches!(
        status,
        Some(AccountStatus::EmailVerifiedPendingOnboarding | AccountStatus::RevisionRequested)
    ) {
        return "/auth/onboarding";
    }

    role_descriptors()
        .iter()
        .find(|descriptor| Some(descriptor.role) == role)
        .map(|descriptor| descriptor.default_dashboard)
        .unwrap_or("/")
}

fn account_status_label(status: Option<AccountStatus>) -> String {
    match status {
        Some(AccountStatus::EmailVerifiedPendingOnboarding) => "Email Verified".into(),
        Some(AccountStatus::Approved) => "Approved".into(),
        Some(AccountStatus::Rejected) => "Rejected".into(),
        Some(AccountStatus::PendingReview) => "Pending Review".into(),
        Some(AccountStatus::PendingOtp) => "Pending OTP".into(),
        Some(AccountStatus::RevisionRequested) => "Revision Requested".into(),
        None => "Unknown".into(),
    }
}

fn permission_key(permission: Permission) -> &'static str {
    match permission {
        Permission::AccessAdminPortal => "access_admin_portal",
        Permission::ManageUsers => "manage_users",
        Permission::ManageRoles => "manage_roles",
        Permission::ManageMasterData => "manage_master_data",
        Permission::ManageLoads => "manage_loads",
        Permission::ManageDispatchDesk => "manage_dispatch_desk",
        Permission::ManageMarketplace => "manage_marketplace",
        Permission::ManageTracking => "manage_tracking",
        Permission::ManagePayments => "manage_payments",
        Permission::ManageTmsOperations => "manage_tms_operations",
    }
}

struct ParsedSessionToken<'a> {
    prefix: &'a str,
    secret: &'a str,
}

fn parse_session_token(token: &str) -> Option<ParsedSessionToken<'_>> {
    let stripped = token.trim().strip_prefix("stl_")?;
    let (prefix, secret) = stripped.split_once('.')?;
    let valid = !prefix.is_empty()
        && !secret.is_empty()
        && prefix.len() <= 32
        && prefix.chars().all(|ch| ch.is_ascii_hexdigit())
        && secret.chars().all(|ch| ch.is_ascii_hexdigit());

    valid.then_some(ParsedSessionToken { prefix, secret })
}

fn session_token_part() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn hash_session_token(secret: &str) -> String {
    let digest = Sha256::digest(secret.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn token_abilities_include(abilities: Option<&str>, expected: &str) -> bool {
    abilities
        .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .map(|items| items.iter().any(|item| item == expected))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::{bearer_token_from_headers, hash_session_token, parse_session_token};

    #[test]
    fn parses_prefixed_session_token() {
        let token = "stl_0123456789abcdef.abcdef0123456789";
        let parsed = parse_session_token(token).expect("token should parse");

        assert_eq!(parsed.prefix, "0123456789abcdef");
        assert_eq!(parsed.secret, "abcdef0123456789");
    }

    #[test]
    fn rejects_unprefixed_legacy_token() {
        assert!(parse_session_token("legacy-plain-token").is_none());
    }

    #[test]
    fn hashes_secret_without_returning_bearer_material() {
        let hash = hash_session_token("supersecret");

        assert_eq!(hash.len(), 64);
        assert_ne!(hash, "supersecret");
    }

    #[test]
    fn bearer_header_parser_requires_explicit_authorization_header() {
        let token = "stl_0123456789abcdef.abcdef0123456789";
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("valid header"),
        );

        assert_eq!(bearer_token_from_headers(&headers).as_deref(), Some(token));

        let mut cookie_only_headers = HeaderMap::new();
        cookie_only_headers.insert(
            header::COOKIE,
            HeaderValue::from_static("session=stl_0123456789abcdef.abcdef0123456789"),
        );

        assert!(bearer_token_from_headers(&cookie_only_headers).is_none());
    }

    #[test]
    fn bearer_header_parser_rejects_non_bearer_schemes() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic c3RsOnNlY3JldA=="),
        );

        assert!(bearer_token_from_headers(&headers).is_none());
    }
}
