use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};
use bcrypt::verify;
use db::auth::find_user_by_email;
use domain::auth::{
    AccountStatusDescriptor, AuthModuleContract, PermissionDescriptor, RoleDescriptor,
    RolePermissionContract, account_status_descriptors, auth_module_contract,
    permission_descriptors, role_descriptors, role_permission_contracts,
};
use serde::Serialize;
use shared::{
    ApiResponse, AuthSessionState, LoginRequest, LoginResponse, LogoutResponse, RealtimeEvent,
    RealtimeEventKind,
};

use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, state::AppState};

#[derive(Debug, Serialize)]
struct AuthOverview {
    contract: AuthModuleContract,
    roles: Vec<RoleDescriptor>,
    account_statuses: Vec<AccountStatusDescriptor>,
    permissions: usize,
    role_permission_sets: usize,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/roles", get(roles))
        .route("/permissions", get(permissions))
        .route("/account-statuses", get(account_statuses))
        .route("/session-contract", get(session_contract))
        .route("/rbac-contract", get(rbac_contract))
        .route("/session", get(session))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

async fn index() -> Json<ApiResponse<AuthOverview>> {
    Json(ApiResponse::ok(AuthOverview {
        contract: auth_module_contract(),
        roles: role_descriptors().to_vec(),
        account_statuses: account_status_descriptors().to_vec(),
        permissions: permission_descriptors().len(),
        role_permission_sets: role_permission_contracts().len(),
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("auth route group ready"))
}

async fn roles() -> Json<ApiResponse<Vec<RoleDescriptor>>> {
    Json(ApiResponse::ok(role_descriptors().to_vec()))
}

async fn permissions() -> Json<ApiResponse<Vec<PermissionDescriptor>>> {
    Json(ApiResponse::ok(permission_descriptors().to_vec()))
}

async fn account_statuses() -> Json<ApiResponse<Vec<AccountStatusDescriptor>>> {
    Json(ApiResponse::ok(account_status_descriptors().to_vec()))
}

async fn session_contract() -> Json<ApiResponse<AuthModuleContract>> {
    Json(ApiResponse::ok(auth_module_contract()))
}

async fn rbac_contract() -> Json<ApiResponse<Vec<RolePermissionContract>>> {
    Json(ApiResponse::ok(role_permission_contracts().to_vec()))
}

async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<AuthSessionState>> {
    let session = match auth_session::resolve_session_from_headers(&state, &headers).await {
        Ok(Some(resolved)) => resolved.session,
        Ok(None) => auth_session::unauthenticated_session_state(
            "No bearer token was supplied to the Rust auth session endpoint.",
        ),
        Err(error) => auth_session::unauthenticated_session_state(&format!(
            "Failed to resolve the Rust auth session: {}",
            error
        )),
    };

    Json(ApiResponse::ok(session))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "Login is unavailable because the database connection is disabled.",
            ),
            message: format!(
                "Login is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let email = payload.email.trim().to_lowercase();
    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "No matching user exists for this email address.",
            ),
            message: "Invalid email or password.".into(),
        }));
    };

    let password_matches = verify(&payload.password, &user.password)
        .unwrap_or_else(|_| payload.password == user.password);

    if !password_matches {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "Password verification failed in the Rust auth layer.",
            ),
            message: "Invalid email or password.".into(),
        }));
    }

    match auth_session::issue_session_token(&state, &user).await {
        Ok(token) => {
            let session = auth_session::build_session_state(&user);
            Json(ApiResponse::ok(LoginResponse {
                success: true,
                token: Some(token),
                session,
                message: "Logged in through the Rust session layer.".into(),
            }))
        }
        Err(error) => Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(&error),
            message: format!("Login failed: {}", error),
        })),
    }
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LogoutResponse>> {
    let Some(token) = auth_session::bearer_token_from_headers(&headers) else {
        return Json(ApiResponse::ok(LogoutResponse {
            success: false,
            message: "No bearer token was supplied for logout.".into(),
        }));
    };

    let actor_user_id = auth_session::resolve_session_from_token(&state, &token)
        .await
        .ok()
        .flatten()
        .map(|resolved| resolved.user.id.max(0) as u64);

    match auth_session::revoke_session_token(&state, &token).await {
        Ok(_) => {
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::SessionInvalidated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id,
                    subject_user_id: actor_user_id,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: "Session token revoked from the Rust auth route.".into(),
                })
                .for_user_ids(actor_user_id.into_iter()),
            );

            Json(ApiResponse::ok(LogoutResponse {
                success: true,
                message: "Logged out from the Rust session layer.".into(),
            }))
        }
        Err(error) => Json(ApiResponse::ok(LogoutResponse {
            success: false,
            message: format!("Logout failed: {}", error),
        })),
    }
}
