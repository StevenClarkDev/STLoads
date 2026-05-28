use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use db::auth::{
    ChangeSelfPasswordInput, CreateKycDocumentInput, CreateUserInput, UpdateKycDocumentInput,
    UpdateSelfProfileInput, UpsertUserOnboardingInput, change_self_password,
    consume_password_reset_otp, consume_password_reset_token, consume_registration_otp,
    count_users_grouped_by_role, create_kyc_document, create_registered_user, delete_kyc_document,
    delete_personal_access_tokens_for_user, find_kyc_document_by_id, find_user_by_email,
    find_user_by_id, find_user_detail_by_user_id, list_kyc_documents_by_user_id, refresh_user_otp,
    revoke_all_access_artifacts_for_user, store_password_reset_token, update_kyc_document,
    update_self_profile, upsert_user_onboarding_details, verify_kyc_document_blockchain,
};
use db::enterprise_identity::{
    ScimDeprovisionInput, ScimTokenRecord, ScimUpsertUserInput, deprovision_scim_user,
    discover_sso_for_email, find_scim_token_by_hash, upsert_scim_user,
};
use db::legal_agreements::{
    AcceptLegalAgreementInput, LegalAgreementAcceptanceRecord, LegalAgreementTemplateRecord,
    accept_latest_legal_agreement, legal_acceptance_summary,
};
use domain::auth::{
    AccountStatus, AccountStatusDescriptor, AuthModuleContract, PermissionDescriptor,
    RoleDescriptor, RolePermissionContract, UserRole, account_status_descriptors,
    auth_module_contract, permission_descriptors, role_descriptors, role_permission_contracts,
};
use jsonwebtoken::{
    Algorithm, DecodingKey, TokenData, Validation, decode, decode_header, jwk::JwkSet,
};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use shared::{
    AcceptLegalAgreementRequest, AcceptLegalAgreementResponse, ApiResponse, AuthOnboardingDraft,
    AuthOnboardingScreen, AuthSessionState, CarrierCapacityProfile, ChangePasswordRequest,
    ChangePasswordResponse, DeleteKycDocumentResponse, EnterpriseSsoDiscoveryRequest,
    EnterpriseSsoDiscoveryResponse, EnterpriseSsoLoginResponse, EnterpriseSsoOidcCallbackRequest,
    ForgotPasswordRequest, ForgotPasswordResponse, KycDocumentItem, LegalAgreementAcceptanceItem,
    LegalAgreementScreen, LegalAgreementTemplateItem, LoginRequest, LoginResponse, LogoutResponse,
    MfaRecoveryCodesResponse, MfaVerifyRequest, MfaVerifyResponse, OtpPurpose,
    PortalRoleCountsResponse, RealtimeEvent, RealtimeEventKind, RegisterRequest, RegisterResponse,
    RequiredDocumentChecklistItem, ResendOtpRequest, ResendOtpResponse, ResetPasswordRequest,
    ResetPasswordResponse, ScimDeprovisionRequest, ScimDeprovisionResponse, ScimUpsertUserRequest,
    ScimUpsertUserResponse, SelfProfileDraft, SelfProfileFact, SelfProfileScreen,
    SubmitOnboardingRequest, SubmitOnboardingResponse, UpdateCarrierCapacityRequest,
    UpdateCarrierCapacityResponse, UpdateSelfProfileRequest, UpdateSelfProfileResponse,
    UpsertKycDocumentRequest, UpsertKycDocumentResponse, VerifyKycDocumentRequest,
    VerifyKycDocumentResponse, VerifyOtpRequest, VerifyOtpResponse,
};
use sqlx::Row;
use std::time::Duration as StdDuration;

use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    auth_session,
    document_validation::validate_uploaded_document,
    email::MailOutcome,
    rate_limit::{LockoutPolicy, RateLimitPolicy, rate_limit_identity},
    realtime_bus::RoutedRealtimeEvent,
    state::AppState,
};

#[derive(Debug, Serialize)]
struct AuthOverview {
    contract: AuthModuleContract,
    roles: Vec<RoleDescriptor>,
    account_statuses: Vec<AccountStatusDescriptor>,
    permissions: usize,
    role_permission_sets: usize,
}

#[derive(Debug, Serialize)]
struct NotificationCenterScreen {
    unread_count: u64,
    notifications: Vec<NotificationEventRow>,
    preferences: Vec<NotificationPreferenceRow>,
    provider_decisions: Vec<NotificationProviderDecisionRow>,
    coverage_rules: Vec<NotificationCoverageRuleRow>,
}

#[derive(Debug, Serialize)]
struct NotificationEventRow {
    id: u64,
    event_key: String,
    category: String,
    priority: String,
    subject: String,
    body: String,
    entity_type: Option<String>,
    entity_id: Option<u64>,
    action_href: Option<String>,
    channels: Vec<String>,
    delivery_status: String,
    read_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct NotificationPreferenceRow {
    id: u64,
    event_key: String,
    email_enabled: bool,
    in_app_enabled: bool,
    sms_enabled: bool,
    push_enabled: bool,
    quiet_hours_start: Option<String>,
    quiet_hours_end: Option<String>,
    timezone: String,
    escalation_minutes: Option<i32>,
}

#[derive(Debug, Serialize)]
struct NotificationProviderDecisionRow {
    channel: String,
    provider_name: String,
    decision_status: String,
    opt_in_required: bool,
    opt_out_required: bool,
    quiet_hours_required: bool,
    emergency_exception_allowed: bool,
    provider_audit_logs_required: bool,
    compliance_notes: String,
}

#[derive(Debug, Serialize)]
struct NotificationCoverageRuleRow {
    event_key: String,
    category: String,
    default_priority: String,
    default_channels: Vec<String>,
    responsible_party: String,
    entity_type: Option<String>,
    escalation_minutes: Option<i32>,
    active: bool,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpsertNotificationPreferenceRequest {
    event_key: String,
    email_enabled: bool,
    in_app_enabled: bool,
    sms_enabled: bool,
    push_enabled: bool,
    quiet_hours_start: Option<String>,
    quiet_hours_end: Option<String>,
    timezone: Option<String>,
    escalation_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct MarkNotificationReadRequest {
    notification_id: Option<u64>,
    mark_all: Option<bool>,
}

#[derive(Debug, Serialize)]
struct NotificationMutationResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct CommunicationGovernanceScreen {
    sender_identities: Vec<SenderIdentityRow>,
    delivery_events: Vec<DeliveryEventRow>,
    suppression_entries: Vec<SuppressionEntryRow>,
    template_governance: Vec<MessageTemplateGovernanceRow>,
    monitoring_rules: Vec<MessageMonitoringRuleRow>,
    branding_policies: Vec<TenantBrandingPolicyRow>,
    brand_assets: Vec<TenantBrandAssetRow>,
    custom_domains: Vec<TenantCustomDomainRow>,
    branded_template_rules: Vec<TenantBrandedTemplateRuleRow>,
}

#[derive(Debug, Serialize)]
struct SenderIdentityRow {
    id: u64,
    environment_key: String,
    sender_domain: String,
    from_email: String,
    from_name: String,
    spf_status: String,
    dkim_status: String,
    dmarc_status: String,
    identity_status: String,
    verified_at: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeliveryEventRow {
    id: u64,
    channel: String,
    event_type: String,
    provider_message_id: Option<String>,
    recipient: String,
    reason: Option<String>,
    occurred_at: String,
}

#[derive(Debug, Serialize)]
struct SuppressionEntryRow {
    id: u64,
    channel: String,
    recipient: String,
    suppression_reason: String,
    status: String,
    expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct MessageTemplateGovernanceRow {
    id: u64,
    template_key: String,
    channel: String,
    locale: String,
    version: i32,
    owner_team: String,
    approval_status: String,
    high_risk: bool,
    test_send_required: bool,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct MessageMonitoringRuleRow {
    rule_key: String,
    event_key: Option<String>,
    template_key: Option<String>,
    category: String,
    priority: String,
    required_sender_identity: bool,
    fallback_channel: String,
    escalation_minutes: i32,
    active: bool,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct TenantBrandingPolicyRow {
    organization_id: u64,
    portal_branding_enabled: bool,
    document_branding_enabled: bool,
    email_branding_enabled: bool,
    custom_domain_enabled: bool,
    white_label_status: String,
    unsupported_message: String,
    fallback_brand_name: String,
    cache_version: i32,
}

#[derive(Debug, Serialize)]
struct TenantBrandAssetRow {
    id: u64,
    asset_type: String,
    asset_url: String,
    mime_type: String,
    file_size_bytes: i64,
    review_status: String,
    cache_key: String,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct TenantCustomDomainRow {
    id: u64,
    domain: String,
    purpose: String,
    verification_status: String,
    dns_txt_name: String,
    dns_txt_value: String,
    tls_status: String,
    rollback_status: String,
    last_checked_at: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct TenantBrandedTemplateRuleRow {
    id: u64,
    template_key: String,
    template_surface: String,
    branding_status: String,
    fallback_allowed: bool,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageTemplateTestSendRequest {
    template_key: String,
    channel: String,
    recipient: String,
}

fn log_auth_failure(action: &str, email: &str, reason: &str) {
    warn!(action, email = %email, reason, "auth flow failed");
}

fn log_auth_success(action: &str, email: &str, user_id: Option<i64>) {
    info!(action, email = %email, user_id, "auth flow succeeded");
}

fn auth_flow_policy(name: &'static str) -> RateLimitPolicy {
    RateLimitPolicy::new(name, 10, StdDuration::from_secs(15 * 60))
}

fn upload_policy() -> RateLimitPolicy {
    RateLimitPolicy::new("auth_upload", 30, StdDuration::from_secs(60 * 60))
}

fn auth_lockout_policy() -> LockoutPolicy {
    LockoutPolicy::new(
        5,
        StdDuration::from_secs(15 * 60),
        StdDuration::from_secs(15 * 60),
    )
}

fn rate_limit_message(flow: &str, retry_after_seconds: u64) -> String {
    format!(
        "Too many {} attempts. Wait about {} seconds before trying again.",
        flow, retry_after_seconds
    )
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
        .route("/public-role-counts", get(public_role_counts))
        .route("/session", get(session))
        .route("/legal-agreements", get(legal_agreements_screen))
        .route("/legal-agreements/accept", post(accept_legal_agreement))
        .route("/sso/discovery", post(enterprise_sso_discovery))
        .route("/sso/oidc/callback", post(enterprise_sso_oidc_callback))
        .route("/scim/users", post(scim_upsert_user))
        .route("/scim/deprovision", post(scim_deprovision))
        .route("/login", post(login))
        .route("/mfa/verify", post(verify_mfa))
        .route(
            "/mfa/recovery-codes/regenerate",
            post(regenerate_mfa_recovery_codes),
        )
        .route("/logout", post(logout))
        .route("/register", post(register))
        .route("/verify-otp", post(verify_otp))
        .route("/otp/resend", post(resend_otp))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/onboarding-screen", get(onboarding_screen))
        .route("/onboarding", post(submit_onboarding))
        .route("/profile-screen", get(profile_screen))
        .route("/profile", post(update_profile))
        .route("/carrier-capacity", post(update_carrier_capacity))
        .route("/change-password", post(change_password))
        .route("/notifications", get(notification_center_screen))
        .route("/notifications/read", post(mark_notification_read))
        .route(
            "/communication-governance",
            get(communication_governance_screen),
        )
        .route(
            "/message-templates/test-send",
            post(record_message_template_test_send),
        )
        .route(
            "/notification-preferences",
            post(upsert_notification_preference),
        )
        .route(
            "/profile/documents/upload",
            post(upload_profile_kyc_document_handler)
                .layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/profile/documents/{document_id}",
            post(update_profile_kyc_document_handler),
        )
        .route(
            "/profile/documents/{document_id}/upload",
            post(replace_profile_kyc_document_handler)
                .layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/profile/documents/{document_id}/verify-blockchain",
            post(verify_profile_kyc_document_handler),
        )
        .route(
            "/profile/documents/{document_id}/delete",
            post(delete_profile_kyc_document_handler),
        )
        .route(
            "/onboarding/documents/upload",
            post(upload_kyc_document_handler).layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/onboarding/documents/{document_id}/file",
            get(download_kyc_document_file),
        )
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

async fn public_role_counts(
    State(state): State<AppState>,
) -> Json<ApiResponse<PortalRoleCountsResponse>> {
    let mut shipper_total = 0_u64;
    let mut carrier_total = 0_u64;
    let mut broker_total = 0_u64;
    let mut freight_forwarder_total = 0_u64;

    if let Some(pool) = state.pool.as_ref() {
        for row in count_users_grouped_by_role(pool).await.unwrap_or_default() {
            let total = row.total.max(0) as u64;
            match row.role_id.and_then(UserRole::from_legacy_id) {
                Some(UserRole::Shipper) => shipper_total = total,
                Some(UserRole::Carrier) => carrier_total = total,
                Some(UserRole::Broker) => broker_total = total,
                Some(UserRole::FreightForwarder) => freight_forwarder_total = total,
                _ => {}
            }
        }
    }

    Json(ApiResponse::ok(PortalRoleCountsResponse {
        shipper_total,
        carrier_total,
        broker_total,
        freight_forwarder_total,
    }))
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

async fn enterprise_sso_discovery(
    State(state): State<AppState>,
    Json(payload): Json<EnterpriseSsoDiscoveryRequest>,
) -> Json<ApiResponse<EnterpriseSsoDiscoveryResponse>> {
    let response = enterprise_sso_discovery_response(&state, &payload.email).await;
    Json(ApiResponse::ok(response))
}

async fn enterprise_sso_oidc_callback(
    State(state): State<AppState>,
    Json(payload): Json<EnterpriseSsoOidcCallbackRequest>,
) -> Result<Json<ApiResponse<EnterpriseSsoLoginResponse>>, StatusCode> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(EnterpriseSsoLoginResponse {
            success: false,
            token: None,
            session: None,
            created_user: false,
            message: format!(
                "Enterprise SSO is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let email = payload.email.trim().to_lowercase();
    let Some(discovery) = discover_sso_for_email(pool, &email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if discovery.provider_type.as_deref() != Some("oidc")
        || discovery.provider_status.as_deref() != Some("active")
        || !discovery.domain_verified
        || !discovery.login_routing_enabled
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let issuer = discovery
        .issuer
        .as_deref()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let client_id = discovery
        .client_id
        .as_deref()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let jwks_url = discovery
        .jwks_url
        .as_deref()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let token_data = validate_oidc_id_token(jwks_url, issuer, client_id, &payload.id_token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if token_data.claims.email.to_ascii_lowercase() != email {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if token_data.claims.email_verified == Some(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let (user, created_user) = match find_user_by_email(pool, &email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(user) => {
            revoke_all_access_artifacts_for_user(pool, user.id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (user, false)
        }
        None if discovery.jit_enabled => {
            let role = role_from_organization_role_key(discovery.default_role_key.as_deref());
            let input = ScimUpsertUserInput {
                organization_id: discovery.organization_id,
                external_id: token_data.claims.sub.clone(),
                email: email.clone(),
                name: token_data
                    .claims
                    .name
                    .clone()
                    .unwrap_or_else(|| email.clone()),
                password_hash: hash(format!("sso-{}", Uuid::new_v4()), 12)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
                role_id: role.legacy_id(),
                role_key: discovery
                    .default_role_key
                    .clone()
                    .unwrap_or_else(|| "member".into()),
                active: true,
                reason: Some("OIDC JIT login".into()),
                payload: serde_json::json!({
                    "provider_id": discovery.provider_id,
                    "issuer": issuer,
                    "subject": token_data.claims.sub,
                }),
            };
            let outcome = upsert_scim_user(pool, &input)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let user = find_user_by_id(pool, outcome.user_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            (user, true)
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let token = auth_session::issue_session_token(&state, &user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let session = auth_session::build_session_state(&state, &user).await;
    Ok(Json(ApiResponse::ok(EnterpriseSsoLoginResponse {
        success: true,
        token: Some(token),
        session: Some(session),
        created_user,
        message: "Enterprise OIDC login completed with verified identity claims.".into(),
    })))
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    let email = payload.email.trim().to_lowercase();
    let rate_identity = rate_limit_identity(&headers, &email);
    let rate_decision = state
        .check_rate_limit(auth_flow_policy("login"), &rate_identity)
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "Login is temporarily rate limited.",
            ),
            message: rate_limit_message("login", rate_decision.retry_after_seconds),
            mfa_required: false,
            mfa_challenge_id: None,
            mfa_expires_at: None,
            next_step: None,
            dev_mfa_code: None,
        }));
    }

    let lockout = state.lockout_status(auth_lockout_policy(), &email).await;
    if !lockout.allowed {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "This account is temporarily locked.",
            ),
            message: rate_limit_message("login", lockout.retry_after_seconds),
            mfa_required: false,
            mfa_challenge_id: None,
            mfa_expires_at: None,
            next_step: None,
            dev_mfa_code: None,
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_auth_failure(
            "login",
            payload.email.trim(),
            "database connection is disabled",
        );
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
            mfa_required: false,
            mfa_challenge_id: None,
            mfa_expires_at: None,
            next_step: None,
            dev_mfa_code: None,
        }));
    };

    let sso_discovery = enterprise_sso_discovery_response(&state, &email).await;
    if sso_discovery.sso_required && !sso_discovery.password_allowed {
        log_auth_failure("login", &email, "enterprise SSO required");
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "This organization requires enterprise SSO.",
            ),
            message: sso_discovery.message,
            mfa_required: false,
            mfa_challenge_id: None,
            mfa_expires_at: None,
            next_step: sso_discovery.sso_url,
            dev_mfa_code: None,
        }));
    }

    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        log_auth_failure("login", &email, "user not found");
        state
            .record_lockout_failure(auth_lockout_policy(), &email)
            .await;
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "No matching user exists for this email address.",
            ),
            message: "Invalid email or password.".into(),
            mfa_required: false,
            mfa_challenge_id: None,
            mfa_expires_at: None,
            next_step: None,
            dev_mfa_code: None,
        }));
    };

    let password_matches = verify_user_password(&payload.password, &user.password);

    if !password_matches {
        log_auth_failure("login", &email, "password verification failed");
        state
            .record_lockout_failure(auth_lockout_policy(), &email)
            .await;
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "Password verification failed in the Rust auth layer.",
            ),
            message: "Invalid email or password.".into(),
            mfa_required: false,
            mfa_challenge_id: None,
            mfa_expires_at: None,
            next_step: None,
            dev_mfa_code: None,
        }));
    }

    if privileged_user_requires_mfa(&user) {
        match create_mfa_challenge(&state, &user).await {
            Ok(challenge) => {
                log_auth_success("login_password_mfa_required", &email, Some(user.id));
                state.record_lockout_success(&email).await;
                return Json(ApiResponse::ok(LoginResponse {
                    success: false,
                    token: None,
                    session: auth_session::unauthenticated_session_state(
                        "MFA is required before this privileged session can start.",
                    ),
                    message: challenge.message,
                    mfa_required: true,
                    mfa_challenge_id: Some(challenge.challenge_id),
                    mfa_expires_at: Some(challenge.expires_at.to_rfc3339()),
                    next_step: Some(format!("/auth/mfa?email={}", email)),
                    dev_mfa_code: exposed_secret(&state, &challenge.code),
                }));
            }
            Err(error) => {
                log_auth_failure("login_password_mfa_required", &email, &error);
                return Json(ApiResponse::ok(LoginResponse {
                    success: false,
                    token: None,
                    session: auth_session::unauthenticated_session_state(&error),
                    message: error,
                    mfa_required: true,
                    mfa_challenge_id: None,
                    mfa_expires_at: None,
                    next_step: Some("/auth/login".into()),
                    dev_mfa_code: None,
                }));
            }
        }
    }

    match auth_session::issue_session_token(&state, &user).await {
        Ok(token) => {
            log_auth_success("login", &email, Some(user.id));
            state.record_lockout_success(&email).await;
            let session = auth_session::build_session_state(&state, &user).await;
            Json(ApiResponse::ok(LoginResponse {
                success: true,
                token: Some(token),
                session,
                message: login_message_for_status(user.account_status()),
                mfa_required: false,
                mfa_challenge_id: None,
                mfa_expires_at: None,
                next_step: None,
                dev_mfa_code: None,
            }))
        }
        Err(error) => {
            log_auth_failure(
                "login",
                &email,
                &format!("session issuance failed: {error}"),
            );
            Json(ApiResponse::ok(LoginResponse {
                success: false,
                token: None,
                session: auth_session::unauthenticated_session_state(&error),
                message: format!("Login failed: {}", error),
                mfa_required: false,
                mfa_challenge_id: None,
                mfa_expires_at: None,
                next_step: None,
                dev_mfa_code: None,
            }))
        }
    }
}

async fn scim_deprovision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ScimDeprovisionRequest>,
) -> Result<Json<ApiResponse<ScimDeprovisionResponse>>, StatusCode> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ScimDeprovisionResponse {
            success: false,
            organization_id: None,
            user_id: None,
            revoked_sessions: 0,
            membership_rows_changed: 0,
            user_rows_changed: 0,
            event_id: None,
            message: format!(
                "SCIM deprovisioning is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let token = authorize_scim_token(pool, &headers).await?;
    let user_id = payload.user_id.and_then(|value| i64::try_from(value).ok());
    let input = ScimDeprovisionInput {
        organization_id: token.organization_id,
        external_id: payload
            .external_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        email: payload
            .email
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        user_id,
        reason: payload
            .reason
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        payload: serde_json::to_value(&payload).unwrap_or_else(|_| serde_json::json!({})),
    };

    if input.external_id.is_none() && input.email.is_none() && input.user_id.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let outcome = deprovision_scim_user(pool, &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let revoked_sessions = if let Some(user_id) = outcome.user_id {
        revoke_all_access_artifacts_for_user(pool, user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        0
    };

    Ok(Json(ApiResponse::ok(ScimDeprovisionResponse {
        success: outcome.outcome == "accepted",
        organization_id: Some(outcome.organization_id.max(0) as u64),
        user_id: outcome.user_id.map(|value| value.max(0) as u64),
        revoked_sessions,
        membership_rows_changed: outcome.membership_rows_changed,
        user_rows_changed: outcome.user_rows_changed,
        event_id: Some(outcome.event_id.max(0) as u64),
        message: if outcome.outcome == "accepted" {
            "SCIM deprovisioning applied and active Rust sessions were revoked.".into()
        } else {
            "SCIM deprovisioning request recorded; no matching user was found.".into()
        },
    })))
}

async fn scim_upsert_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ScimUpsertUserRequest>,
) -> Result<Json<ApiResponse<ScimUpsertUserResponse>>, StatusCode> {
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ScimUpsertUserResponse {
            success: false,
            organization_id: None,
            user_id: None,
            created: false,
            reactivated: false,
            revoked_sessions: 0,
            event_id: None,
            message: format!(
                "SCIM user provisioning is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let token = authorize_scim_token(pool, &headers).await?;
    let email = payload.email.trim().to_lowercase();
    let name = payload.name.trim();
    let external_id = payload.external_id.trim();
    if !email.contains('@') || name.len() < 2 || external_id.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let role = payload
        .role_key
        .as_deref()
        .and_then(parse_role_key)
        .unwrap_or(UserRole::Carrier);
    let password_hash = hash(format!("scim-{}", Uuid::new_v4()), 12)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let input = ScimUpsertUserInput {
        organization_id: token.organization_id,
        external_id: external_id.to_string(),
        email,
        name: name.to_string(),
        password_hash,
        role_id: role.legacy_id(),
        role_key: organization_role_key_for_user_role(role).into(),
        active: payload.active,
        reason: payload
            .reason
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        payload: serde_json::to_value(&payload).unwrap_or_else(|_| serde_json::json!({})),
    };

    let outcome = upsert_scim_user(pool, &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let revoked_sessions = revoke_all_access_artifacts_for_user(pool, outcome.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(ScimUpsertUserResponse {
        success: true,
        organization_id: Some(outcome.organization_id.max(0) as u64),
        user_id: Some(outcome.user_id.max(0) as u64),
        created: outcome.created,
        reactivated: outcome.reactivated,
        revoked_sessions,
        event_id: Some(outcome.event_id.max(0) as u64),
        message: if outcome.created {
            "SCIM user provisioned and linked.".into()
        } else if outcome.reactivated {
            "SCIM user reactivated and active Rust sessions were rotated.".into()
        } else {
            "SCIM user updated and active Rust sessions were rotated.".into()
        },
    })))
}

async fn verify_mfa(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MfaVerifyRequest>,
) -> Json<ApiResponse<MfaVerifyResponse>> {
    let email = payload.email.trim().to_lowercase();
    let rate_identity = rate_limit_identity(&headers, format!("mfa:{email}"));
    let rate_decision = state
        .check_rate_limit(auth_flow_policy("verify_mfa"), &rate_identity)
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(MfaVerifyResponse {
            success: false,
            email,
            token: None,
            session: None,
            recovery_codes: Vec::new(),
            message: rate_limit_message("MFA verification", rate_decision.retry_after_seconds),
            next_step: "/auth/mfa".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(MfaVerifyResponse {
            success: false,
            email,
            token: None,
            session: None,
            recovery_codes: Vec::new(),
            message: "MFA verification is unavailable because the database is disabled.".into(),
            next_step: "/auth/login".into(),
        }));
    };

    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        return Json(ApiResponse::ok(MfaVerifyResponse {
            success: false,
            email,
            token: None,
            session: None,
            recovery_codes: Vec::new(),
            message: "Invalid or expired MFA challenge.".into(),
            next_step: "/auth/login".into(),
        }));
    };

    let code = payload.code.trim();
    let challenge_id = payload.challenge_id.trim();
    let code_hash = mfa_hash(code);
    let challenge_consumed = sqlx::query_scalar::<_, i64>(
        "UPDATE mfa_challenges
         SET consumed_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1::uuid
           AND user_id = $2
           AND email = $3
           AND code_hash = $4
           AND consumed_at IS NULL
           AND expires_at > CURRENT_TIMESTAMP
         RETURNING user_id",
    )
    .bind(challenge_id)
    .bind(user.id)
    .bind(&email)
    .bind(&code_hash)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some();

    let recovery_consumed = if challenge_consumed {
        false
    } else {
        consume_mfa_recovery_code(pool, user.id, code)
            .await
            .unwrap_or(false)
    };

    if !challenge_consumed && !recovery_consumed {
        state
            .record_lockout_failure(auth_lockout_policy(), format!("mfa:{email}"))
            .await;
        return Json(ApiResponse::ok(MfaVerifyResponse {
            success: false,
            email,
            token: None,
            session: None,
            recovery_codes: Vec::new(),
            message: "Invalid or expired MFA code.".into(),
            next_step: "/auth/mfa".into(),
        }));
    }

    match auth_session::issue_session_token_with_mfa(&state, &user, true).await {
        Ok(token) => {
            state
                .record_lockout_success(format!("mfa:{}", user.email.to_ascii_lowercase()))
                .await;
            let mut session = auth_session::build_session_state(&state, &user).await;
            if !session
                .permissions
                .iter()
                .any(|permission| permission == "mfa_verified")
            {
                session.permissions.push("mfa_verified".into());
            }
            let recovery_codes = ensure_mfa_recovery_codes(pool, user.id)
                .await
                .unwrap_or_default();
            let next_step = session
                .user
                .as_ref()
                .map(|user| user.dashboard_href.clone())
                .unwrap_or_else(|| "/".into());

            Json(ApiResponse::ok(MfaVerifyResponse {
                success: true,
                email,
                token: Some(token),
                session: Some(session),
                recovery_codes,
                message: "MFA verified. Your privileged Rust session is live.".into(),
                next_step,
            }))
        }
        Err(error) => Json(ApiResponse::ok(MfaVerifyResponse {
            success: false,
            email,
            token: None,
            session: None,
            recovery_codes: Vec::new(),
            message: format!("MFA verified, but session issuance failed: {}", error),
            next_step: "/auth/login".into(),
        })),
    }
}

async fn regenerate_mfa_recovery_codes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<MfaRecoveryCodesResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(MfaRecoveryCodesResponse {
            success: false,
            recovery_codes: Vec::new(),
            message: "Sign in before regenerating MFA recovery codes.".into(),
        }));
    };

    if !resolved
        .session
        .permissions
        .iter()
        .any(|permission| permission == "mfa_verified")
    {
        return Json(ApiResponse::ok(MfaRecoveryCodesResponse {
            success: false,
            recovery_codes: Vec::new(),
            message: "MFA step-up is required before regenerating recovery codes.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(MfaRecoveryCodesResponse {
            success: false,
            recovery_codes: Vec::new(),
            message: "MFA recovery code reset is unavailable because the database is disabled."
                .into(),
        }));
    };

    match regenerate_mfa_recovery_code_set(pool, resolved.user.id).await {
        Ok(recovery_codes) => Json(ApiResponse::ok(MfaRecoveryCodesResponse {
            success: true,
            recovery_codes,
            message:
                "MFA recovery codes were regenerated. Store them securely; they are shown once."
                    .into(),
        })),
        Err(error) => Json(ApiResponse::ok(MfaRecoveryCodesResponse {
            success: false,
            recovery_codes: Vec::new(),
            message: format!("MFA recovery code reset failed: {}", error),
        })),
    }
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LogoutResponse>> {
    let Some(token) = auth_session::bearer_token_from_headers(&headers) else {
        warn!(
            action = "logout",
            reason = "missing bearer token",
            "auth flow failed"
        );
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
            info!(action = "logout", actor_user_id, "auth flow succeeded");
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
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
        Err(error) => {
            warn!(action = "logout", actor_user_id, error = %error, "auth flow failed");
            Json(ApiResponse::ok(LogoutResponse {
                success: false,
                message: format!("Logout failed: {}", error),
            }))
        }
    }
}

async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> Json<ApiResponse<RegisterResponse>> {
    let email = payload.email.trim().to_lowercase();
    let rate_decision = state
        .check_rate_limit(
            auth_flow_policy("register"),
            rate_limit_identity(&headers, &email),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: payload.role_key,
            next_step: "/auth/register".into(),
            message: rate_limit_message("registration", rate_decision.retry_after_seconds),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_auth_failure(
            "register",
            payload.email.trim(),
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email: payload.email,
            role_key: payload.role_key,
            next_step: "/auth/login".into(),
            message:
                "Registration is unavailable because the Rust database connection is disabled."
                    .into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    let name = payload.name.trim().to_string();
    let role = match parse_role_key(&payload.role_key) {
        Some(UserRole::Admin) | None => {
            return Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: payload.role_key,
                next_step: "/auth/register".into(),
                message: "Choose a valid business role before continuing.".into(),
                otp_expires_at: None,
                dev_otp: None,
            }));
        }
        Some(role) => role,
    };

    if name.is_empty() || email.is_empty() {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: "Name and email are required for Rust registration.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    if payload.password != payload.password_confirmation {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: "Password confirmation does not match.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    if payload.password.len() < 8 {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: "Use a password with at least 8 characters.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    if find_user_by_email(pool, &email)
        .await
        .unwrap_or(None)
        .is_some()
    {
        log_auth_failure("register", &email, "email already exists");
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email: email.clone(),
            role_key: role_key(role).into(),
            next_step: format!("/auth/login?email={}", email),
            message: "An account with this email already exists.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    let password_hash = match hash(&payload.password, 12) {
        Ok(value) => value,
        Err(error) => {
            log_auth_failure(
                "register",
                &email,
                &format!("password hashing failed: {error}"),
            );
            return Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: role_key(role).into(),
                next_step: "/auth/register".into(),
                message: format!("Password hashing failed: {}", error),
                otp_expires_at: None,
                dev_otp: None,
            }));
        }
    };

    let otp = generate_otp();
    let otp_expires_at = Utc::now() + Duration::minutes(5);

    let input = CreateUserInput {
        name: name.clone(),
        email: email.clone(),
        password_hash,
        role_id: role.legacy_id(),
        phone_no: payload.phone_no.filter(|value| !value.trim().is_empty()),
        address: payload.address.filter(|value| !value.trim().is_empty()),
        otp: otp.clone(),
        otp_expires_at: otp_expires_at.naive_utc(),
    };

    let mail_outcome = match state
        .email
        .send_registration_otp(&email, Some(&name), &otp)
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            log_auth_failure(
                "register",
                &email,
                &format!("registration otp email failed: {error}"),
            );
            return Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: role_key(role).into(),
                next_step: "/auth/register".into(),
                message: format!("Registration OTP email failed: {}", error),
                otp_expires_at: None,
                dev_otp: None,
            }));
        }
    };

    match create_registered_user(pool, &input).await {
        Ok(_) => {
            info!(
                action = "register",
                email = %email,
                role = role_key(role),
                "auth flow succeeded"
            );
            Json(ApiResponse::ok(RegisterResponse {
                success: true,
                email: email.clone(),
                role_key: role_key(role).into(),
                next_step: format!(
                    "/auth/verify-otp?email={}&purpose={}",
                    email,
                    OtpPurpose::Registration.as_key()
                ),
                message: mail_outcome.append_to_message(format!(
                    "Rust registration created. Verify the 6-digit OTP for {} to continue.",
                    email
                )),
                otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                dev_otp: exposed_secret(&state, &otp),
            }))
        }
        Err(error) => {
            log_auth_failure(
                "register",
                &email,
                &format!("user creation failed: {error}"),
            );
            Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: role_key(role).into(),
                next_step: "/auth/register".into(),
                message: format!("Registration failed: {}", error),
                otp_expires_at: None,
                dev_otp: None,
            }))
        }
    }
}

async fn verify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VerifyOtpRequest>,
) -> Json<ApiResponse<VerifyOtpResponse>> {
    let email = payload.email.trim().to_lowercase();
    let lockout_identity = format!("otp:{}:{}", payload.purpose.as_key(), email);
    let rate_decision = state
        .check_rate_limit(
            auth_flow_policy("verify_otp"),
            rate_limit_identity(&headers, &lockout_identity),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(VerifyOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            next_step: "/auth/verify-otp".into(),
            message: rate_limit_message("OTP verification", rate_decision.retry_after_seconds),
            token: None,
            session: None,
            reset_token: None,
        }));
    }

    let lockout = state
        .lockout_status(auth_lockout_policy(), &lockout_identity)
        .await;
    if !lockout.allowed {
        return Json(ApiResponse::ok(VerifyOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            next_step: "/auth/verify-otp".into(),
            message: rate_limit_message("OTP verification", lockout.retry_after_seconds),
            token: None,
            session: None,
            reset_token: None,
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_auth_failure(
            "verify_otp",
            payload.email.trim(),
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(VerifyOtpResponse {
            success: false,
            email: payload.email,
            purpose: payload.purpose,
            next_step: "/auth/login".into(),
            message:
                "OTP verification is unavailable because the Rust database connection is disabled."
                    .into(),
            token: None,
            session: None,
            reset_token: None,
        }));
    };

    let otp = payload.otp.trim().to_string();
    if otp.len() != 6 || !otp.chars().all(|ch| ch.is_ascii_digit()) {
        log_auth_failure("verify_otp", &email, "invalid otp format");
        state
            .record_lockout_failure(auth_lockout_policy(), &lockout_identity)
            .await;
        return Json(ApiResponse::ok(VerifyOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            next_step: format!(
                "/auth/verify-otp?email={}&purpose={}",
                payload.email.trim().to_lowercase(),
                payload.purpose.as_key()
            ),
            message: "Enter the full 6-digit OTP code.".into(),
            token: None,
            session: None,
            reset_token: None,
        }));
    }

    match payload.purpose {
        OtpPurpose::Registration => match consume_registration_otp(
            pool,
            &email,
            &otp,
            AccountStatus::EmailVerifiedPendingOnboarding.legacy_code(),
        )
        .await
        {
            Ok(Some(user)) => match auth_session::issue_session_token(&state, &user).await {
                Ok(token) => {
                    log_auth_success("verify_registration_otp", &email, Some(user.id));
                    state.record_lockout_success(&lockout_identity).await;
                    let session = auth_session::build_session_state(&state, &user).await;
                    Json(ApiResponse::ok(VerifyOtpResponse {
                        success: true,
                        email,
                        purpose: OtpPurpose::Registration,
                        next_step: session
                            .user
                            .as_ref()
                            .map(|item| item.dashboard_href.clone())
                            .unwrap_or_else(|| "/auth/onboarding".into()),
                        message: "Email verified. Your Rust session is live and onboarding is the next required step.".into(),
                        token: Some(token),
                        session: Some(session),
                        reset_token: None,
                    }))
                }
                Err(error) => {
                    log_auth_failure(
                        "verify_registration_otp",
                        &email,
                        &format!("session issuance failed: {error}"),
                    );
                    Json(ApiResponse::ok(VerifyOtpResponse {
                        success: false,
                        email,
                        purpose: OtpPurpose::Registration,
                        next_step: "/auth/login".into(),
                        message: format!("OTP verified, but session issuance failed: {}", error),
                        token: None,
                        session: None,
                        reset_token: None,
                    }))
                }
            },
            Ok(None) => {
                log_auth_failure("verify_registration_otp", &email, "invalid or expired otp");
                state
                    .record_lockout_failure(auth_lockout_policy(), &lockout_identity)
                    .await;
                Json(ApiResponse::ok(VerifyOtpResponse {
                    success: false,
                    email,
                    purpose: OtpPurpose::Registration,
                    next_step: format!(
                        "/auth/verify-otp?email={}&purpose={}",
                        payload.email.trim().to_lowercase(),
                        OtpPurpose::Registration.as_key()
                    ),
                    message: "Invalid or expired OTP.".into(),
                    token: None,
                    session: None,
                    reset_token: None,
                }))
            }
            Err(error) => {
                log_auth_failure(
                    "verify_registration_otp",
                    &email,
                    &format!("otp verification failed: {error}"),
                );
                Json(ApiResponse::ok(VerifyOtpResponse {
                    success: false,
                    email,
                    purpose: OtpPurpose::Registration,
                    next_step: "/auth/register".into(),
                    message: format!("OTP verification failed: {}", error),
                    token: None,
                    session: None,
                    reset_token: None,
                }))
            }
        },
        OtpPurpose::PasswordReset => match consume_password_reset_otp(pool, &email, &otp).await {
            Ok(Some(_)) => {
                let reset_token = uuid::Uuid::new_v4().to_string();
                match store_password_reset_token(pool, &email, &reset_token).await {
                    Ok(_) => {
                        log_auth_success("verify_password_reset_otp", &email, None);
                        state.record_lockout_success(&lockout_identity).await;
                        Json(ApiResponse::ok(VerifyOtpResponse {
                            success: true,
                            email,
                            purpose: OtpPurpose::PasswordReset,
                            next_step: format!(
                                "/auth/reset-password?email={}",
                                payload.email.trim().to_lowercase()
                            ),
                            message: "OTP verified. Set a new password in the Rust reset form."
                                .into(),
                            token: None,
                            session: None,
                            reset_token: exposed_secret(&state, &reset_token),
                        }))
                    }
                    Err(error) => {
                        log_auth_failure(
                            "verify_password_reset_otp",
                            &email,
                            &format!("failed to issue password reset token: {error}"),
                        );
                        Json(ApiResponse::ok(VerifyOtpResponse {
                            success: false,
                            email,
                            purpose: OtpPurpose::PasswordReset,
                            next_step: "/auth/forgot-password".into(),
                            message: format!("Failed to issue password reset token: {}", error),
                            token: None,
                            session: None,
                            reset_token: None,
                        }))
                    }
                }
            }
            Ok(None) => {
                log_auth_failure(
                    "verify_password_reset_otp",
                    &email,
                    "invalid or expired otp",
                );
                state
                    .record_lockout_failure(auth_lockout_policy(), &lockout_identity)
                    .await;
                Json(ApiResponse::ok(VerifyOtpResponse {
                    success: false,
                    email,
                    purpose: OtpPurpose::PasswordReset,
                    next_step: format!(
                        "/auth/verify-otp?email={}&purpose={}",
                        payload.email.trim().to_lowercase(),
                        OtpPurpose::PasswordReset.as_key()
                    ),
                    message: "Invalid or expired OTP.".into(),
                    token: None,
                    session: None,
                    reset_token: None,
                }))
            }
            Err(error) => {
                log_auth_failure(
                    "verify_password_reset_otp",
                    &email,
                    &format!("otp verification failed: {error}"),
                );
                Json(ApiResponse::ok(VerifyOtpResponse {
                    success: false,
                    email,
                    purpose: OtpPurpose::PasswordReset,
                    next_step: "/auth/forgot-password".into(),
                    message: format!("OTP verification failed: {}", error),
                    token: None,
                    session: None,
                    reset_token: None,
                }))
            }
        },
    }
}

async fn resend_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ResendOtpRequest>,
) -> Json<ApiResponse<ResendOtpResponse>> {
    let email = payload.email.trim().to_lowercase();
    let rate_subject = format!("otp-resend:{}:{}", payload.purpose.as_key(), email);
    let rate_decision = state
        .check_rate_limit(
            auth_flow_policy("resend_otp"),
            rate_limit_identity(&headers, &rate_subject),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(ResendOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            message: rate_limit_message("OTP resend", rate_decision.retry_after_seconds),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_auth_failure(
            "resend_otp",
            payload.email.trim(),
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(ResendOtpResponse {
            success: false,
            email: payload.email,
            purpose: payload.purpose,
            message: "OTP resend is unavailable because the Rust database connection is disabled."
                .into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        log_auth_failure("resend_otp", &email, "user not found");
        return Json(ApiResponse::ok(ResendOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            message: "No Rust user was found for this email address.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    match next_resend_count(user.last_otp_resend_at, user.otp_resend_count) {
        Ok(resend_count) => {
            let otp = generate_otp();
            let otp_expires_at = Utc::now() + Duration::minutes(5);
            match refresh_user_otp(
                pool,
                user.id,
                &otp,
                otp_expires_at.naive_utc(),
                resend_count,
            )
            .await
            {
                Ok(_) => {
                    let mail_outcome = match send_otp_notification(
                        &state,
                        payload.purpose,
                        &email,
                        Some(&user.name),
                        &otp,
                    )
                    .await
                    {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            log_auth_failure(
                                "resend_otp",
                                &email,
                                &format!("otp refreshed but email delivery failed: {error}"),
                            );
                            return Json(ApiResponse::ok(ResendOtpResponse {
                                success: false,
                                email,
                                purpose: payload.purpose,
                                message: format!(
                                    "OTP was refreshed, but email delivery failed: {}",
                                    error
                                ),
                                otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                                dev_otp: exposed_secret(&state, &otp),
                            }));
                        }
                    };

                    log_auth_success("resend_otp", &email, Some(user.id));
                    Json(ApiResponse::ok(ResendOtpResponse {
                        success: true,
                        email,
                        purpose: payload.purpose,
                        message: mail_outcome.append_to_message(resend_message(payload.purpose)),
                        otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                        dev_otp: exposed_secret(&state, &otp),
                    }))
                }
                Err(error) => {
                    log_auth_failure("resend_otp", &email, &format!("otp resend failed: {error}"));
                    Json(ApiResponse::ok(ResendOtpResponse {
                        success: false,
                        email,
                        purpose: payload.purpose,
                        message: format!("OTP resend failed: {}", error),
                        otp_expires_at: None,
                        dev_otp: None,
                    }))
                }
            }
        }
        Err(message) => {
            log_auth_failure("resend_otp", &email, &message);
            Json(ApiResponse::ok(ResendOtpResponse {
                success: false,
                email,
                purpose: payload.purpose,
                message,
                otp_expires_at: None,
                dev_otp: None,
            }))
        }
    }
}

async fn forgot_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Json<ApiResponse<ForgotPasswordResponse>> {
    let email = payload.email.trim().to_lowercase();
    let rate_decision = state
        .check_rate_limit(
            auth_flow_policy("forgot_password"),
            rate_limit_identity(&headers, &email),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(ForgotPasswordResponse {
            success: false,
            email,
            next_step: "/auth/forgot-password".into(),
            message: rate_limit_message(
                "password reset request",
                rate_decision.retry_after_seconds,
            ),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_auth_failure(
            "forgot_password",
            payload.email.trim(),
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(ForgotPasswordResponse {
            success: false,
            email: payload.email,
            next_step: "/auth/login".into(),
            message:
                "Password reset is unavailable because the Rust database connection is disabled."
                    .into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        log_auth_failure("forgot_password", &email, "user not found");
        return Json(ApiResponse::ok(ForgotPasswordResponse {
            success: false,
            email,
            next_step: "/auth/forgot-password".into(),
            message: "No Rust user was found for this email address.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    match next_resend_count(user.last_otp_resend_at, user.otp_resend_count) {
        Ok(resend_count) => {
            let otp = generate_otp();
            let otp_expires_at = Utc::now() + Duration::minutes(5);
            match refresh_user_otp(
                pool,
                user.id,
                &otp,
                otp_expires_at.naive_utc(),
                resend_count,
            )
            .await
            {
                Ok(_) => {
                    let mail_outcome = match state
                        .email
                        .send_password_reset_otp(&email, Some(&user.name), &otp)
                        .await
                    {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            log_auth_failure(
                                "forgot_password",
                                &email,
                                &format!(
                                    "password reset otp refreshed but email delivery failed: {error}"
                                ),
                            );
                            return Json(ApiResponse::ok(ForgotPasswordResponse {
                                success: false,
                                email,
                                next_step: "/auth/forgot-password".into(),
                                message: format!(
                                    "Password reset OTP was refreshed, but email delivery failed: {}",
                                    error
                                ),
                                otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                                dev_otp: exposed_secret(&state, &otp),
                            }));
                        }
                    };

                    log_auth_success("forgot_password", &email, Some(user.id));
                    Json(ApiResponse::ok(ForgotPasswordResponse {
                        success: true,
                        email: email.clone(),
                        next_step: format!(
                            "/auth/verify-otp?email={}&purpose={}",
                            email,
                            OtpPurpose::PasswordReset.as_key()
                        ),
                        message: mail_outcome.append_to_message(
                            "A password reset OTP is ready for verification in the Rust auth flow.",
                        ),
                        otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                        dev_otp: exposed_secret(&state, &otp),
                    }))
                }
                Err(error) => {
                    log_auth_failure(
                        "forgot_password",
                        &email,
                        &format!("failed to issue password reset otp: {error}"),
                    );
                    Json(ApiResponse::ok(ForgotPasswordResponse {
                        success: false,
                        email,
                        next_step: "/auth/forgot-password".into(),
                        message: format!("Failed to issue a password reset OTP: {}", error),
                        otp_expires_at: None,
                        dev_otp: None,
                    }))
                }
            }
        }
        Err(message) => {
            log_auth_failure("forgot_password", &email, &message);
            Json(ApiResponse::ok(ForgotPasswordResponse {
                success: false,
                email,
                next_step: "/auth/forgot-password".into(),
                message,
                otp_expires_at: None,
                dev_otp: None,
            }))
        }
    }
}

async fn reset_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ResetPasswordRequest>,
) -> Json<ApiResponse<ResetPasswordResponse>> {
    let email = payload.email.trim().to_lowercase();
    let rate_decision = state
        .check_rate_limit(
            auth_flow_policy("reset_password"),
            rate_limit_identity(&headers, &email),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: "/auth/reset-password".into(),
            message: rate_limit_message("password reset", rate_decision.retry_after_seconds),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_auth_failure(
            "reset_password",
            payload.email.trim(),
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email: payload.email,
            next_step: "/auth/login".into(),
            message:
                "Password reset is unavailable because the Rust database connection is disabled."
                    .into(),
        }));
    };

    if payload.password != payload.password_confirmation {
        log_auth_failure("reset_password", &email, "password confirmation mismatch");
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: format!(
                "/auth/reset-password?email={}",
                payload.email.trim().to_lowercase()
            ),
            message: "Password confirmation does not match.".into(),
        }));
    }

    if payload.password.len() < 8 {
        log_auth_failure("reset_password", &email, "password too short");
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: format!(
                "/auth/reset-password?email={}",
                payload.email.trim().to_lowercase()
            ),
            message: "Use a password with at least 8 characters.".into(),
        }));
    }

    let password_hash = match hash(&payload.password, 12) {
        Ok(value) => value,
        Err(error) => {
            log_auth_failure(
                "reset_password",
                &email,
                &format!("password hashing failed: {error}"),
            );
            return Json(ApiResponse::ok(ResetPasswordResponse {
                success: false,
                email,
                next_step: format!(
                    "/auth/reset-password?email={}",
                    payload.email.trim().to_lowercase()
                ),
                message: format!("Password hashing failed: {}", error),
            }));
        }
    };

    match consume_password_reset_token(pool, &email, payload.reset_token.trim(), &password_hash)
        .await
    {
        Ok(true) => {
            if let Ok(Some(user)) = find_user_by_email(pool, &email).await {
                let _ = delete_personal_access_tokens_for_user(pool, user.id).await;
            }
            log_auth_success("reset_password", &email, None);
            Json(ApiResponse::ok(ResetPasswordResponse {
                success: true,
                email,
                next_step: format!("/auth/login?email={}", payload.email.trim().to_lowercase()),
                message: "Password updated in the Rust auth flow. Sign in with the new password."
                    .into(),
            }))
        }
        Ok(false) => {
            log_auth_failure("reset_password", &email, "invalid or expired reset token");
            Json(ApiResponse::ok(ResetPasswordResponse {
                success: false,
                email,
                next_step: format!(
                    "/auth/reset-password?email={}",
                    payload.email.trim().to_lowercase()
                ),
                message: "The reset token is invalid or expired.".into(),
            }))
        }
        Err(error) => {
            log_auth_failure(
                "reset_password",
                &email,
                &format!("password reset failed: {error}"),
            );
            Json(ApiResponse::ok(ResetPasswordResponse {
                success: false,
                email,
                next_step: "/auth/forgot-password".into(),
                message: format!("Password reset failed: {}", error),
            }))
        }
    }
}

async fn onboarding_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<AuthOnboardingScreen>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(unauthenticated_onboarding_screen()));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(onboarding_screen_from_user(
            &resolved.user,
            None,
            Vec::new(),
            false,
            vec!["Rust onboarding is waiting for the database connection to come back.".into()],
        )));
    };

    let details = find_user_detail_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or(None);

    let documents = list_kyc_documents_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or_default();
    let can_submit = can_submit_onboarding(resolved.user.account_status());
    let mut notes = onboarding_notes_for_user(&resolved.user);
    notes.push(
        "KYC uploads now use the Rust secure document path, and files are viewable only by admin users plus the profile that uploaded them."
            .into(),
    );

    Json(ApiResponse::ok(onboarding_screen_from_user(
        &resolved.user,
        details,
        documents,
        can_submit,
        notes,
    )))
}

async fn legal_agreements_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LegalAgreementScreen>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(LegalAgreementScreen {
            title: "Legal Agreements".into(),
            missing_required: Vec::new(),
            acceptance_proofs: Vec::new(),
            notes: vec!["Sign in before reviewing required legal agreements.".into()],
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(LegalAgreementScreen {
            title: "Legal Agreements".into(),
            missing_required: Vec::new(),
            acceptance_proofs: Vec::new(),
            notes: vec![format!(
                "Legal agreements are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        }));
    };

    Json(ApiResponse::ok(
        legal_agreement_screen_for_user(pool, &resolved.user)
            .await
            .unwrap_or_else(|error| LegalAgreementScreen {
                title: "Legal Agreements".into(),
                missing_required: Vec::new(),
                acceptance_proofs: Vec::new(),
                notes: vec![format!("Legal agreement lookup failed: {}", error)],
            }),
    ))
}

async fn accept_legal_agreement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AcceptLegalAgreementRequest>,
) -> Json<ApiResponse<AcceptLegalAgreementResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(AcceptLegalAgreementResponse {
            success: false,
            message: "Sign in before accepting legal agreements.".into(),
            screen: LegalAgreementScreen {
                title: "Legal Agreements".into(),
                missing_required: Vec::new(),
                acceptance_proofs: Vec::new(),
                notes: Vec::new(),
            },
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(AcceptLegalAgreementResponse {
            success: false,
            message: "Legal agreement acceptance is unavailable while the database is offline."
                .into(),
            screen: LegalAgreementScreen {
                title: "Legal Agreements".into(),
                missing_required: Vec::new(),
                acceptance_proofs: Vec::new(),
                notes: Vec::new(),
            },
        }));
    };

    let agreement_key = payload.agreement_key.trim();
    if agreement_key.is_empty() {
        let screen = legal_agreement_screen_for_user(pool, &resolved.user)
            .await
            .unwrap_or_else(|_| LegalAgreementScreen {
                title: "Legal Agreements".into(),
                missing_required: Vec::new(),
                acceptance_proofs: Vec::new(),
                notes: Vec::new(),
            });
        return Json(ApiResponse::ok(AcceptLegalAgreementResponse {
            success: false,
            message: "Choose a legal agreement before accepting.".into(),
            screen,
        }));
    }

    let role_key_value = resolved.user.primary_role().map(role_key);
    let result = accept_latest_legal_agreement(
        pool,
        &AcceptLegalAgreementInput {
            agreement_key,
            signer_user_id: resolved.user.id,
            organization_id: resolved.user.organization_id,
            role_key: role_key_value,
            signer_name: &resolved.user.name,
            signer_email: &resolved.user.email,
            ip_address: header_value(&headers, "x-forwarded-for"),
            user_agent: header_value(&headers, "user-agent"),
            request_id: header_value(&headers, crate::app::REQUEST_ID_HEADER),
            accept_for_organization: payload.accept_for_organization,
        },
    )
    .await;

    let screen = legal_agreement_screen_for_user(pool, &resolved.user)
        .await
        .unwrap_or_else(|_| LegalAgreementScreen {
            title: "Legal Agreements".into(),
            missing_required: Vec::new(),
            acceptance_proofs: Vec::new(),
            notes: Vec::new(),
        });

    match result {
        Ok(acceptance) => Json(ApiResponse::ok(AcceptLegalAgreementResponse {
            success: true,
            message: format!(
                "Accepted {} version {} with audit evidence.",
                acceptance.agreement_key, acceptance.version
            ),
            screen,
        })),
        Err(error) => Json(ApiResponse::ok(AcceptLegalAgreementResponse {
            success: false,
            message: format!("Legal agreement acceptance failed: {}", error),
            screen,
        })),
    }
}

async fn submit_onboarding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SubmitOnboardingRequest>,
) -> Json<ApiResponse<SubmitOnboardingResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: None,
            next_step: "/auth/login".into(),
            message: "Sign in before submitting Rust onboarding.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: None,
            next_step: "/auth/onboarding".into(),
            message:
                "Rust onboarding cannot save right now because the database connection is disabled."
                    .into(),
        }));
    };

    if resolved.user.email_verified_at.is_none() {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/verify-otp".into(),
            message: "Verify the account OTP before continuing into onboarding.".into(),
        }));
    }

    if !can_submit_onboarding(resolved.user.account_status()) {
        let next_step = resolved
            .session
            .user
            .as_ref()
            .map(|item| item.dashboard_href.clone())
            .unwrap_or_else(|| "/".into());
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step,
            message: "This account is not currently editable through the Rust onboarding form."
                .into(),
        }));
    }

    if payload.company_name.trim().is_empty() || payload.company_address.trim().is_empty() {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message:
                "Company name and company address are required before Rust onboarding can continue."
                    .into(),
        }));
    }

    let role = resolved.user.primary_role();
    if let Some(validation_message) = validate_role_specific_onboarding(role, &payload) {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message: validation_message,
        }));
    }

    let documents = list_kyc_documents_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|document| build_kyc_document_item(document, true))
        .collect::<Vec<_>>();
    let missing_documents = kyc_required_document_checklist(role, &documents)
        .into_iter()
        .filter(|item| item.is_required && !item.is_satisfied)
        .map(|item| item.label)
        .collect::<Vec<_>>();
    if !missing_documents.is_empty() {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message: format!(
                "Upload required onboarding documents before submitting: {}.",
                missing_documents.join(", ")
            ),
        }));
    }

    let role_key_value = role.map(role_key);
    match legal_acceptance_summary(
        pool,
        resolved.user.id,
        resolved.user.organization_id,
        role_key_value,
    )
    .await
    {
        Ok((missing, _)) if !missing.is_empty() => {
            let names = missing
                .iter()
                .map(|agreement| agreement.title.clone())
                .collect::<Vec<_>>()
                .join(", ");
            return Json(ApiResponse::ok(SubmitOnboardingResponse {
                success: false,
                session: Some(resolved.session),
                next_step: "/auth/legal-agreements".into(),
                message: format!(
                    "Accept required legal agreements before submitting onboarding: {}.",
                    names
                ),
            }));
        }
        Ok(_) => {}
        Err(error) => {
            return Json(ApiResponse::ok(SubmitOnboardingResponse {
                success: false,
                session: Some(resolved.session),
                next_step: "/auth/legal-agreements".into(),
                message: format!("Legal agreement check failed: {}", error),
            }));
        }
    }

    let input = UpsertUserOnboardingInput {
        user_id: resolved.user.id,
        company_name: payload.company_name.trim().to_string(),
        company_address: payload.company_address.trim().to_string(),
        dot_number: optional_owned(&payload.dot_number),
        mc_number: optional_owned(&payload.mc_number),
        equipment_types: optional_owned(&payload.equipment_types),
        business_entity_id: optional_owned(&payload.business_entity_id),
        facility_address: optional_owned(&payload.facility_address),
        fulfillment_contact_info: optional_owned(&payload.fulfillment_contact_info),
        fmcsa_broker_license_no: optional_owned(&payload.fmcsa_broker_license_no),
        mc_authority_number: optional_owned(&payload.mc_authority_number),
        freight_forwarder_license: optional_owned(&payload.freight_forwarder_license),
        customs_license: optional_owned(&payload.customs_license),
        next_status: AccountStatus::PendingReview.legacy_code(),
    };

    match upsert_user_onboarding_details(pool, &input).await {
        Ok(()) => {
            let refreshed_user = find_user_by_id(pool, resolved.user.id)
                .await
                .ok()
                .flatten()
                .unwrap_or(resolved.user);
            let session = auth_session::build_session_state(&state, &refreshed_user).await;
            Json(ApiResponse::ok(SubmitOnboardingResponse {
                success: true,
                session: Some(session),
                next_step: "/".into(),
                message: "Rust onboarding submitted. The account is now pending admin review."
                    .into(),
            }))
        }
        Err(error) => Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message: format!("Onboarding save failed: {}", error),
        })),
    }
}

async fn profile_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<SelfProfileScreen>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(unauthenticated_profile_screen()));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(SelfProfileScreen {
            title: "My Profile".into(),
            role_key: resolved
                .user
                .primary_role()
                .map(role_key)
                .unwrap_or("unknown")
                .into(),
            role_label: resolved
                .user
                .primary_role()
                .map(|role| role.label().to_string())
                .unwrap_or_else(|| "Unknown".into()),
            status_label: resolved
                .user
                .account_status()
                .map(account_status_display)
                .unwrap_or_else(|| "Unknown".into()),
            draft: self_profile_draft_from_user(&resolved.user, None),
            personal_facts: self_profile_personal_facts(&resolved.user),
            company_facts: self_profile_company_facts(&resolved.user, None),
            carrier_capacity: None,
            documents: Vec::new(),
            required_documents: kyc_required_document_checklist(
                resolved.user.primary_role(),
                &Vec::new(),
            ),
            notes: vec![format!(
                "Profile data is read-only right now because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        }));
    };

    let details = find_user_detail_by_user_id(pool, resolved.user.id)
        .await
        .ok()
        .flatten();
    let documents = list_kyc_documents_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|document| build_kyc_document_item(document, true))
        .collect::<Vec<_>>();
    let carrier_capacity = if resolved.user.primary_role() == Some(UserRole::Carrier) {
        carrier_capacity_profile(pool, resolved.user.id)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    Json(ApiResponse::ok(profile_screen_from_user(
        &resolved.user,
        details,
        carrier_capacity,
        documents,
    )))
}

async fn notification_center_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<NotificationCenterScreen>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(NotificationCenterScreen {
            unread_count: 0,
            notifications: Vec::new(),
            preferences: Vec::new(),
            provider_decisions: Vec::new(),
            coverage_rules: Vec::new(),
        }));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(NotificationCenterScreen {
            unread_count: 0,
            notifications: Vec::new(),
            preferences: Vec::new(),
            provider_decisions: Vec::new(),
            coverage_rules: Vec::new(),
        }));
    };

    let user_id = resolved.user.id;
    let organization_id = auth_session::session_organization_id(&resolved);
    let notifications = sqlx::query(
        "SELECT id, event_key, category, priority, subject, body, entity_type, entity_id,
                action_href, channels, delivery_status, read_at, created_at
         FROM notification_events
         WHERE recipient_user_id = $1
         ORDER BY read_at NULLS FIRST, created_at DESC
         LIMIT 80",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| NotificationEventRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        event_key: row.get("event_key"),
        category: row.get("category"),
        priority: row.get("priority"),
        subject: row.get("subject"),
        body: row.get("body"),
        entity_type: row.get("entity_type"),
        entity_id: row
            .get::<Option<i64>, _>("entity_id")
            .map(|value| value.max(0) as u64),
        action_href: row.get("action_href"),
        channels: row.get::<Vec<String>, _>("channels"),
        delivery_status: row.get("delivery_status"),
        read_at: row
            .get::<Option<chrono::NaiveDateTime>, _>("read_at")
            .map(|value| value.to_string()),
        created_at: row
            .get::<chrono::NaiveDateTime, _>("created_at")
            .to_string(),
    })
    .collect::<Vec<_>>();

    let unread_count = notifications
        .iter()
        .filter(|notification| notification.read_at.is_none())
        .count() as u64;

    let preferences = sqlx::query(
        "SELECT id, event_key, email_enabled, in_app_enabled, sms_enabled, push_enabled,
                quiet_hours_start::TEXT AS quiet_hours_start,
                quiet_hours_end::TEXT AS quiet_hours_end,
                timezone, escalation_minutes
         FROM notification_preferences
         WHERE (organization_id IS NULL OR organization_id = $1)
           AND (user_id IS NULL OR user_id = $2)
         ORDER BY user_id NULLS LAST, event_key ASC",
    )
    .bind(organization_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| NotificationPreferenceRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        event_key: row.get("event_key"),
        email_enabled: row.get("email_enabled"),
        in_app_enabled: row.get("in_app_enabled"),
        sms_enabled: row.get("sms_enabled"),
        push_enabled: row.get("push_enabled"),
        quiet_hours_start: row.get("quiet_hours_start"),
        quiet_hours_end: row.get("quiet_hours_end"),
        timezone: row.get("timezone"),
        escalation_minutes: row.get("escalation_minutes"),
    })
    .collect();

    let provider_decisions = sqlx::query(
        "SELECT channel, provider_name, decision_status, opt_in_required, opt_out_required,
                quiet_hours_required, emergency_exception_allowed,
                provider_audit_logs_required, compliance_notes
         FROM notification_provider_decisions
         ORDER BY channel",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| NotificationProviderDecisionRow {
        channel: row.get("channel"),
        provider_name: row.get("provider_name"),
        decision_status: row.get("decision_status"),
        opt_in_required: row.get("opt_in_required"),
        opt_out_required: row.get("opt_out_required"),
        quiet_hours_required: row.get("quiet_hours_required"),
        emergency_exception_allowed: row.get("emergency_exception_allowed"),
        provider_audit_logs_required: row.get("provider_audit_logs_required"),
        compliance_notes: row.get("compliance_notes"),
    })
    .collect();

    let coverage_rules = sqlx::query(
        "SELECT event_key, category, default_priority, default_channels, responsible_party,
                entity_type, escalation_minutes, active, notes
         FROM notification_coverage_rules
         ORDER BY category, event_key",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| NotificationCoverageRuleRow {
        event_key: row.get("event_key"),
        category: row.get("category"),
        default_priority: row.get("default_priority"),
        default_channels: row.get::<Vec<String>, _>("default_channels"),
        responsible_party: row.get("responsible_party"),
        entity_type: row.get("entity_type"),
        escalation_minutes: row.get("escalation_minutes"),
        active: row.get("active"),
        notes: row.get("notes"),
    })
    .collect();

    Json(ApiResponse::ok(NotificationCenterScreen {
        unread_count,
        notifications,
        preferences,
        provider_decisions,
        coverage_rules,
    }))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MarkNotificationReadRequest>,
) -> Json<ApiResponse<NotificationMutationResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Sign in before updating notifications.".into(),
        }));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Notification center requires the database connection.".into(),
        }));
    };

    let result = if payload.mark_all.unwrap_or(false) {
        sqlx::query(
            "UPDATE notification_events
             SET read_at = COALESCE(read_at, CURRENT_TIMESTAMP), updated_at = CURRENT_TIMESTAMP
             WHERE recipient_user_id = $1 AND read_at IS NULL",
        )
        .bind(resolved.user.id)
        .execute(pool)
        .await
    } else if let Some(notification_id) = payload.notification_id {
        sqlx::query(
            "UPDATE notification_events
             SET read_at = COALESCE(read_at, CURRENT_TIMESTAMP), updated_at = CURRENT_TIMESTAMP
             WHERE id = $1 AND recipient_user_id = $2",
        )
        .bind(notification_id as i64)
        .bind(resolved.user.id)
        .execute(pool)
        .await
    } else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Choose a notification or mark all.".into(),
        }));
    };

    Json(ApiResponse::ok(NotificationMutationResponse {
        success: result.is_ok(),
        message: result
            .map(|result| format!("{} notification(s) marked read.", result.rows_affected()))
            .unwrap_or_else(|error| format!("Notification update failed: {}", error)),
    }))
}

async fn upsert_notification_preference(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertNotificationPreferenceRequest>,
) -> Json<ApiResponse<NotificationMutationResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Sign in before saving notification preferences.".into(),
        }));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Notification preferences require the database connection.".into(),
        }));
    };

    let event_key = payload.event_key.trim();
    if event_key.is_empty() {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Choose a notification event key before saving preferences.".into(),
        }));
    }

    let timezone = payload
        .timezone
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("UTC");
    let result = sqlx::query(
        "INSERT INTO notification_preferences (
             organization_id, user_id, event_key, email_enabled, in_app_enabled,
             sms_enabled, push_enabled, quiet_hours_start, quiet_hours_end,
             timezone, escalation_minutes, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8::TIME, $9::TIME, $10, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (COALESCE(organization_id, 0), COALESCE(user_id, 0), event_key)
         DO UPDATE SET
             email_enabled = EXCLUDED.email_enabled,
             in_app_enabled = EXCLUDED.in_app_enabled,
             sms_enabled = EXCLUDED.sms_enabled,
             push_enabled = EXCLUDED.push_enabled,
             quiet_hours_start = EXCLUDED.quiet_hours_start,
             quiet_hours_end = EXCLUDED.quiet_hours_end,
             timezone = EXCLUDED.timezone,
             escalation_minutes = EXCLUDED.escalation_minutes,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(auth_session::session_organization_id(&resolved))
    .bind(resolved.user.id)
    .bind(event_key)
    .bind(payload.email_enabled)
    .bind(payload.in_app_enabled)
    .bind(payload.sms_enabled)
    .bind(payload.push_enabled)
    .bind(payload.quiet_hours_start.as_deref().filter(|value| !value.trim().is_empty()))
    .bind(payload.quiet_hours_end.as_deref().filter(|value| !value.trim().is_empty()))
    .bind(timezone)
    .bind(payload.escalation_minutes)
    .execute(pool)
    .await;

    Json(ApiResponse::ok(NotificationMutationResponse {
        success: result.is_ok(),
        message: result
            .map(|_| "Notification preference saved.".into())
            .unwrap_or_else(|error| format!("Notification preference failed: {}", error)),
    }))
}

async fn communication_governance_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<CommunicationGovernanceScreen>> {
    let empty = || CommunicationGovernanceScreen {
        sender_identities: Vec::new(),
        delivery_events: Vec::new(),
        suppression_entries: Vec::new(),
        template_governance: Vec::new(),
        monitoring_rules: Vec::new(),
        branding_policies: Vec::new(),
        brand_assets: Vec::new(),
        custom_domains: Vec::new(),
        branded_template_rules: Vec::new(),
    };

    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(empty()));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(empty()));
    };
    let organization_id = auth_session::session_organization_id(&resolved);

    let sender_identities = sqlx::query(
        "SELECT id, environment_key, sender_domain, from_email, from_name,
                spf_status, dkim_status, dmarc_status, identity_status, verified_at, notes
         FROM message_sender_identities
         WHERE organization_id IS NULL OR organization_id = $1
         ORDER BY environment_key, organization_id NULLS FIRST, from_email",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| SenderIdentityRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        environment_key: row.get("environment_key"),
        sender_domain: row.get("sender_domain"),
        from_email: row.get("from_email"),
        from_name: row.get("from_name"),
        spf_status: row.get("spf_status"),
        dkim_status: row.get("dkim_status"),
        dmarc_status: row.get("dmarc_status"),
        identity_status: row.get("identity_status"),
        verified_at: row
            .get::<Option<chrono::NaiveDateTime>, _>("verified_at")
            .map(|value| value.to_string()),
        notes: row.get("notes"),
    })
    .collect();

    let delivery_events = sqlx::query(
        "SELECT id, channel, event_type, provider_message_id, recipient, reason, occurred_at
         FROM message_delivery_events
         WHERE organization_id IS NULL OR organization_id = $1
         ORDER BY occurred_at DESC
         LIMIT 80",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| DeliveryEventRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        channel: row.get("channel"),
        event_type: row.get("event_type"),
        provider_message_id: row.get("provider_message_id"),
        recipient: row.get("recipient"),
        reason: row.get("reason"),
        occurred_at: row
            .get::<chrono::NaiveDateTime, _>("occurred_at")
            .to_string(),
    })
    .collect();

    let suppression_entries = sqlx::query(
        "SELECT id, channel, recipient, suppression_reason, status, expires_at
         FROM message_suppression_entries
         WHERE organization_id IS NULL OR organization_id = $1
         ORDER BY created_at DESC
         LIMIT 80",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| SuppressionEntryRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        channel: row.get("channel"),
        recipient: row.get("recipient"),
        suppression_reason: row.get("suppression_reason"),
        status: row.get("status"),
        expires_at: row
            .get::<Option<chrono::NaiveDateTime>, _>("expires_at")
            .map(|value| value.to_string()),
    })
    .collect();

    let template_governance = sqlx::query(
        "SELECT id, template_key, channel, locale, version, owner_team, approval_status,
                high_risk, test_send_required, notes
         FROM message_template_governance
         ORDER BY high_risk DESC, template_key, channel, version DESC",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| MessageTemplateGovernanceRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        template_key: row.get("template_key"),
        channel: row.get("channel"),
        locale: row.get("locale"),
        version: row.get("version"),
        owner_team: row.get("owner_team"),
        approval_status: row.get("approval_status"),
        high_risk: row.get("high_risk"),
        test_send_required: row.get("test_send_required"),
        notes: row.get("notes"),
    })
    .collect();

    let monitoring_rules = sqlx::query(
        "SELECT rule_key, event_key, template_key, category, priority, required_sender_identity,
                fallback_channel, escalation_minutes, active, notes
         FROM message_monitoring_rules
         ORDER BY priority DESC, category, rule_key",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| MessageMonitoringRuleRow {
        rule_key: row.get("rule_key"),
        event_key: row.get("event_key"),
        template_key: row.get("template_key"),
        category: row.get("category"),
        priority: row.get("priority"),
        required_sender_identity: row.get("required_sender_identity"),
        fallback_channel: row.get("fallback_channel"),
        escalation_minutes: row.get("escalation_minutes"),
        active: row.get("active"),
        notes: row.get("notes"),
    })
    .collect();

    let branding_policies = sqlx::query(
        "SELECT organization_id, portal_branding_enabled, document_branding_enabled,
                email_branding_enabled, custom_domain_enabled, white_label_status,
                unsupported_message, fallback_brand_name, cache_version
         FROM tenant_branding_policies
         WHERE organization_id = $1
         ORDER BY updated_at DESC",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| TenantBrandingPolicyRow {
        organization_id: row.get::<i64, _>("organization_id").max(0) as u64,
        portal_branding_enabled: row.get("portal_branding_enabled"),
        document_branding_enabled: row.get("document_branding_enabled"),
        email_branding_enabled: row.get("email_branding_enabled"),
        custom_domain_enabled: row.get("custom_domain_enabled"),
        white_label_status: row.get("white_label_status"),
        unsupported_message: row.get("unsupported_message"),
        fallback_brand_name: row.get("fallback_brand_name"),
        cache_version: row.get("cache_version"),
    })
    .collect();

    let brand_assets = sqlx::query(
        "SELECT id, asset_type, asset_url, mime_type, file_size_bytes, review_status, cache_key, notes
         FROM tenant_brand_assets
         WHERE organization_id = $1
         ORDER BY review_status, asset_type, updated_at DESC",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| TenantBrandAssetRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        asset_type: row.get("asset_type"),
        asset_url: row.get("asset_url"),
        mime_type: row.get("mime_type"),
        file_size_bytes: row.get("file_size_bytes"),
        review_status: row.get("review_status"),
        cache_key: row.get("cache_key"),
        notes: row.get("notes"),
    })
    .collect();

    let custom_domains = sqlx::query(
        "SELECT id, domain, purpose, verification_status, dns_txt_name, dns_txt_value,
                tls_status, rollback_status, last_checked_at, notes
         FROM tenant_custom_domains
         WHERE organization_id = $1
         ORDER BY verification_status, purpose, domain",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| TenantCustomDomainRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        domain: row.get("domain"),
        purpose: row.get("purpose"),
        verification_status: row.get("verification_status"),
        dns_txt_name: row.get("dns_txt_name"),
        dns_txt_value: row.get("dns_txt_value"),
        tls_status: row.get("tls_status"),
        rollback_status: row.get("rollback_status"),
        last_checked_at: row
            .get::<Option<chrono::NaiveDateTime>, _>("last_checked_at")
            .map(|value| value.to_string()),
        notes: row.get("notes"),
    })
    .collect();

    let branded_template_rules = sqlx::query(
        "SELECT id, template_key, template_surface, branding_status, fallback_allowed, notes
         FROM tenant_branded_template_rules
         WHERE organization_id = $1
         ORDER BY template_surface, template_key",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| TenantBrandedTemplateRuleRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        template_key: row.get("template_key"),
        template_surface: row.get("template_surface"),
        branding_status: row.get("branding_status"),
        fallback_allowed: row.get("fallback_allowed"),
        notes: row.get("notes"),
    })
    .collect();

    Json(ApiResponse::ok(CommunicationGovernanceScreen {
        sender_identities,
        delivery_events,
        suppression_entries,
        template_governance,
        monitoring_rules,
        branding_policies,
        brand_assets,
        custom_domains,
        branded_template_rules,
    }))
}

async fn record_message_template_test_send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MessageTemplateTestSendRequest>,
) -> Json<ApiResponse<NotificationMutationResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Sign in before recording a template test-send.".into(),
        }));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Template test-send requires the database connection.".into(),
        }));
    };

    let template_key = payload.template_key.trim();
    let channel = payload.channel.trim();
    let recipient = payload.recipient.trim();
    if template_key.is_empty() || channel.is_empty() || recipient.is_empty() {
        return Json(ApiResponse::ok(NotificationMutationResponse {
            success: false,
            message: "Template key, channel, and recipient are required.".into(),
        }));
    }

    let result = sqlx::query(
        "INSERT INTO message_delivery_events (
             organization_id, channel, event_type, recipient, reason, metadata
         )
         VALUES ($1, $2, 'test_send', $3, $4, jsonb_build_object('template_key', $5, 'requested_by_user_id', $6))",
    )
    .bind(auth_session::session_organization_id(&resolved))
    .bind(channel)
    .bind(recipient)
    .bind("Template governance test-send recorded from the authenticated portal.")
    .bind(template_key)
    .bind(resolved.user.id)
    .execute(pool)
    .await;

    Json(ApiResponse::ok(NotificationMutationResponse {
        success: result.is_ok(),
        message: result
            .map(|_| "Template test-send recorded for deliverability review.".into())
            .unwrap_or_else(|error| format!("Template test-send failed: {}", error)),
    }))
}

async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateSelfProfileRequest>,
) -> Json<ApiResponse<UpdateSelfProfileResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Sign in before editing the Rust profile.".into(),
            session: None,
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: format!(
                "Profile updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            session: Some(resolved.session),
        }));
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Name and email are required before saving the Rust profile.".into(),
            session: Some(resolved.session),
        }));
    }

    if let Some(existing_user) = find_user_by_email(pool, payload.email.trim())
        .await
        .unwrap_or(None)
        && existing_user.id != resolved.user.id
    {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Another account already uses that email address.".into(),
            session: Some(resolved.session),
        }));
    }

    let password_hash = match (
        optional_owned(&payload.password),
        optional_owned(&payload.password_confirmation),
    ) {
        (Some(password), Some(password_confirmation)) => {
            if password != password_confirmation {
                return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                    success: false,
                    message: "Password confirmation does not match.".into(),
                    session: Some(resolved.session),
                }));
            }
            if password.len() < 8 {
                return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                    success: false,
                    message: "Use a password with at least 8 characters.".into(),
                    session: Some(resolved.session),
                }));
            }

            match hash(&password, 12) {
                Ok(value) => Some(value),
                Err(error) => {
                    return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                        success: false,
                        message: format!("Password hashing failed: {}", error),
                        session: Some(resolved.session),
                    }));
                }
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                success: false,
                message: "Enter both password fields before saving a new password.".into(),
                session: Some(resolved.session),
            }));
        }
        (None, None) => None,
    };

    let role = resolved.user.primary_role();
    if matches!(role, Some(UserRole::Carrier))
        && (optional_owned(&payload.dot_number).is_none()
            || optional_owned(&payload.mc_number).is_none())
    {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Carrier profiles need DOT and MC numbers in the Rust edit flow.".into(),
            session: Some(resolved.session),
        }));
    }

    let input = UpdateSelfProfileInput {
        user_id: resolved.user.id,
        name: payload.name.trim().to_string(),
        email: payload.email.trim().to_lowercase(),
        phone_no: optional_owned(&payload.phone_no),
        address: optional_owned(&payload.address),
        company_name: optional_owned(&payload.company_name),
        dot_number: optional_owned(&payload.dot_number),
        mc_number: optional_owned(&payload.mc_number),
        mc_cbsa_usdot_no: optional_owned(&payload.mc_cbsa_usdot_no),
        ucr_hcc_no: optional_owned(&payload.ucr_hcc_no),
        password_hash,
        status: resolved.user.status,
    };

    match update_self_profile(pool, &input).await {
        Ok(Some(user)) => {
            let session = auth_session::build_session_state(&state, &user).await;
            Json(ApiResponse::ok(UpdateSelfProfileResponse {
                success: true,
                message: "Rust profile updated successfully.".into(),
                session: Some(session),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "The selected Rust profile could not be updated.".into(),
            session: Some(resolved.session),
        })),
        Err(error) => Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: format!("Profile update failed: {}", error),
            session: Some(resolved.session),
        })),
    }
}

async fn update_carrier_capacity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateCarrierCapacityRequest>,
) -> Json<ApiResponse<UpdateCarrierCapacityResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: false,
            message: "Sign in before editing carrier capacity.".into(),
            capacity: None,
        }));
    };

    if resolved.user.primary_role() != Some(UserRole::Carrier) {
        return Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: false,
            message: "Only carrier accounts can maintain carrier capacity.".into(),
            capacity: None,
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: false,
            message: format!(
                "Carrier capacity updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            capacity: None,
        }));
    };

    let availability_status = normalize_capacity_value(&payload.availability_status)
        .unwrap_or_else(|| "available".into());
    if !matches!(
        availability_status.as_str(),
        "available" | "limited" | "unavailable" | "seasonal" | "paused"
    ) {
        return Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: false,
            message: "Availability must be available, limited, unavailable, seasonal, or paused."
                .into(),
            capacity: None,
        }));
    }

    let equipment_types = normalize_capacity_list(&payload.equipment_types);
    let lane_preferences = normalize_capacity_list(&payload.lane_preferences);
    let operating_regions = normalize_capacity_list(&payload.operating_regions);
    let preferred_commodities = normalize_capacity_list(&payload.preferred_commodities);
    let service_levels = normalize_capacity_list(&payload.service_levels);
    let certifications = normalize_capacity_list(&payload.certifications);
    let capacity_notes = optional_owned(&payload.capacity_notes);
    if equipment_types.is_empty() || operating_regions.is_empty() {
        return Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: false,
            message: "Carrier capacity requires at least one equipment type and operating region."
                .into(),
            capacity: None,
        }));
    }

    let result = sqlx::query(
        "INSERT INTO carrier_capacity_profiles (
            organization_id, carrier_user_id, equipment_types, lane_preferences,
            operating_regions, preferred_commodities, service_levels, certifications,
            availability_status, available_power_units, insurance_limit_usd,
            capacity_notes, last_updated_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (carrier_user_id)
         DO UPDATE SET
            organization_id = EXCLUDED.organization_id,
            equipment_types = EXCLUDED.equipment_types,
            lane_preferences = EXCLUDED.lane_preferences,
            operating_regions = EXCLUDED.operating_regions,
            preferred_commodities = EXCLUDED.preferred_commodities,
            service_levels = EXCLUDED.service_levels,
            certifications = EXCLUDED.certifications,
            availability_status = EXCLUDED.availability_status,
            available_power_units = EXCLUDED.available_power_units,
            insurance_limit_usd = EXCLUDED.insurance_limit_usd,
            capacity_notes = EXCLUDED.capacity_notes,
            last_updated_by_user_id = EXCLUDED.last_updated_by_user_id,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(resolved.user.organization_id)
    .bind(resolved.user.id)
    .bind(&equipment_types)
    .bind(&lane_preferences)
    .bind(&operating_regions)
    .bind(&preferred_commodities)
    .bind(&service_levels)
    .bind(&certifications)
    .bind(&availability_status)
    .bind(payload.available_power_units as i32)
    .bind(payload.insurance_limit_usd.max(0.0))
    .bind(capacity_notes)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: true,
            message: "Carrier capacity profile saved.".into(),
            capacity: carrier_capacity_profile(pool, resolved.user.id)
                .await
                .ok()
                .flatten(),
        })),
        Err(error) => Json(ApiResponse::ok(UpdateCarrierCapacityResponse {
            success: false,
            message: format!("Carrier capacity save failed: {}", error),
            capacity: None,
        })),
    }
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Json<ApiResponse<ChangePasswordResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Sign in before changing your Rust password.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: format!(
                "Password changes are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    if payload.current_password.is_empty() {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Enter your current password before saving a new one.".into(),
        }));
    }

    if payload.password != payload.password_confirmation {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Password confirmation does not match.".into(),
        }));
    }

    if payload.password.len() < 8 {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Use a password with at least 8 characters.".into(),
        }));
    }

    let current_password_matches =
        verify_user_password(&payload.current_password, &resolved.user.password);

    if !current_password_matches {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Current password is incorrect.".into(),
        }));
    }

    let password_hash = match hash(&payload.password, 12) {
        Ok(value) => value,
        Err(error) => {
            return Json(ApiResponse::ok(ChangePasswordResponse {
                success: false,
                message: format!("Password hashing failed: {}", error),
            }));
        }
    };

    match change_self_password(
        pool,
        &ChangeSelfPasswordInput {
            user_id: resolved.user.id,
            password_hash,
            status: resolved.user.status,
            remarks: Some("Password changed through the Rust account security page.".into()),
        },
    )
    .await
    {
        Ok(Some(_)) => Json(ApiResponse::ok(ChangePasswordResponse {
            success: true,
            message: "Rust password updated successfully.".into(),
        })),
        Ok(None) => Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "The current account could not be updated.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: format!("Password update failed: {}", error),
        })),
    }
}

fn parse_role_key(value: &str) -> Option<UserRole> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "admin" => Some(UserRole::Admin),
        "shipper" => Some(UserRole::Shipper),
        "carrier" => Some(UserRole::Carrier),
        "broker" => Some(UserRole::Broker),
        "freight_forwarder" | "freight forwarder" => Some(UserRole::FreightForwarder),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct OidcIdTokenClaims {
    sub: String,
    iss: String,
    aud: OidcAudience,
    #[serde(rename = "exp")]
    _exp: usize,
    email: String,
    email_verified: Option<bool>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OidcAudience {
    One(String),
    Many(Vec<String>),
}

impl OidcAudience {
    fn contains(&self, expected: &str) -> bool {
        match self {
            Self::One(value) => value == expected,
            Self::Many(values) => values.iter().any(|value| value == expected),
        }
    }
}

async fn validate_oidc_id_token(
    jwks_url: &str,
    issuer: &str,
    client_id: &str,
    id_token: &str,
) -> Result<TokenData<OidcIdTokenClaims>, String> {
    let header =
        decode_header(id_token).map_err(|error| format!("invalid OIDC header: {error}"))?;
    let kid = header
        .kid
        .ok_or_else(|| "OIDC token missing kid".to_string())?;
    let jwks = reqwest::get(jwks_url)
        .await
        .map_err(|error| format!("failed to fetch JWKS: {error}"))?
        .json::<JwkSet>()
        .await
        .map_err(|error| format!("failed to parse JWKS: {error}"))?;
    let jwk = jwks
        .find(&kid)
        .ok_or_else(|| "matching JWK not found".to_string())?;
    let decoding_key = DecodingKey::from_jwk(jwk)
        .map_err(|error| format!("failed to build decoding key: {error}"))?;
    let mut validation = Validation::new(header.alg);
    if !matches!(
        header.alg,
        Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384
    ) {
        return Err("unsupported OIDC signing algorithm".into());
    }
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[client_id]);
    let token_data = decode::<OidcIdTokenClaims>(id_token, &decoding_key, &validation)
        .map_err(|error| format!("OIDC token validation failed: {error}"))?;
    if token_data.claims.iss != issuer || !token_data.claims.aud.contains(client_id) {
        return Err("OIDC issuer or audience mismatch".into());
    }
    Ok(token_data)
}

async fn enterprise_sso_discovery_response(
    state: &AppState,
    email: &str,
) -> EnterpriseSsoDiscoveryResponse {
    let email = email.trim().to_lowercase();
    let email_domain = email
        .split('@')
        .nth(1)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let Some(pool) = state.pool.as_ref() else {
        return EnterpriseSsoDiscoveryResponse {
            email_domain,
            organization_id: None,
            organization_name: None,
            sso_required: false,
            password_allowed: true,
            provider_type: None,
            provider_display_name: None,
            sso_url: None,
            jit_enabled: false,
            default_role_key: None,
            message: "Enterprise SSO discovery is unavailable because the database is offline."
                .into(),
        };
    };

    match discover_sso_for_email(pool, &email).await {
        Ok(Some(record)) => {
            let active_provider = record.provider_status.as_deref() == Some("active");
            let sso_required =
                record.domain_verified && record.login_routing_enabled && active_provider;
            EnterpriseSsoDiscoveryResponse {
                email_domain: Some(record.domain),
                organization_id: Some(record.organization_id.max(0) as u64),
                organization_name: Some(record.organization_name),
                sso_required,
                password_allowed: !sso_required,
                provider_type: record.provider_type,
                provider_display_name: record.provider_display_name,
                sso_url: record.sso_url,
                jit_enabled: record.jit_enabled,
                default_role_key: record.default_role_key,
                message: if sso_required {
                    "This organization uses enterprise SSO; continue through the identity provider."
                        .into()
                } else if record.domain_verified {
                    "This domain is verified, but password login remains allowed until SSO routing is active."
                        .into()
                } else {
                    "This domain is registered but not yet verified for enterprise SSO.".into()
                },
            }
        }
        Ok(None) => EnterpriseSsoDiscoveryResponse {
            email_domain,
            organization_id: None,
            organization_name: None,
            sso_required: false,
            password_allowed: true,
            provider_type: None,
            provider_display_name: None,
            sso_url: None,
            jit_enabled: false,
            default_role_key: None,
            message: "No enterprise SSO routing matched this email domain.".into(),
        },
        Err(error) => EnterpriseSsoDiscoveryResponse {
            email_domain,
            organization_id: None,
            organization_name: None,
            sso_required: false,
            password_allowed: true,
            provider_type: None,
            provider_display_name: None,
            sso_url: None,
            jit_enabled: false,
            default_role_key: None,
            message: format!("Enterprise SSO discovery failed: {}", error),
        },
    }
}

async fn authorize_scim_token(
    pool: &db::DbPool,
    headers: &HeaderMap,
) -> Result<ScimTokenRecord, StatusCode> {
    let token = auth_session::bearer_token_from_headers(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let parsed = parse_scim_token(&token).ok_or(StatusCode::UNAUTHORIZED)?;
    let token_hash = hash_scim_secret(parsed.1);
    find_scim_token_by_hash(pool, parsed.0, &token_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

fn parse_scim_token(token: &str) -> Option<(&str, &str)> {
    let stripped = token.strip_prefix("scim_")?;
    let (prefix, secret) = stripped.split_once('.')?;
    if prefix.is_empty() || secret.is_empty() {
        return None;
    }
    Some((prefix, secret))
}

fn hash_scim_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn header_value<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

async fn legal_agreement_screen_for_user(
    pool: &db::DbPool,
    user: &db::auth::UserRecord,
) -> Result<LegalAgreementScreen, sqlx::Error> {
    let role_key_value = user.primary_role().map(role_key);
    let (missing, proofs) =
        legal_acceptance_summary(pool, user.id, user.organization_id, role_key_value).await?;
    Ok(LegalAgreementScreen {
        title: "Legal Agreements".into(),
        missing_required: missing
            .into_iter()
            .map(legal_template_item)
            .collect::<Vec<_>>(),
        acceptance_proofs: proofs
            .into_iter()
            .map(legal_acceptance_item)
            .collect::<Vec<_>>(),
        notes: vec![
            "Required agreements are versioned and acceptance is stored with signer, timestamp, IP, user agent, evidence snapshot, and audit event.".into(),
            "Updated agreement versions automatically appear as missing until accepted.".into(),
        ],
    })
}

fn legal_template_item(template: LegalAgreementTemplateRecord) -> LegalAgreementTemplateItem {
    LegalAgreementTemplateItem {
        id: template.id.max(0) as u64,
        agreement_key: template.agreement_key,
        version: template.version,
        title: template.title,
        document_uri: template.document_uri,
        required_role_key: template.required_role_key,
        requires_user_acceptance: template.requires_user_acceptance,
        requires_organization_acceptance: template.requires_organization_acceptance,
        effective_at: template.effective_at.to_string(),
    }
}

fn legal_acceptance_item(
    acceptance: LegalAgreementAcceptanceRecord,
) -> LegalAgreementAcceptanceItem {
    LegalAgreementAcceptanceItem {
        id: acceptance.id.max(0) as u64,
        agreement_key: acceptance.agreement_key,
        version: acceptance.version,
        signer_name: acceptance.signer_name,
        signer_email: acceptance.signer_email,
        accepted_at: acceptance.accepted_at.to_string(),
        audit_event_id: acceptance.audit_event_id.map(|value| value.max(0) as u64),
    }
}

fn role_key(role: UserRole) -> &'static str {
    match role {
        UserRole::Admin => "admin",
        UserRole::Shipper => "shipper",
        UserRole::Carrier => "carrier",
        UserRole::Broker => "broker",
        UserRole::FreightForwarder => "freight_forwarder",
    }
}

fn organization_role_key_for_user_role(role: UserRole) -> &'static str {
    match role {
        UserRole::Admin => "admin",
        _ => "member",
    }
}

fn role_from_organization_role_key(role_key: Option<&str>) -> UserRole {
    match role_key
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "admin" | "owner" => UserRole::Admin,
        _ => UserRole::Carrier,
    }
}

fn unauthenticated_profile_screen() -> SelfProfileScreen {
    SelfProfileScreen {
        title: "My Profile".into(),
        role_key: "unknown".into(),
        role_label: "Unknown".into(),
        status_label: "Signed out".into(),
        draft: SelfProfileDraft {
            name: String::new(),
            email: String::new(),
            phone_no: None,
            address: None,
            company_name: None,
            dot_number: None,
            mc_number: None,
            mc_cbsa_usdot_no: None,
            ucr_hcc_no: None,
        },
        personal_facts: Vec::new(),
        company_facts: Vec::new(),
        carrier_capacity: None,
        documents: Vec::new(),
        required_documents: Vec::new(),
        notes: vec!["Sign in first so the Rust profile can load your account.".into()],
    }
}

fn profile_screen_from_user(
    user: &db::auth::UserRecord,
    details: Option<db::auth::UserDetailRecord>,
    carrier_capacity: Option<CarrierCapacityProfile>,
    documents: Vec<KycDocumentItem>,
) -> SelfProfileScreen {
    let role = user.primary_role();
    let role_key_value = role.map(role_key).unwrap_or("unknown").to_string();
    let required_documents = kyc_required_document_checklist(role, &documents);

    SelfProfileScreen {
        title: "My Profile".into(),
        role_key: role_key_value,
        role_label: role
            .map(|value| value.label().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        status_label: user
            .account_status()
            .map(account_status_display)
            .unwrap_or_else(|| "Unknown".into()),
        draft: self_profile_draft_from_user(user, details.as_ref()),
        personal_facts: self_profile_personal_facts(user),
        company_facts: self_profile_company_facts(user, details.as_ref()),
        carrier_capacity,
        documents,
        required_documents,
        notes: vec![
            "This Rust profile replaces the old read-only and edit profile Blade pages with one self-serve workspace.".into(),
            "Admins can still inspect the deeper compliance view from the Rust admin directory when needed.".into(),
        ],
    }
}

fn self_profile_draft_from_user(
    user: &db::auth::UserRecord,
    details: Option<&db::auth::UserDetailRecord>,
) -> shared::SelfProfileDraft {
    shared::SelfProfileDraft {
        name: user.name.clone(),
        email: user.email.clone(),
        phone_no: user.phone_no.clone(),
        address: user.address.clone(),
        company_name: details
            .and_then(|item| item.company_name.clone())
            .or_else(|| user.company_name.clone()),
        dot_number: details.and_then(|item| item.dot_number.clone()),
        mc_number: details
            .and_then(|item| item.mc_number.clone())
            .or_else(|| user.mc_number.clone()),
        mc_cbsa_usdot_no: user.mc_cbsa_usdot_no.clone(),
        ucr_hcc_no: user.ucr_hcc_no.clone(),
    }
}

fn self_profile_personal_facts(user: &db::auth::UserRecord) -> Vec<SelfProfileFact> {
    let mut facts = Vec::new();
    push_self_profile_fact(&mut facts, "Email", Some(user.email.clone()));
    push_self_profile_fact(&mut facts, "Phone", user.phone_no.clone());
    push_self_profile_fact(&mut facts, "Address", user.address.clone());
    push_self_profile_fact(
        &mut facts,
        "Date of birth",
        user.dob.map(|value| value.format("%Y-%m-%d").to_string()),
    );
    push_self_profile_fact(&mut facts, "Gender", user.gender.clone());
    push_self_profile_fact(
        &mut facts,
        "Member since",
        Some(format_profile_datetime(&user.created_at)),
    );
    facts
}

fn self_profile_company_facts(
    user: &db::auth::UserRecord,
    details: Option<&db::auth::UserDetailRecord>,
) -> Vec<SelfProfileFact> {
    let mut facts = Vec::new();
    push_self_profile_fact(
        &mut facts,
        "Company",
        details
            .and_then(|item| item.company_name.clone())
            .or_else(|| user.company_name.clone()),
    );
    push_self_profile_fact(
        &mut facts,
        "DOT number",
        details.and_then(|item| item.dot_number.clone()),
    );
    push_self_profile_fact(
        &mut facts,
        "MC number",
        details
            .and_then(|item| item.mc_number.clone())
            .or_else(|| user.mc_number.clone()),
    );
    push_self_profile_fact(&mut facts, "MC/CBSA/USDOT", user.mc_cbsa_usdot_no.clone());
    push_self_profile_fact(&mut facts, "UCR/HCC", user.ucr_hcc_no.clone());
    facts
}

async fn carrier_capacity_profile(
    pool: &db::DbPool,
    carrier_user_id: i64,
) -> Result<Option<CarrierCapacityProfile>, sqlx::Error> {
    let row = sqlx::query_as::<
        _,
        (
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            Vec<String>,
            String,
            i32,
            f64,
            Option<String>,
        ),
    >(
        "SELECT equipment_types, lane_preferences, operating_regions,
                preferred_commodities, service_levels, certifications,
                availability_status, available_power_units, insurance_limit_usd,
                capacity_notes
         FROM carrier_capacity_profiles
         WHERE carrier_user_id = $1",
    )
    .bind(carrier_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(
            equipment_types,
            lane_preferences,
            operating_regions,
            preferred_commodities,
            service_levels,
            certifications,
            availability_status,
            available_power_units,
            insurance_limit_usd,
            capacity_notes,
        )| {
            let readiness_label = carrier_capacity_readiness(
                &equipment_types,
                &operating_regions,
                &availability_status,
                available_power_units,
                insurance_limit_usd,
            );
            CarrierCapacityProfile {
                equipment_types,
                lane_preferences,
                operating_regions,
                preferred_commodities,
                service_levels,
                certifications,
                availability_status,
                available_power_units: available_power_units.max(0) as u32,
                insurance_limit_usd,
                capacity_notes,
                readiness_label,
            }
        },
    ))
}

fn normalize_capacity_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| normalize_capacity_value(value))
        .fold(Vec::<String>::new(), |mut items, value| {
            if !items.iter().any(|item| item == &value) {
                items.push(value);
            }
            items
        })
}

fn normalize_capacity_value(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    (!normalized.is_empty()
        && normalized.len() <= 96
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'))
    .then_some(normalized)
}

fn carrier_capacity_readiness(
    equipment_types: &[String],
    operating_regions: &[String],
    availability_status: &str,
    available_power_units: i32,
    insurance_limit_usd: f64,
) -> String {
    if availability_status == "unavailable" || availability_status == "paused" {
        return "Not eligible: unavailable".into();
    }
    if equipment_types.is_empty() || operating_regions.is_empty() {
        return "Not eligible: missing equipment or geography".into();
    }
    if available_power_units <= 0 {
        return "Limited: no available power units".into();
    }
    if insurance_limit_usd <= 0.0 {
        return "Limited: insurance limit missing".into();
    }
    "Eligible for capacity matching".into()
}

fn push_self_profile_fact(facts: &mut Vec<SelfProfileFact>, label: &str, value: Option<String>) {
    if let Some(value) = value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
    {
        facts.push(SelfProfileFact {
            label: label.into(),
            value,
        });
    }
}

fn generate_otp() -> String {
    let seed = Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_else(|| Utc::now().timestamp_micros() * 1000)
        .unsigned_abs();
    let value = 100_000 + (seed % 900_000);
    format!("{:06}", value)
}

struct MfaChallenge {
    challenge_id: String,
    code: String,
    expires_at: chrono::DateTime<Utc>,
    message: String,
}

fn privileged_user_requires_mfa(user: &db::auth::UserRecord) -> bool {
    matches!(user.primary_role(), Some(UserRole::Admin))
}

fn mfa_hash(value: &str) -> String {
    let digest = Sha256::digest(value.trim().as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

async fn create_mfa_challenge(
    state: &AppState,
    user: &db::auth::UserRecord,
) -> Result<MfaChallenge, String> {
    let Some(pool) = state.pool.as_ref() else {
        return Err("MFA is unavailable because the database is disabled.".into());
    };

    let challenge_id = uuid::Uuid::new_v4().to_string();
    let code = generate_otp();
    let code_hash = mfa_hash(&code);
    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query(
        "INSERT INTO mfa_challenges
            (id, user_id, email, purpose, code_hash, expires_at, created_at, updated_at)
         VALUES ($1::uuid, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(&challenge_id)
    .bind(user.id)
    .bind(user.email.to_ascii_lowercase())
    .bind("privileged_login")
    .bind(&code_hash)
    .bind(expires_at.naive_utc())
    .execute(pool)
    .await
    .map_err(|error| format!("MFA challenge creation failed: {}", error))?;

    let mail_outcome = state
        .email
        .send_mfa_otp(&user.email, Some(&user.name), &code)
        .await?;

    Ok(MfaChallenge {
        challenge_id,
        code,
        expires_at,
        message: mail_outcome.append_to_message(
            "MFA is required for this privileged account. Enter the emailed code to continue.",
        ),
    })
}

async fn consume_mfa_recovery_code(
    pool: &db::DbPool,
    user_id: i64,
    code: &str,
) -> Result<bool, sqlx::Error> {
    let code_hash = mfa_hash(code);
    sqlx::query_scalar::<_, i64>(
        "UPDATE mfa_recovery_codes
         SET used_at = CURRENT_TIMESTAMP
         WHERE id = (
            SELECT id
            FROM mfa_recovery_codes
            WHERE user_id = $1
              AND code_hash = $2
              AND used_at IS NULL
            ORDER BY id
            LIMIT 1
         )
         RETURNING user_id",
    )
    .bind(user_id)
    .bind(code_hash)
    .fetch_optional(pool)
    .await
    .map(|value| value.is_some())
}

async fn ensure_mfa_recovery_codes(
    pool: &db::DbPool,
    user_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let unused_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM mfa_recovery_codes
         WHERE user_id = $1
           AND used_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if unused_count > 0 {
        return Ok(Vec::new());
    }

    let mut recovery_codes = Vec::new();
    for _ in 0..8 {
        let code = format!(
            "{}-{}",
            uuid::Uuid::new_v4().simple().to_string()[0..8].to_ascii_uppercase(),
            uuid::Uuid::new_v4().simple().to_string()[0..8].to_ascii_uppercase()
        );
        let code_hash = mfa_hash(&code);
        sqlx::query(
            "INSERT INTO mfa_recovery_codes (user_id, code_hash, created_at)
             VALUES ($1, $2, CURRENT_TIMESTAMP)",
        )
        .bind(user_id)
        .bind(code_hash)
        .execute(pool)
        .await?;
        recovery_codes.push(code);
    }

    Ok(recovery_codes)
}

async fn regenerate_mfa_recovery_code_set(
    pool: &db::DbPool,
    user_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query("DELETE FROM mfa_recovery_codes WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    ensure_mfa_recovery_codes(pool, user_id).await
}

fn next_resend_count(
    last_otp_resend_at: Option<chrono::NaiveDateTime>,
    current_count: i32,
) -> Result<i32, String> {
    let now = Utc::now().naive_utc();
    let base_count = match last_otp_resend_at {
        Some(last_sent_at) if now - last_sent_at < Duration::hours(1) => current_count,
        _ => 0,
    };

    if base_count >= 5 {
        Err("OTP resend limit reached. Try again after an hour.".into())
    } else {
        Ok(base_count + 1)
    }
}

fn exposed_secret(state: &AppState, value: &str) -> Option<String> {
    if state.config.environment.eq_ignore_ascii_case("production") {
        None
    } else {
        Some(value.to_string())
    }
}

fn resend_message(purpose: OtpPurpose) -> String {
    match purpose {
        OtpPurpose::Registration => {
            "A fresh registration OTP is ready in the Rust auth flow.".into()
        }
        OtpPurpose::PasswordReset => {
            "A fresh password reset OTP is ready in the Rust auth flow.".into()
        }
    }
}

async fn send_otp_notification(
    state: &AppState,
    purpose: OtpPurpose,
    email: &str,
    name: Option<&str>,
    otp: &str,
) -> Result<MailOutcome, String> {
    match purpose {
        OtpPurpose::Registration => state.email.send_registration_otp(email, name, otp).await,
        OtpPurpose::PasswordReset => state.email.send_password_reset_otp(email, name, otp).await,
    }
}

fn login_message_for_status(status: Option<AccountStatus>) -> String {
    match status {
        Some(AccountStatus::PendingOtp) => {
            "Logged in, but OTP verification is still pending for this account.".into()
        }
        Some(AccountStatus::EmailVerifiedPendingOnboarding) => {
            "Logged in. Email is verified and onboarding is the next Rust migration slice.".into()
        }
        Some(AccountStatus::PendingReview) => {
            "Logged in. This account is waiting for admin review.".into()
        }
        Some(AccountStatus::RevisionRequested) => {
            "Logged in. Admin requested onboarding revisions for this account.".into()
        }
        Some(AccountStatus::Rejected) => {
            "Logged in, but this account is currently rejected in the legacy status model.".into()
        }
        _ => "Logged in through the Rust session layer.".into(),
    }
}

fn unauthenticated_onboarding_screen() -> AuthOnboardingScreen {
    AuthOnboardingScreen {
        title: "Rust Onboarding".into(),
        subtitle: "Continue the account setup after OTP verification.".into(),
        role_key: "unknown".into(),
        role_label: "Unknown".into(),
        status_label: "Signed out".into(),
        can_submit: false,
        requires_otp: true,
        draft: empty_onboarding_draft(),
        documents: Vec::new(),
        required_documents: Vec::new(),
        required_fields: vec!["company_name".into(), "company_address".into()],
        notes: vec!["Sign in first so the Rust onboarding flow can load your account.".into()],
    }
}

fn onboarding_screen_from_user(
    user: &db::auth::UserRecord,
    details: Option<db::auth::UserDetailRecord>,
    documents: Vec<db::auth::KycDocumentRecord>,
    can_submit: bool,
    notes: Vec<String>,
) -> AuthOnboardingScreen {
    let role = user.primary_role();
    let required_fields = required_onboarding_fields(role);
    let draft = AuthOnboardingDraft {
        company_name: details
            .as_ref()
            .and_then(|item| item.company_name.clone())
            .or_else(|| user.company_name.clone()),
        company_address: details
            .as_ref()
            .and_then(|item| item.company_address.clone())
            .or_else(|| user.company_address.clone()),
        dot_number: details.as_ref().and_then(|item| item.dot_number.clone()),
        mc_number: details.as_ref().and_then(|item| item.mc_number.clone()),
        equipment_types: details
            .as_ref()
            .and_then(|item| item.equipment_types.clone()),
        business_entity_id: details
            .as_ref()
            .and_then(|item| item.business_entity_id.clone()),
        facility_address: details
            .as_ref()
            .and_then(|item| item.facility_address.clone()),
        fulfillment_contact_info: details
            .as_ref()
            .and_then(|item| item.fulfillment_contact_info.clone()),
        fmcsa_broker_license_no: details
            .as_ref()
            .and_then(|item| item.fmcsa_broker_license_no.clone()),
        mc_authority_number: details
            .as_ref()
            .and_then(|item| item.mc_authority_number.clone()),
        freight_forwarder_license: details
            .as_ref()
            .and_then(|item| item.freight_forwarder_license.clone()),
        customs_license: details
            .as_ref()
            .and_then(|item| item.customs_license.clone()),
    };

    let document_items = documents
        .into_iter()
        .map(|document| build_kyc_document_item(document, true))
        .collect::<Vec<_>>();
    let required_documents = kyc_required_document_checklist(role, &document_items);

    AuthOnboardingScreen {
        title: "Rust Onboarding".into(),
        subtitle: "Complete the company profile that follows OTP verification before the account enters review.".into(),
        role_key: role.map(role_key).unwrap_or("unknown").into(),
        role_label: role.map(|item| item.label().to_string()).unwrap_or_else(|| "Unknown".into()),
        status_label: user
            .account_status()
            .map(account_status_display)
            .unwrap_or_else(|| "Unknown".into()),
        can_submit,
        requires_otp: user.email_verified_at.is_none(),
        draft,
        documents: document_items,
        required_documents,
        required_fields,
        notes,
    }
}

fn empty_onboarding_draft() -> AuthOnboardingDraft {
    AuthOnboardingDraft {
        company_name: None,
        company_address: None,
        dot_number: None,
        mc_number: None,
        equipment_types: None,
        business_entity_id: None,
        facility_address: None,
        fulfillment_contact_info: None,
        fmcsa_broker_license_no: None,
        mc_authority_number: None,
        freight_forwarder_license: None,
        customs_license: None,
    }
}

fn can_submit_onboarding(status: Option<AccountStatus>) -> bool {
    matches!(
        status,
        Some(AccountStatus::EmailVerifiedPendingOnboarding | AccountStatus::RevisionRequested)
    )
}

fn onboarding_notes_for_user(user: &db::auth::UserRecord) -> Vec<String> {
    let mut notes = Vec::new();
    match user.account_status() {
        Some(AccountStatus::PendingOtp) => notes.push(
            "OTP verification is still pending, so the onboarding form stays locked until that step completes."
                .into(),
        ),
        Some(AccountStatus::EmailVerifiedPendingOnboarding) => notes.push(
            "OTP is complete. Submit this form to move the account into pending review."
                .into(),
        ),
        Some(AccountStatus::RevisionRequested) => notes.push(
            "Admin requested revisions, so this Rust onboarding form is open for another submission cycle."
                .into(),
        ),
        Some(AccountStatus::PendingReview) => notes.push(
            "This account is already pending review, so the form is now read-only until admin feedback arrives."
                .into(),
        ),
        Some(AccountStatus::Approved) => notes.push(
            "This account is already approved, so onboarding values are shown for reference only."
                .into(),
        ),
        Some(AccountStatus::Rejected) => notes.push(
            "This account is currently rejected in the legacy status model.".into(),
        ),
        None => notes.push("The account status could not be mapped from the legacy schema.".into()),
    }
    notes
}

fn required_onboarding_fields(role: Option<UserRole>) -> Vec<String> {
    let mut fields = vec!["company_name".into(), "company_address".into()];
    match role {
        Some(UserRole::Carrier) => {
            fields.push("dot_number".into());
            fields.push("mc_number".into());
            fields.push("equipment_types".into());
        }
        Some(UserRole::Shipper) => {
            fields.push("business_entity_id".into());
            fields.push("facility_address".into());
            fields.push("fulfillment_contact_info".into());
        }
        Some(UserRole::Broker) => {
            fields.push("fmcsa_broker_license_no".into());
            fields.push("mc_authority_number".into());
        }
        Some(UserRole::FreightForwarder) => {
            fields.push("freight_forwarder_license".into());
            fields.push("customs_license".into());
        }
        _ => {}
    }
    fields
}

fn validate_role_specific_onboarding(
    role: Option<UserRole>,
    payload: &SubmitOnboardingRequest,
) -> Option<String> {
    match role {
        Some(UserRole::Carrier)
            if optional_owned(&payload.dot_number).is_none()
                || optional_owned(&payload.mc_number).is_none()
                || optional_owned(&payload.equipment_types).is_none() =>
        {
            Some("Carrier onboarding requires DOT number, MC number, and equipment types.".into())
        }
        Some(UserRole::Shipper)
            if optional_owned(&payload.business_entity_id).is_none()
                || optional_owned(&payload.facility_address).is_none()
                || optional_owned(&payload.fulfillment_contact_info).is_none() =>
        {
            Some("Shipper onboarding requires business entity id, facility address, and fulfillment contact info.".into())
        }
        Some(UserRole::Broker)
            if optional_owned(&payload.fmcsa_broker_license_no).is_none()
                || optional_owned(&payload.mc_authority_number).is_none() =>
        {
            Some("Broker onboarding requires FMCSA broker license number and MC authority number.".into())
        }
        Some(UserRole::FreightForwarder)
            if optional_owned(&payload.freight_forwarder_license).is_none()
                || optional_owned(&payload.customs_license).is_none() =>
        {
            Some("Freight forwarder onboarding requires freight forwarder and customs license values.".into())
        }
        _ => None,
    }
}

fn optional_owned(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn account_status_display(status: AccountStatus) -> String {
    match status {
        AccountStatus::EmailVerifiedPendingOnboarding => "Email Verified".into(),
        AccountStatus::Approved => "Approved".into(),
        AccountStatus::Rejected => "Rejected".into(),
        AccountStatus::PendingReview => "Pending Review".into(),
        AccountStatus::PendingOtp => "Pending OTP".into(),
        AccountStatus::RevisionRequested => "Revision Requested".into(),
    }
}

fn build_kyc_document_item(
    document: db::auth::KycDocumentRecord,
    can_edit: bool,
) -> KycDocumentItem {
    let file_label = document
        .original_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| profile_file_label(&document.file_path));
    let blockchain_label = if document.document_type.eq_ignore_ascii_case("blockchain") {
        Some(
            document
                .mock_blockchain_tx
                .as_ref()
                .map(|_| "SHA-256 hash stored".to_string())
                .unwrap_or_else(|| "Hash verification requested".into()),
        )
    } else {
        None
    };
    let blockchain_tone = if document.document_type.eq_ignore_ascii_case("blockchain") {
        Some(
            if document.hash.is_some() {
                "success"
            } else {
                "warning"
            }
            .into(),
        )
    } else {
        None
    };
    let blockchain_hash_preview = document.hash.as_ref().map(|hash| {
        if hash.len() > 24 {
            format!("{}...", &hash[..24])
        } else {
            hash.clone()
        }
    });

    KycDocumentItem {
        id: document.id.max(0) as u64,
        document_name: document.document_name,
        document_type: document.document_type,
        file_label,
        original_name: document.original_name,
        mime_type: document.mime_type,
        file_size_bytes: document
            .file_size
            .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
        uploaded_at_label: format_profile_datetime(&document.created_at),
        current_version: document.current_version.max(1) as u32,
        version_count: document.version_count.max(1) as u64,
        version_history_label: document_version_label(
            document.current_version,
            document.version_count,
        ),
        download_path: Some(kyc_document_download_path(document.id.max(0) as u64)),
        can_view_file: true,
        blockchain_label,
        blockchain_tone,
        blockchain_hash_preview,
        blockchain_hash: document.hash.clone(),
        can_edit,
        can_verify_blockchain: can_edit && document.hash.is_none(),
        can_delete: can_edit,
    }
}

fn document_version_label(current_version: i32, version_count: i64) -> String {
    let current = current_version.max(1);
    let count = version_count.max(1);
    if count == 1 {
        format!("v{} (original)", current)
    } else {
        format!("v{} of {}", current, count)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn kyc_required_document_checklist(
    role: Option<UserRole>,
    documents: &[KycDocumentItem],
) -> Vec<RequiredDocumentChecklistItem> {
    let requirements: Vec<(&str, &str, &str)> = match role {
        Some(UserRole::Carrier) => vec![
            (
                "operating_authority",
                "Operating authority",
                "Carrier onboarding",
            ),
            (
                "insurance_certificate",
                "Insurance certificate",
                "Carrier onboarding",
            ),
            ("w9", "W-9 tax form", "Carrier onboarding"),
        ],
        Some(UserRole::Broker) => vec![
            (
                "broker_authority",
                "Broker operating authority",
                "Broker onboarding",
            ),
            (
                "insurance_certificate",
                "Insurance certificate",
                "Broker onboarding",
            ),
        ],
        Some(UserRole::Shipper) => Vec::new(),
        _ => Vec::new(),
    };

    requirements
        .into_iter()
        .map(|(key, label, scope)| {
            let is_satisfied = documents
                .iter()
                .any(|document| document_matches_required_key(document, key, label));
            RequiredDocumentChecklistItem {
                key: key.into(),
                label: label.into(),
                requirement_scope: scope.into(),
                lifecycle_state: "submit_onboarding".into(),
                is_required: true,
                is_satisfied,
                status_label: if is_satisfied { "Ready" } else { "Missing" }.into(),
                status_tone: if is_satisfied { "success" } else { "warning" }.into(),
                blocking_message: (!is_satisfied).then(|| {
                    format!(
                        "{} is required before onboarding review can be completed.",
                        label
                    )
                }),
            }
        })
        .collect()
}

fn document_matches_required_key(document: &KycDocumentItem, key: &str, label: &str) -> bool {
    let haystack = format!(
        "{} {} {} {}",
        document.document_name,
        document.document_type,
        document.file_label,
        document.original_name.clone().unwrap_or_default()
    )
    .to_ascii_lowercase()
    .replace(['-', ' '], "_");
    let normalized_label = label.to_ascii_lowercase().replace(['-', ' '], "_");
    haystack.contains(key) || haystack.contains(&normalized_label)
}

fn validate_kyc_document_type(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "standard" | "blockchain" => Some(normalized),
        _ => None,
    }
}

fn validate_profile_kyc_payload(
    payload: &UpsertKycDocumentRequest,
) -> Result<UpsertKycDocumentRequest, String> {
    let document_name = payload.document_name.trim().to_string();
    if document_name.is_empty() {
        return Err("Enter a document name before saving this KYC row.".into());
    }

    let Some(document_type) = validate_kyc_document_type(&payload.document_type) else {
        return Err("Choose either standard or content hash for this KYC row.".into());
    };

    Ok(UpsertKycDocumentRequest {
        document_name,
        document_type,
    })
}

async fn upload_profile_kyc_document_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before updating profile KYC documents.",
        );
    };

    let rate_decision = state
        .check_rate_limit(upload_policy(), format!("profile:{}", resolved.user.id))
        .await;
    if !rate_decision.allowed {
        return text_response(
            StatusCode::TOO_MANY_REQUESTS,
            &rate_limit_message("profile document upload", rate_decision.retry_after_seconds),
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Profile KYC document actions are unavailable because the database connection is disabled.",
        );
    };

    let parsed = match parse_kyc_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => return text_response(StatusCode::BAD_REQUEST, &message),
    };

    let Some(document_type) = validate_kyc_document_type(&parsed.document_type) else {
        return text_response(
            StatusCode::BAD_REQUEST,
            "Choose either standard or content hash before uploading a profile document.",
        );
    };

    let saved = match state
        .document_storage
        .save_kyc_document(resolved.user.id, &parsed.original_name, &parsed.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC document storage failed: {}", error),
            );
        }
    };

    let mut record = match create_kyc_document(
        pool,
        &CreateKycDocumentInput {
            user_id: resolved.user.id,
            document_name: parsed.document_name.clone(),
            document_type: document_type.clone(),
            file_path: saved.file_path,
            original_name: Some(parsed.original_name.clone()),
            mime_type: parsed.mime_type.clone(),
            file_size: Some(parsed.bytes.len() as i64),
        },
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC document persistence failed: {}", error),
            );
        }
    };

    let pending_review = AccountStatus::PendingReview.legacy_code();
    let updated = match update_kyc_document(
        pool,
        record.id,
        resolved.user.id,
        &UpdateKycDocumentInput {
            document_name: record.document_name.clone(),
            document_type: document_type.clone(),
            file_path: None,
            original_name: None,
            mime_type: None,
            file_size: None,
            next_status: pending_review,
        },
    )
    .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return text_response(
                StatusCode::NOT_FOUND,
                "The uploaded KYC document could not be refreshed after save.",
            );
        }
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC review handoff failed: {}", error),
            );
        }
    };
    record = updated;

    if document_type == "blockchain" {
        let content_sha256 = sha256_hex(&parsed.bytes);
        record = match verify_kyc_document_blockchain(
            pool,
            record.id,
            resolved.user.id,
            &content_sha256,
            Some("Content hash calculated immediately after Rust profile upload."),
            pending_review,
        )
        .await
        {
            Ok(Some(value)) => value,
            Ok(None) => {
                return text_response(
                    StatusCode::NOT_FOUND,
                    "The uploaded KYC document disappeared before content hash verification completed.",
                );
            }
            Err(error) => {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Profile KYC content hash verification failed: {}", error),
                );
            }
        };
    }

    publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);

    Json(ApiResponse::ok(UpsertKycDocumentResponse {
        success: true,
        document_id: Some(record.id.max(0) as u64),
        message: format!(
            "Profile document {} was uploaded through the Rust revision workspace and the account is back in pending review.",
            record.document_name
        ),
    }))
    .into_response()
}

async fn update_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpsertKycDocumentRequest>,
) -> Json<ApiResponse<UpsertKycDocumentResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message: "Sign in before editing profile KYC rows.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message:
                "Profile KYC document actions are unavailable because the database connection is disabled."
                    .into(),
        }));
    };

    let payload = match validate_profile_kyc_payload(&payload) {
        Ok(value) => value,
        Err(message) => {
            return Json(ApiResponse::ok(UpsertKycDocumentResponse {
                success: false,
                document_id: Some(document_id.max(0) as u64),
                message,
            }));
        }
    };

    match update_kyc_document(
        pool,
        document_id,
        resolved.user.id,
        &UpdateKycDocumentInput {
            document_name: payload.document_name.clone(),
            document_type: payload.document_type.clone(),
            file_path: None,
            original_name: None,
            mime_type: None,
            file_size: None,
            next_status: AccountStatus::PendingReview.legacy_code(),
        },
    )
    .await
    {
        Ok(Some(record)) => {
            publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);
            Json(ApiResponse::ok(UpsertKycDocumentResponse {
                success: true,
                document_id: Some(record.id.max(0) as u64),
                message: format!(
                    "Profile document {} was updated and the account is queued for another admin review.",
                    record.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message: "That KYC row could not be found for this signed-in profile.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message: format!("Profile document update failed: {}", error),
        })),
    }
}

async fn replace_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before replacing a profile KYC file.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Profile KYC document actions are unavailable because the database connection is disabled.",
        );
    };

    let Some(existing) = find_kyc_document_by_id(pool, document_id)
        .await
        .unwrap_or(None)
    else {
        return text_response(StatusCode::NOT_FOUND, "KYC document not found.");
    };

    if existing.user_id != resolved.user.id {
        return text_response(
            StatusCode::FORBIDDEN,
            "This signed-in profile cannot replace that KYC document.",
        );
    }

    let parsed = match parse_kyc_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => return text_response(StatusCode::BAD_REQUEST, &message),
    };

    let Some(document_type) = validate_kyc_document_type(&parsed.document_type) else {
        return text_response(
            StatusCode::BAD_REQUEST,
            "Choose either standard or content hash before replacing this profile file.",
        );
    };

    let saved = match state
        .document_storage
        .save_kyc_document(resolved.user.id, &parsed.original_name, &parsed.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC replacement storage failed: {}", error),
            );
        }
    };

    let mut record = match update_kyc_document(
        pool,
        document_id,
        resolved.user.id,
        &UpdateKycDocumentInput {
            document_name: parsed.document_name.clone(),
            document_type: document_type.clone(),
            file_path: Some(saved.file_path),
            original_name: Some(parsed.original_name.clone()),
            mime_type: parsed.mime_type.clone(),
            file_size: Some(parsed.bytes.len() as i64),
            next_status: AccountStatus::PendingReview.legacy_code(),
        },
    )
    .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return text_response(
                StatusCode::NOT_FOUND,
                "That KYC row could not be found for this signed-in profile.",
            );
        }
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC replacement persistence failed: {}", error),
            );
        }
    };

    if document_type == "blockchain" {
        let content_sha256 = sha256_hex(&parsed.bytes);
        record = match verify_kyc_document_blockchain(
            pool,
            record.id,
            resolved.user.id,
            &content_sha256,
            Some("Content hash calculated immediately after file replacement in the Rust profile."),
            AccountStatus::PendingReview.legacy_code(),
        )
        .await
        {
            Ok(Some(value)) => value,
            Ok(None) => {
                return text_response(
                    StatusCode::NOT_FOUND,
                    "The replaced KYC document disappeared before content hash verification completed.",
                );
            }
            Err(error) => {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Profile KYC content hash verification failed: {}", error),
                );
            }
        };
    }

    publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);

    Json(ApiResponse::ok(UpsertKycDocumentResponse {
        success: true,
        document_id: Some(record.id.max(0) as u64),
        message: format!(
            "Profile document {} was replaced and sent back for admin review.",
            record.document_name
        ),
    }))
    .into_response()
}

async fn verify_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<VerifyKycDocumentRequest>,
) -> Json<ApiResponse<VerifyKycDocumentResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: "Sign in before anchoring a profile KYC document.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message:
                "Profile KYC content hash actions are unavailable because the database connection is disabled."
                    .into(),
        }));
    };

    let Some(existing) = find_kyc_document_by_id(pool, document_id)
        .await
        .ok()
        .flatten()
        .filter(|document| document.user_id == resolved.user.id)
    else {
        return Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: "That KYC row could not be found for this signed-in profile.".into(),
        }));
    };

    let bytes = match state
        .document_storage
        .read_document(state.document_storage.backend(), &existing.file_path)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => {
            return Json(ApiResponse::ok(VerifyKycDocumentResponse {
                success: false,
                document_id: document_id.max(0) as u64,
                hash: None,
                message: format!(
                    "Profile document content could not be read for hashing: {}",
                    error
                ),
            }));
        }
    };
    let content_sha256 = sha256_hex(&bytes);

    match verify_kyc_document_blockchain(
        pool,
        document_id,
        resolved.user.id,
        &content_sha256,
        payload.note.as_deref(),
        AccountStatus::PendingReview.legacy_code(),
    )
    .await
    {
        Ok(Some(record)) => {
            publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);
            Json(ApiResponse::ok(VerifyKycDocumentResponse {
                success: true,
                document_id: record.id.max(0) as u64,
                hash: record.hash,
                message: format!(
                    "Profile document {} now has a verified SHA-256 content hash and is pending review again.",
                    record.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: "That KYC row could not be found for this signed-in profile.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: format!("Profile content hash verification failed: {}", error),
        })),
    }
}

async fn delete_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<DeleteKycDocumentResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "Sign in before deleting a profile KYC document.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message:
                "Profile KYC document actions are unavailable because the database connection is disabled."
                    .into(),
        }));
    };

    let Some(existing) = find_kyc_document_by_id(pool, document_id)
        .await
        .unwrap_or(None)
    else {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "That KYC row no longer exists.".into(),
        }));
    };

    if existing.user_id != resolved.user.id {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "This signed-in profile cannot delete that KYC row.".into(),
        }));
    }
    if resolved.user.primary_role() == Some(UserRole::Admin)
        && !resolved
            .session
            .permissions
            .iter()
            .any(|permission| permission == "mfa_verified")
    {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "MFA step-up is required before an admin deletes profile KYC documents."
                .into(),
        }));
    }

    match delete_kyc_document(
        pool,
        document_id,
        resolved.user.id,
        AccountStatus::PendingReview.legacy_code(),
    )
    .await
    {
        Ok(Some(record)) => {
            if let Err(error) = state
                .document_storage
                .delete_document(
                    normalize_storage_provider(&record.file_path),
                    &record.file_path,
                )
                .await
            {
                tracing::warn!(
                    document_id,
                    error = %error,
                    "profile KYC binary could not be removed after row deletion"
                );
            }

            publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);

            Json(ApiResponse::ok(DeleteKycDocumentResponse {
                success: true,
                document_id: record.id.max(0) as u64,
                message: format!(
                    "Profile document {} was removed and the account is queued for another review cycle.",
                    record.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "That KYC row could not be found for this signed-in profile.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: format!("Profile KYC deletion failed: {}", error),
        })),
    }
}

fn publish_profile_document_realtime(state: &AppState, actor_name: &str, actor_user_id: i64) {
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(actor_user_id.max(0) as u64),
            subject_user_id: Some(actor_user_id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} updated profile KYC documents through the Rust revision workspace.",
                actor_name
            ),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"]),
    );
}

async fn upload_kyc_document_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before uploading a KYC document.",
        );
    };

    let rate_decision = state
        .check_rate_limit(upload_policy(), format!("onboarding:{}", resolved.user.id))
        .await;
    if !rate_decision.allowed {
        return text_response(
            StatusCode::TOO_MANY_REQUESTS,
            &rate_limit_message("KYC document upload", rate_decision.retry_after_seconds),
        );
    }

    if !resolved
        .user
        .account_status()
        .map(|status| can_submit_onboarding(Some(status)))
        .unwrap_or(false)
    {
        return text_response(
            StatusCode::FORBIDDEN,
            "This account cannot upload onboarding KYC documents in the current Rust state.",
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Rust KYC upload is unavailable because the database connection is disabled.",
        );
    };

    let parsed = match parse_kyc_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => return text_response(StatusCode::BAD_REQUEST, &message),
    };

    let saved = match state
        .document_storage
        .save_kyc_document(resolved.user.id, &parsed.original_name, &parsed.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("KYC document storage failed: {}", error),
            );
        }
    };

    let record = match create_kyc_document(
        pool,
        &CreateKycDocumentInput {
            user_id: resolved.user.id,
            document_name: parsed.document_name,
            document_type: parsed.document_type,
            file_path: saved.file_path,
            original_name: Some(parsed.original_name),
            mime_type: parsed.mime_type,
            file_size: Some(parsed.bytes.len() as i64),
        },
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("KYC document persistence failed: {}", error),
            );
        }
    };

    let response = ApiResponse::ok(build_kyc_document_item(record, true));

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(resolved.user.id.max(0) as u64),
            subject_user_id: Some(resolved.user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} uploaded a KYC document through Rust onboarding.",
                resolved.user.name
            ),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"]),
    );

    Json(response).into_response()
}

async fn download_kyc_document_file(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before opening this KYC document.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Rust KYC documents are unavailable because the database connection is disabled.",
        );
    };

    let Some(document) = find_kyc_document_by_id(pool, document_id)
        .await
        .unwrap_or(None)
    else {
        return text_response(StatusCode::NOT_FOUND, "KYC document not found.");
    };

    let can_view = resolved.user.primary_role() == Some(UserRole::Admin)
        || document.user_id == resolved.user.id;
    if !can_view {
        return text_response(
            StatusCode::FORBIDDEN,
            "This KYC document is restricted to the uploader and admin users.",
        );
    }

    let bytes = match state
        .document_storage
        .read_document(
            normalize_storage_provider(&document.file_path),
            &document.file_path,
        )
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("KYC document read failed: {}", error),
            );
        }
    };

    let mime = document
        .mime_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("application/octet-stream");
    let file_name = document
        .original_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("document.bin");

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "inline; filename=\"{}\"",
            sanitize_header_file_name(file_name)
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    response
}

#[derive(Debug)]
struct ParsedKycDocumentUpload {
    document_name: String,
    document_type: String,
    original_name: String,
    mime_type: Option<String>,
    bytes: Vec<u8>,
}

async fn parse_kyc_document_upload(
    mut multipart: Multipart,
) -> Result<ParsedKycDocumentUpload, String> {
    let mut document_name = None::<String>;
    let mut document_type = None::<String>;
    let mut original_name = None::<String>;
    let mut mime_type = None::<String>;
    let mut bytes = None::<Vec<u8>>;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| format!("KYC upload parsing failed: {}", error))?
    {
        match field.name().unwrap_or_default() {
            "document_name" => {
                document_name = Some(
                    field
                        .text()
                        .await
                        .map_err(|error| format!("Document name parsing failed: {}", error))?,
                );
            }
            "document_type" => {
                document_type = Some(
                    field
                        .text()
                        .await
                        .map_err(|error| format!("Document type parsing failed: {}", error))?,
                );
            }
            "file" => {
                original_name = field.file_name().map(str::to_string);
                mime_type = field.content_type().map(str::to_string);
                let payload = field
                    .bytes()
                    .await
                    .map_err(|error| format!("Document file parsing failed: {}", error))?;
                bytes = Some(payload.to_vec());
            }
            _ => {}
        }
    }

    let document_name = document_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Enter a document name before uploading a KYC file.".to_string())?;
    let document_type = document_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Choose a KYC document type before uploading a file.".to_string())?;
    let bytes =
        bytes.ok_or_else(|| "Choose a file before uploading a KYC document.".to_string())?;
    let original_name = original_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "document.bin".into());
    let verdict = validate_uploaded_document(&original_name, mime_type.as_deref(), &bytes)?;

    Ok(ParsedKycDocumentUpload {
        document_name,
        document_type,
        original_name,
        mime_type: verdict.normalized_mime_type,
        bytes,
    })
}

fn normalize_storage_provider(file_path: &str) -> &str {
    if file_path.starts_with("local://") {
        "local"
    } else if file_path.starts_with("ibm-cos://") {
        "ibm_cos"
    } else if file_path.starts_with("s3://") {
        "s3"
    } else {
        "local"
    }
}

fn kyc_document_download_path(document_id: u64) -> String {
    format!("/auth/onboarding/documents/{}/file", document_id)
}

fn profile_file_label(file_path: &str) -> String {
    let normalized = file_path
        .strip_prefix("local://")
        .or_else(|| file_path.strip_prefix("ibm-cos://"))
        .or_else(|| file_path.strip_prefix("s3://"))
        .unwrap_or(file_path);
    normalized
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("document.bin")
        .to_string()
}

fn sanitize_header_file_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '"' | '\\' | '\r' | '\n' => '_',
            _ => ch,
        })
        .collect::<String>();
    if sanitized.trim().is_empty() {
        "document.bin".into()
    } else {
        sanitized
    }
}

fn format_profile_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
}

fn text_response(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

fn verify_user_password(password: &str, stored_password: &str) -> bool {
    if verify(password, stored_password).unwrap_or(false) {
        return true;
    }

    if let Some(laravel_bcrypt) = stored_password.strip_prefix("$2y$") {
        let normalized_hash = format!("$2b${laravel_bcrypt}");
        if verify(password, &normalized_hash).unwrap_or(false) {
            return true;
        }
    }

    password == stored_password
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        auth_headers_for_user, fetch_password_reset_token, insert_user_with_role_status,
        prepare_pool, test_state,
    };
    use serial_test::serial;
    use shared::{
        EnterpriseSsoDiscoveryRequest, EnterpriseSsoOidcCallbackRequest, ForgotPasswordRequest,
        LoginRequest, RegisterRequest, ScimDeprovisionRequest, ScimUpsertUserRequest,
        UpdateCarrierCapacityRequest,
    };

    #[test]
    fn sha256_hex_uses_file_bytes_not_mock_tokens() {
        assert_eq!(
            sha256_hex(b"stloads document bytes"),
            "2c236321204a3754c2f4ebd233c071f0118706b551d811b7c852e3a393c294db"
        );
    }

    #[test]
    fn password_verification_accepts_laravel_bcrypt_prefix() {
        let bcrypt_hash = hash("Password123!", 4).expect("bcrypt hash");
        let laravel_hash = bcrypt_hash.replacen("$2b$", "$2y$", 1);

        assert!(verify_user_password("Password123!", &laravel_hash));
        assert!(!verify_user_password("WrongPassword123!", &laravel_hash));
    }

    #[test]
    fn carrier_required_document_checklist_reports_missing_and_ready_documents() {
        let missing = kyc_required_document_checklist(Some(UserRole::Carrier), &[]);
        assert_eq!(missing.len(), 3);
        assert!(missing.iter().all(|item| !item.is_satisfied));
        assert!(
            missing
                .iter()
                .any(|item| item.key == "operating_authority" && item.blocking_message.is_some())
        );

        let documents = vec![
            test_kyc_document_item("Operating authority", "operating-authority.pdf"),
            test_kyc_document_item("Insurance certificate", "coi.pdf"),
            test_kyc_document_item("W-9 tax form", "w9.pdf"),
        ];
        let ready = kyc_required_document_checklist(Some(UserRole::Carrier), &documents);
        assert!(ready.iter().all(|item| item.is_satisfied));
        assert!(ready.iter().all(|item| item.status_tone == "success"));
    }

    #[test]
    fn carrier_capacity_readiness_reports_matching_eligibility() {
        assert_eq!(
            carrier_capacity_readiness(
                &["dry_van".into()],
                &["tx".into()],
                "available",
                2,
                1_000_000.0,
            ),
            "Eligible for capacity matching"
        );
        assert!(
            carrier_capacity_readiness(&[], &["tx".into()], "available", 2, 1_000_000.0)
                .contains("missing equipment")
        );
        assert!(
            carrier_capacity_readiness(
                &["dry_van".into()],
                &["tx".into()],
                "paused",
                2,
                1_000_000.0
            )
            .contains("unavailable")
        );
    }

    #[tokio::test]
    #[serial]
    async fn carrier_capacity_profile_can_be_saved_and_loaded()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let carrier = insert_user_with_role_status(
            &pool,
            "Capacity Carrier",
            "capacity-carrier@example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        let headers = auth_headers_for_user(&state, &carrier).await?;

        let response = update_carrier_capacity(
            State(state.clone()),
            headers.clone(),
            Json(UpdateCarrierCapacityRequest {
                equipment_types: vec!["Dry Van".into(), "dry-van".into(), "Reefer".into()],
                lane_preferences: vec!["Dallas to Chicago".into()],
                operating_regions: vec!["TX".into(), "Midwest".into()],
                preferred_commodities: vec!["consumer goods".into()],
                service_levels: vec!["standard".into()],
                certifications: vec!["hazmat".into()],
                availability_status: "available".into(),
                available_power_units: 3,
                insurance_limit_usd: 1_000_000.0,
                capacity_notes: Some("Weekday capacity only.".into()),
            }),
        )
        .await
        .0
        .data;
        assert!(response.success, "{}", response.message);
        let capacity = response.capacity.expect("capacity payload");
        assert_eq!(capacity.equipment_types, vec!["dry_van", "reefer"]);
        assert_eq!(capacity.readiness_label, "Eligible for capacity matching");

        let screen = profile_screen(State(state), headers).await.0.data;
        let screen_capacity = screen.carrier_capacity.expect("screen capacity");
        assert_eq!(screen_capacity.available_power_units, 3);
        assert_eq!(screen_capacity.insurance_limit_usd, 1_000_000.0);

        Ok(())
    }

    fn test_kyc_document_item(document_name: &str, file_label: &str) -> KycDocumentItem {
        KycDocumentItem {
            id: 1,
            document_name: document_name.into(),
            document_type: "standard".into(),
            file_label: file_label.into(),
            original_name: Some(file_label.into()),
            mime_type: Some("application/pdf".into()),
            file_size_bytes: Some(100),
            uploaded_at_label: "May 25, 2026 00:00".into(),
            download_path: None,
            can_view_file: true,
            blockchain_label: None,
            blockchain_tone: None,
            blockchain_hash_preview: None,
            blockchain_hash: None,
            can_edit: true,
            can_verify_blockchain: false,
            can_delete: true,
            current_version: 1,
            version_count: 1,
            version_history_label: "v1 (original)".into(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn registration_and_password_reset_routes_work_end_to_end()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let email = "route-auth@example.com".to_string();

        let register_response = register(
            State(state.clone()),
            HeaderMap::new(),
            Json(RegisterRequest {
                name: "Route Auth".into(),
                email: email.clone(),
                password: "Password123!".into(),
                password_confirmation: "Password123!".into(),
                role_key: "shipper".into(),
                phone_no: Some("555-0101".into()),
                address: Some("100 Auth Test Way".into()),
            }),
        )
        .await
        .0
        .data;
        assert!(register_response.success);
        let registration_otp = register_response
            .dev_otp
            .clone()
            .expect("development registration otp");

        let verify_registration = verify_otp(
            State(state.clone()),
            HeaderMap::new(),
            Json(VerifyOtpRequest {
                email: email.clone(),
                otp: registration_otp,
                purpose: OtpPurpose::Registration,
            }),
        )
        .await
        .0
        .data;
        assert!(verify_registration.success);
        assert!(verify_registration.token.is_some());
        assert_eq!(verify_registration.next_step, "/auth/onboarding");

        let forgot_response = forgot_password(
            State(state.clone()),
            HeaderMap::new(),
            Json(ForgotPasswordRequest {
                email: email.clone(),
            }),
        )
        .await
        .0
        .data;
        assert!(forgot_response.success);
        let reset_otp = forgot_response
            .dev_otp
            .clone()
            .expect("development reset otp");

        let verify_reset = verify_otp(
            State(state.clone()),
            HeaderMap::new(),
            Json(VerifyOtpRequest {
                email: email.clone(),
                otp: reset_otp,
                purpose: OtpPurpose::PasswordReset,
            }),
        )
        .await
        .0
        .data;
        assert!(verify_reset.success);
        let reset_token = verify_reset
            .reset_token
            .clone()
            .expect("development reset token");
        assert_eq!(
            fetch_password_reset_token(&pool, &email).await?,
            Some(reset_token.clone())
        );

        let reset_response = reset_password(
            State(state.clone()),
            HeaderMap::new(),
            Json(ResetPasswordRequest {
                email: email.clone(),
                reset_token,
                password: "Password456!".into(),
                password_confirmation: "Password456!".into(),
            }),
        )
        .await
        .0
        .data;
        assert!(reset_response.success);

        let login_response = login(
            State(state),
            HeaderMap::new(),
            Json(LoginRequest {
                email,
                password: "Password456!".into(),
            }),
        )
        .await
        .0
        .data;
        assert!(login_response.success);
        let issued_token = login_response.token.clone().expect("login token");
        let stored_token = sqlx::query_scalar::<_, String>(
            "SELECT token
             FROM personal_access_tokens
             WHERE tokenable_type = 'App\\Models\\User'
             ORDER BY id DESC
             LIMIT 1",
        )
        .fetch_one(&pool)
        .await?;
        assert_ne!(stored_token, issued_token);
        assert!(!stored_token.starts_with("stl_"));
        assert_eq!(
            login_response
                .session
                .user
                .as_ref()
                .map(|user| user.account_status_label.as_str()),
            Some("Email Verified")
        );

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn admin_login_requires_mfa_before_session_token()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin = insert_user_with_role_status(
            &pool,
            "MFA Admin",
            "mfa-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;

        let login_response = login(
            State(state.clone()),
            HeaderMap::new(),
            Json(LoginRequest {
                email: admin.email.clone(),
                password: "Password123!".into(),
            }),
        )
        .await
        .0
        .data;

        assert!(!login_response.success);
        assert!(login_response.token.is_none());
        assert!(login_response.mfa_required);
        let challenge_id = login_response
            .mfa_challenge_id
            .clone()
            .expect("mfa challenge id");
        let mfa_code = login_response.dev_mfa_code.clone().expect("dev mfa code");

        let verify_response = verify_mfa(
            State(state.clone()),
            HeaderMap::new(),
            Json(MfaVerifyRequest {
                email: admin.email.clone(),
                challenge_id,
                code: mfa_code,
            }),
        )
        .await
        .0
        .data;

        assert!(verify_response.success);
        assert!(verify_response.token.is_some());
        assert!(
            verify_response
                .session
                .as_ref()
                .expect("session")
                .permissions
                .iter()
                .any(|permission| permission == "mfa_verified")
        );
        assert_eq!(verify_response.recovery_codes.len(), 8);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn enterprise_sso_discovery_blocks_password_login_when_routing_is_active()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let organization_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO organizations (name, slug, account_type, status, support_tier, created_at, updated_at)
             VALUES ('SSO Customer', 'sso-customer', 'customer', 'active', 'enterprise', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO organization_domains (
                organization_id, domain, verification_status, verification_token, verified_at, login_routing_enabled, created_at, updated_at
             )
             VALUES ($1, 'sso.example.com', 'verified', 'verify-sso-example', CURRENT_TIMESTAMP, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(organization_id)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO enterprise_identity_providers (
                organization_id, provider_type, status, display_name, sso_url, jit_enabled, default_role_key, created_at, updated_at
             )
             VALUES ($1, 'oidc', 'active', 'Example IdP', 'https://idp.example.com/login', TRUE, 'member', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(organization_id)
        .execute(&pool)
        .await?;
        let user = insert_user_with_role_status(
            &pool,
            "SSO Routed User",
            "driver@sso.example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        sqlx::query("UPDATE users SET organization_id = $1 WHERE id = $2")
            .bind(organization_id)
            .bind(user.id)
            .execute(&pool)
            .await?;

        let discovery = enterprise_sso_discovery(
            State(state.clone()),
            Json(EnterpriseSsoDiscoveryRequest {
                email: user.email.clone(),
            }),
        )
        .await
        .0
        .data;
        assert!(discovery.sso_required);
        assert!(!discovery.password_allowed);
        assert_eq!(discovery.provider_type.as_deref(), Some("oidc"));

        let login_response = login(
            State(state),
            HeaderMap::new(),
            Json(LoginRequest {
                email: user.email,
                password: "Password123!".into(),
            }),
        )
        .await
        .0
        .data;
        assert!(!login_response.success);
        assert_eq!(
            login_response.next_step.as_deref(),
            Some("https://idp.example.com/login")
        );

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn enterprise_sso_oidc_callback_rejects_unconfigured_domain()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool);

        let response = enterprise_sso_oidc_callback(
            State(state),
            Json(EnterpriseSsoOidcCallbackRequest {
                email: "nobody@unconfigured.example".into(),
                id_token: "not-a-jwt".into(),
            }),
        )
        .await;

        assert!(matches!(response, Err(StatusCode::UNAUTHORIZED)));
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn scim_deprovision_revokes_active_sessions() -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let user = insert_user_with_role_status(
            &pool,
            "SCIM User",
            "scim-user@example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        let session_token = auth_session::issue_session_token(&state, &user).await?;
        assert!(
            auth_session::resolve_session_from_token(&state, &session_token)
                .await?
                .is_some()
        );
        sqlx::query(
            "INSERT INTO sessions (id, user_id, payload, last_activity)
             VALUES ('legacy-session-scim', $1, '{}', 1)",
        )
        .bind(user.id)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO password_reset_tokens (email, token, created_at)
             VALUES ($1, 'reset-before-scim', CURRENT_TIMESTAMP)
             ON CONFLICT (email) DO UPDATE SET token = EXCLUDED.token",
        )
        .bind(&user.email)
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO mfa_challenges (id, user_id, email, purpose, code_hash, expires_at, created_at, updated_at)
             VALUES ($1, $2, $3, 'login', 'hash-before-scim', CURRENT_TIMESTAMP + INTERVAL '5 minutes', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(Uuid::new_v4())
        .bind(user.id)
        .bind(&user.email)
        .execute(&pool)
        .await?;
        sqlx::query("UPDATE users SET remember_token = 'legacy-remember' WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await?;

        let token_prefix = "testscim";
        let token_secret = "secret-value";
        sqlx::query(
            "INSERT INTO scim_tokens (
                organization_id, token_prefix, token_hash, label, status, created_at
             )
             VALUES (1, $1, $2, 'Test SCIM token', 'active', CURRENT_TIMESTAMP)",
        )
        .bind(token_prefix)
        .bind(hash_scim_secret(token_secret))
        .execute(&pool)
        .await?;
        sqlx::query(
            "INSERT INTO scim_user_links (
                organization_id, user_id, external_id, active, created_at, updated_at
             )
             VALUES (1, $1, 'external-scim-user', TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(user.id)
        .execute(&pool)
        .await?;

        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer scim_{}.{}", token_prefix, token_secret))?,
        );
        let response = scim_deprovision(
            State(state.clone()),
            headers,
            Json(ScimDeprovisionRequest {
                external_id: Some("external-scim-user".into()),
                email: None,
                user_id: None,
                reason: Some("Removed in identity provider".into()),
            }),
        )
        .await
        .map_err(|status| format!("SCIM deprovision failed with status {status}"))?
        .0
        .data;

        assert!(response.success);
        assert_eq!(response.user_id, Some(user.id.max(0) as u64));
        assert_eq!(response.revoked_sessions, 5);
        assert!(
            auth_session::resolve_session_from_token(&state, &session_token)
                .await?
                .is_none()
        );

        let membership_status = sqlx::query_scalar::<_, String>(
            "SELECT status
             FROM organization_memberships
             WHERE organization_id = 1
               AND user_id = $1",
        )
        .bind(user.id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(membership_status, "deprovisioned");
        let remaining_artifacts = sqlx::query_scalar::<_, i64>(
            "SELECT
                (SELECT COUNT(*) FROM personal_access_tokens WHERE tokenable_id = $1)
              + (SELECT COUNT(*) FROM sessions WHERE user_id = $1)
              + (SELECT COUNT(*) FROM password_reset_tokens WHERE email = $2)
              + (SELECT COUNT(*) FROM mfa_challenges WHERE user_id = $1)
              + (SELECT COUNT(*) FROM users WHERE id = $1 AND remember_token IS NOT NULL)",
        )
        .bind(user.id)
        .bind(&user.email)
        .fetch_one(&pool)
        .await?;
        assert_eq!(remaining_artifacts, 0);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn scim_upsert_provisions_updates_and_reactivates_users()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let token_prefix = "upsertscim";
        let token_secret = "secret-value";
        sqlx::query(
            "INSERT INTO scim_tokens (
                organization_id, token_prefix, token_hash, label, status, created_at
             )
             VALUES (1, $1, $2, 'Upsert SCIM token', 'active', CURRENT_TIMESTAMP)",
        )
        .bind(token_prefix)
        .bind(hash_scim_secret(token_secret))
        .execute(&pool)
        .await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer scim_{}.{}", token_prefix, token_secret))?,
        );

        let created = scim_upsert_user(
            State(state.clone()),
            headers.clone(),
            Json(ScimUpsertUserRequest {
                external_id: "worker-100".into(),
                email: "worker-100@example.com".into(),
                name: "Worker One Hundred".into(),
                role_key: Some("carrier".into()),
                active: true,
                reason: Some("Initial IdP assignment".into()),
            }),
        )
        .await
        .map_err(|status| format!("SCIM provision failed with status {status}"))?
        .0
        .data;
        assert!(created.success);
        assert!(created.created);
        let user_id = created.user_id.expect("created user id") as i64;

        let user = find_user_by_id(&pool, user_id)
            .await?
            .expect("created user");
        let session_token = auth_session::issue_session_token(&state, &user).await?;
        assert!(
            auth_session::resolve_session_from_token(&state, &session_token)
                .await?
                .is_some()
        );

        let updated = scim_upsert_user(
            State(state.clone()),
            headers.clone(),
            Json(ScimUpsertUserRequest {
                external_id: "worker-100".into(),
                email: "worker-100@example.com".into(),
                name: "Worker 100 Updated".into(),
                role_key: Some("broker".into()),
                active: true,
                reason: Some("Role changed in IdP".into()),
            }),
        )
        .await
        .map_err(|status| format!("SCIM update failed with status {status}"))?
        .0
        .data;
        assert!(updated.success);
        assert!(!updated.created);
        assert_eq!(updated.revoked_sessions, 1);
        assert!(
            auth_session::resolve_session_from_token(&state, &session_token)
                .await?
                .is_none()
        );

        let deactivated = scim_upsert_user(
            State(state.clone()),
            headers.clone(),
            Json(ScimUpsertUserRequest {
                external_id: "worker-100".into(),
                email: "worker-100@example.com".into(),
                name: "Worker 100 Updated".into(),
                role_key: Some("broker".into()),
                active: false,
                reason: Some("Temporarily removed".into()),
            }),
        )
        .await
        .map_err(|status| format!("SCIM deactivate failed with status {status}"))?
        .0
        .data;
        assert!(deactivated.success);

        let membership_status = sqlx::query_scalar::<_, String>(
            "SELECT status FROM organization_memberships WHERE organization_id = 1 AND user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(membership_status, "deprovisioned");

        let reactivated = scim_upsert_user(
            State(state),
            headers,
            Json(ScimUpsertUserRequest {
                external_id: "worker-100".into(),
                email: "worker-100@example.com".into(),
                name: "Worker 100 Updated".into(),
                role_key: Some("broker".into()),
                active: true,
                reason: Some("Restored in IdP".into()),
            }),
        )
        .await
        .map_err(|status| format!("SCIM reactivate failed with status {status}"))?
        .0
        .data;
        assert!(reactivated.success);
        assert!(reactivated.reactivated);

        let event_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM scim_events
             WHERE organization_id = 1
               AND external_id = 'worker-100'
               AND action IN ('provision', 'update', 'reactivate')",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(event_count, 4);

        Ok(())
    }
}
