use axum::http::{HeaderMap, header};
use chrono::Utc;
use db::auth::{
    UserRecord, delete_personal_access_token_by_token, find_personal_access_token_exact,
    find_user_by_id, insert_personal_access_token, touch_personal_access_token,
};
use domain::auth::{
    AccountStatus, Permission, ROLE_PERMISSION_CONTRACTS, UserRole, role_descriptors,
};
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

    let Some(token_record) = find_personal_access_token_exact(pool, token)
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
        let _ = delete_personal_access_token_by_token(pool, token).await;
        return Ok(None);
    }

    let Some(user) = find_user_by_id(pool, token_record.tokenable_id)
        .await
        .map_err(|error| format!("user lookup failed: {}", error))?
    else {
        return Ok(None);
    };

    let _ = touch_personal_access_token(pool, token_record.id).await;
    let session = build_session_state(&user);

    let _ = token_record;

    Ok(Some(ResolvedSession { user, session }))
}

pub async fn issue_session_token(state: &AppState, user: &UserRecord) -> Result<String, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Err("database is unavailable for login".into());
    };

    let token = uuid::Uuid::new_v4().to_string();
    let abilities = serde_json::to_string(&permission_keys_for_user(user)).ok();
    let expires_at = Some(Utc::now().naive_utc() + chrono::Duration::days(14));

    insert_personal_access_token(
        pool,
        user.id,
        "rust-session",
        &token,
        abilities.as_deref(),
        expires_at,
    )
    .await
    .map_err(|error| format!("token insert failed: {}", error))?;

    Ok(token)
}

pub async fn revoke_session_token(state: &AppState, token: &str) -> Result<u64, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(0);
    };

    delete_personal_access_token_by_token(pool, token)
        .await
        .map_err(|error| format!("token delete failed: {}", error))
}

pub fn build_session_state(user: &UserRecord) -> AuthSessionState {
    let permissions = permission_keys_for_user(user);
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
            role_key: role_key(user.primary_role()),
            role_label: role_label(user.primary_role()),
            account_status_label: account_status_label(user.account_status()),
            dashboard_href: dashboard_href(user.primary_role()).to_string(),
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

pub fn permission_keys_for_user(user: &UserRecord) -> Vec<String> {
    let Some(role) = user.primary_role() else {
        return Vec::new();
    };

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

pub fn dashboard_href(role: Option<UserRole>) -> &'static str {
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
