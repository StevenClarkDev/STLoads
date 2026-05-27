use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use bcrypt::hash;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use db::{
    access_reviews::{
        AccessReviewDecisionInput, CreateAccessReviewInput, create_access_review,
        decide_access_review_item, latest_access_review_items, list_access_reviews,
    },
    audit::{
        AuditEventInput, CreateBreakGlassSessionInput, create_break_glass_session,
        has_active_break_glass_session, insert_audit_event,
    },
    auth::{
        CreateAdminUserInput, UpdateAdminUserProfileInput, create_admin_user_account,
        delete_admin_user_account, delete_personal_access_tokens_for_user, find_user_by_id,
        find_user_detail_by_user_id, list_admin_users, list_kyc_documents_by_user_id,
        list_pending_onboarding_users, list_permission_names_for_role, list_user_history_entries,
        list_user_ids_for_role, replace_role_permissions, review_onboarding_user,
        revoke_all_access_artifacts_for_user, update_admin_user_account, update_admin_user_profile,
    },
    dispatch::{count_admin_load_legs_filtered, list_admin_load_legs_filtered, review_load_status},
    integrations::{list_webhook_delivery_logs, mark_webhook_delivery_for_replay},
    tms::{
        list_open_tms_conflicts, list_tms_source_of_truth_rules, repair_tms_conflict,
        resolve_sync_error,
    },
};
use domain::auth::{
    AccountStatus, UserRole, permission_descriptors, role_descriptors, role_permission_contracts,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::{
    AdminAccessElevationDecisionRequest, AdminAccessElevationRequestRow,
    AdminAccessReviewDecisionRequest, AdminAccessReviewItemRow, AdminAccessReviewMutationResponse,
    AdminAccessReviewRow, AdminAccessReviewScreen, AdminAuditEventRow, AdminAuditExportResponse,
    AdminAuditSearchFilters, AdminAuditSearchScreen, AdminBreakGlassRequest,
    AdminBreakGlassResponse, AdminCheckIdentityDomainDnsRequest, AdminCreateAccessElevationRequest,
    AdminCreateSupportCaseRequest, AdminCreateSupportNoteRequest, AdminCreateSupportNoteResponse,
    AdminCreateUserRequest, AdminCreateUserResponse, AdminDeleteUserResponse,
    AdminIdentityDomainRow, AdminIdentityMutationResponse, AdminIdentityProviderRow,
    AdminIdentityScimEventRow, AdminIdentityScreen, AdminLoadListScreen, AdminLoadRow,
    AdminLoadTab, AdminOnboardingReviewScreen, AdminOnboardingReviewUser, AdminReviewLoadRequest,
    AdminReviewLoadResponse, AdminRolePermissionOption, AdminRolePermissionRow,
    AdminRolePermissionScreen, AdminStartAccessReviewRequest, AdminSupportCaseFeedbackRequest,
    AdminSupportCaseMutationResponse, AdminSupportCaseRow, AdminSupportCaseScreen,
    AdminSupportSearchFact, AdminSupportSearchResult, AdminSupportSearchScreen,
    AdminSupportTimelineEntry, AdminSupportTimelineRequest, AdminSupportTimelineScreen,
    AdminUpdateRolePermissionsRequest, AdminUpdateRolePermissionsResponse,
    AdminUpdateSupportCaseRequest, AdminUpdateUserProfileRequest, AdminUpdateUserProfileResponse,
    AdminUpdateUserRequest, AdminUpdateUserResponse, AdminUpsertIdentityDomainRequest,
    AdminUpsertIdentityProviderRequest, AdminUserDirectoryRoleOption, AdminUserDirectoryScreen,
    AdminUserDirectoryStatusOption, AdminUserDirectoryUser, AdminUserHistoryItem,
    AdminUserProfileFact, AdminUserProfileScreen, AdminVerifyIdentityDomainRequest, ApiResponse,
    KycDocumentItem, RealtimeEvent, RealtimeEventKind, RealtimeTopic, ResolveSyncErrorRequest,
    ResolveSyncErrorResponse, ReviewOnboardingRequest, ReviewOnboardingResponse,
    StloadsOperationsScreen, StloadsReconciliationScreen,
};
use sqlx::Row;
use std::collections::HashMap;

use crate::{
    app::REQUEST_ID_HEADER, auth_session, auth_session::ResolvedSession,
    realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState,
};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct AdminOverview {
    screen_routes: Vec<&'static str>,
    operational_views: usize,
    user_total: usize,
    shipper_total: usize,
    carrier_total: usize,
    broker_total: usize,
    freight_forwarder_total: usize,
    admin_total: usize,
    notes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct WebhookDeliveryQuery {
    status: Option<String>,
}

#[derive(Debug, Serialize)]
struct WebhookDeliveryScreen {
    organization_id: u64,
    status_filter: Option<String>,
    deliveries: Vec<WebhookDeliveryRow>,
}

#[derive(Debug, Serialize)]
struct WebhookDeliveryRow {
    id: u64,
    endpoint_id: Option<u64>,
    event_type: String,
    event_id: String,
    delivery_status: String,
    attempt_count: i32,
    next_retry_at: Option<String>,
    last_attempt_at: Option<String>,
    response_status_code: Option<i32>,
    response_latency_ms: Option<i32>,
    response_body_excerpt: Option<String>,
    dead_letter_reason: Option<String>,
    replay_of_delivery_id: Option<u64>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct WebhookReplayResponse {
    success: bool,
    delivery_id: u64,
    replay_delivery_id: Option<u64>,
    message: String,
}

#[derive(Debug, Serialize)]
struct TmsConflictScreen {
    source_of_truth_rules: Vec<TmsSourceRuleRow>,
    conflicts: Vec<TmsConflictRow>,
}

#[derive(Debug, Serialize)]
struct TmsSourceRuleRow {
    field_key: String,
    owning_system: String,
    conflict_policy: String,
    default_repair_action: String,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct TmsConflictRow {
    id: u64,
    handoff_id: u64,
    field_key: String,
    source_of_truth: String,
    stloads_value: Option<String>,
    tms_value: Option<String>,
    conflict_status: String,
    severity: String,
    repair_action: String,
    detected_by: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct RepairTmsConflictRequest {
    resolution_note: Option<String>,
}

#[derive(Debug, Serialize)]
struct RepairTmsConflictResponse {
    success: bool,
    conflict_id: u64,
    status: Option<String>,
    message: String,
}

#[derive(Debug, Serialize)]
struct IntegrationPortalScreen {
    organization_id: u64,
    api_version: &'static str,
    docs: Vec<IntegrationDocLink>,
    sandbox: IntegrationSandboxSummary,
    api_keys: Vec<IntegrationApiKeyRow>,
    webhook_endpoints: Vec<IntegrationWebhookEndpointRow>,
    recent_deliveries: Vec<WebhookDeliveryRow>,
}

#[derive(Debug, Serialize)]
struct IntegrationDocLink {
    label: &'static str,
    href: &'static str,
    description: &'static str,
}

#[derive(Debug, Serialize)]
struct IntegrationSandboxSummary {
    base_url: &'static str,
    production_safety: &'static str,
    reset_policy: &'static str,
}

#[derive(Debug, Serialize)]
struct IntegrationApiKeyRow {
    id: u64,
    client_name: String,
    key_prefix: String,
    scopes: Vec<String>,
    status: String,
    rate_limit_per_minute: i32,
    require_request_signature: bool,
    last_used_at: Option<String>,
    expires_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct IntegrationWebhookEndpointRow {
    id: u64,
    endpoint_name: String,
    target_url: String,
    event_types: Vec<String>,
    status: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreatePartnerApiKeyRequest {
    client_name: String,
    scopes: Vec<String>,
    rate_limit_per_minute: Option<i32>,
    require_request_signature: Option<bool>,
    expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreatePartnerApiKeyResponse {
    success: bool,
    api_key: Option<String>,
    key_prefix: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct UpsertWebhookEndpointRequest {
    endpoint_name: String,
    target_url: String,
    event_types: Vec<String>,
    status: Option<String>,
}

#[derive(Debug, Serialize)]
struct IntegrationMutationResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct EdiIntegrationScreen {
    supported_transactions: Vec<EdiTransactionMappingRow>,
    partner_profiles: Vec<EdiPartnerProfileRow>,
    message_logs: Vec<EdiMessageLogRow>,
    replay_policy: &'static str,
}

#[derive(Debug, Serialize)]
struct EdiTransactionMappingRow {
    id: u64,
    transaction_code: String,
    direction: String,
    stloads_model: String,
    mapping_version: String,
    status: String,
    required_fields: Vec<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize)]
struct EdiPartnerProfileRow {
    id: u64,
    partner_name: String,
    isa_id: Option<String>,
    gs_id: Option<String>,
    transport_type: String,
    status: String,
    supported_transactions: Vec<String>,
    validation_mode: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct EdiMessageLogRow {
    id: u64,
    partner_profile_id: Option<u64>,
    transaction_code: String,
    direction: String,
    control_number: Option<String>,
    business_key: Option<String>,
    message_status: String,
    ack_status: String,
    retry_count: i32,
    next_retry_at: Option<String>,
    error_summary: Option<String>,
    replay_of_message_id: Option<u64>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct UpsertEdiPartnerProfileRequest {
    partner_name: String,
    isa_id: Option<String>,
    gs_id: Option<String>,
    transport_type: Option<String>,
    status: Option<String>,
    supported_transactions: Vec<String>,
    validation_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValidateEdiMessageRequest {
    transaction_code: String,
    direction: String,
    control_number: Option<String>,
    business_key: Option<String>,
    payload_excerpt: Option<String>,
}

#[derive(Debug, Serialize)]
struct EdiValidationResponse {
    success: bool,
    message_id: Option<u64>,
    missing_fields: Vec<String>,
    ack_status: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct EdiReplayResponse {
    success: bool,
    message_id: u64,
    replay_message_id: Option<u64>,
    message: String,
}

#[derive(Debug, Serialize)]
struct SandboxGovernanceScreen {
    environments: Vec<SandboxEnvironmentRow>,
    reset_jobs: Vec<SandboxResetJobRow>,
    policy_notes: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct SandboxEnvironmentRow {
    id: u64,
    environment_key: String,
    display_name: String,
    base_url: String,
    data_classification: String,
    pii_allowed: bool,
    production_payment_blocked: bool,
    production_tms_push_blocked: bool,
    production_notification_blocked: bool,
    seeded_dataset_version: String,
    reset_status: String,
    last_reset_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct SandboxResetJobRow {
    id: u64,
    sandbox_environment_id: u64,
    job_status: String,
    reset_reason: Option<String>,
    result_summary: Option<String>,
    created_at: String,
    completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QueueSandboxResetRequest {
    sandbox_environment_id: u64,
    reset_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct SandboxResetResponse {
    success: bool,
    reset_job_id: Option<u64>,
    message: String,
}

#[derive(Debug, Serialize)]
struct ApiLifecycleScreen {
    policies: Vec<ApiLifecyclePolicyRow>,
    examples: Vec<ApiPartnerExampleRow>,
    upgrade_paths: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ApiLifecyclePolicyRow {
    api_version: String,
    release_status: String,
    released_on: String,
    sunset_on: Option<String>,
    minimum_notice_days: i32,
    emergency_breaking_change_policy: String,
    changelog_url: String,
    postman_collection_url: Option<String>,
    sdk_strategy: String,
    compatibility_test_status: String,
}

#[derive(Debug, Serialize)]
struct ApiPartnerExampleRow {
    api_version: String,
    example_key: String,
    surface: String,
    method: String,
    path: String,
    sandbox_runnable: bool,
}

#[derive(Debug, Deserialize)]
struct OperationsQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReconciliationQuery {
    action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdminLoadsQuery {
    tab: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdminSupportSearchQuery {
    q: Option<String>,
    target_organization_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AdminSupportCaseQuery {
    target_organization_id: Option<u64>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdminIdentityQuery {
    target_organization_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AdminAccessReviewQuery {
    target_organization_id: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
struct AdminAuditSearchQuery {
    q: Option<String>,
    target_organization_id: Option<u64>,
    actor_user_id: Option<u64>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    action: Option<String>,
    request_id: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/stloads/operations", get(stloads_operations))
        .route("/stloads/reconciliation", get(stloads_reconciliation))
        .route("/audit", get(audit_search))
        .route("/audit/export", get(audit_export))
        .route("/support/search", get(support_search))
        .route("/support/cases", get(support_cases))
        .route("/support/cases", post(create_support_case))
        .route("/support/cases/{case_id}", post(update_support_case))
        .route(
            "/support/cases/{case_id}/feedback",
            post(record_support_case_feedback),
        )
        .route("/support/timeline", get(support_timeline))
        .route("/support/notes", post(create_support_note))
        .route("/users", get(user_directory))
        .route("/loads", get(load_list))
        .route("/loads/{load_id}/review", post(review_load_handler))
        .route("/users", post(create_user_handler))
        .route("/users/{user_id}/profile", get(user_profile))
        .route(
            "/users/{user_id}/profile",
            post(update_user_profile_handler),
        )
        .route("/users/{user_id}/delete", post(delete_user_handler))
        .route("/break-glass", post(start_break_glass_handler))
        .route("/identity", get(identity_screen))
        .route("/identity/domains", post(upsert_identity_domain))
        .route("/identity/domains/verify", post(verify_identity_domain))
        .route(
            "/identity/domains/verify-dns",
            post(check_identity_domain_dns),
        )
        .route("/identity/providers", post(upsert_identity_provider))
        .route("/access-reviews", get(access_reviews_screen))
        .route("/access-reviews/start", post(start_access_review))
        .route(
            "/access-reviews/elevation-requests",
            post(create_access_elevation_request),
        )
        .route(
            "/access-reviews/elevation-requests/{request_id}/decision",
            post(decide_access_elevation_request),
        )
        .route(
            "/access-reviews/items/{item_id}/decision",
            post(decide_access_review),
        )
        .route("/roles/permissions", get(role_permissions))
        .route(
            "/users/{user_id}/account",
            post(update_user_account_handler),
        )
        .route(
            "/roles/{role_key}/permissions",
            post(update_role_permissions_handler),
        )
        .route("/onboarding-reviews", get(onboarding_reviews))
        .route("/users/{user_id}/review", post(review_user_handler))
        .route(
            "/stloads/sync-errors/{sync_error_id}/resolve",
            post(resolve_sync_error_handler),
        )
        .route("/stloads/tms-conflicts", get(tms_conflict_screen))
        .route(
            "/stloads/tms-conflicts/{conflict_id}/repair",
            post(repair_tms_conflict_handler),
        )
        .route("/integrations/webhooks", get(webhook_delivery_screen))
        .route(
            "/integrations/webhooks/{delivery_id}/replay",
            post(replay_webhook_delivery),
        )
        .route("/integrations/portal", get(integration_portal))
        .route("/integrations/api-keys", post(create_partner_api_key))
        .route(
            "/integrations/webhook-endpoints",
            post(upsert_webhook_endpoint),
        )
        .route("/integrations/edi", get(edi_integration_screen))
        .route(
            "/integrations/edi/partners",
            post(upsert_edi_partner_profile),
        )
        .route(
            "/integrations/edi/messages/validate",
            post(validate_edi_message),
        )
        .route(
            "/integrations/edi/messages/{message_id}/replay",
            post(replay_edi_message),
        )
        .route("/integrations/sandbox", get(sandbox_governance_screen))
        .route("/integrations/sandbox/reset", post(queue_sandbox_reset))
        .route("/integrations/api-lifecycle", get(api_lifecycle_screen))
}

async fn index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminOverview>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_payments",
            "manage_master_data",
            "manage_users",
        ],
    )
    .await?;

    let mut shipper_total = 0_usize;
    let mut carrier_total = 0_usize;
    let mut broker_total = 0_usize;
    let mut freight_forwarder_total = 0_usize;
    let mut admin_total = 0_usize;

    if let Some(pool) = state.pool.as_ref() {
        let organization_id = auth_session::session_organization_id(&session);
        for row in list_admin_users(pool, organization_id)
            .await
            .unwrap_or_default()
        {
            match row.role_id.and_then(UserRole::from_legacy_id) {
                Some(UserRole::Shipper) => shipper_total += 1,
                Some(UserRole::Carrier) => carrier_total += 1,
                Some(UserRole::Broker) => broker_total += 1,
                Some(UserRole::FreightForwarder) => freight_forwarder_total += 1,
                Some(UserRole::Admin) => admin_total += 1,
                None => {}
            }
        }
    }

    Ok(Json(ApiResponse::ok(AdminOverview {
        screen_routes: vec![
            "/admin/users",
            "/admin/roles",
            "/admin/onboarding-reviews",
            "/admin/stloads/operations",
            "/admin/stloads/reconciliation",
            "/admin/payments",
            "/admin/master-data",
            "/admin/loads",
        ],
        operational_views: 8,
        user_total: shipper_total
            + carrier_total
            + broker_total
            + freight_forwarder_total
            + admin_total,
        shipper_total,
        carrier_total,
        broker_total,
        freight_forwarder_total,
        admin_total,
        notes: vec![
            "Admin is now the route home for ops dashboards rather than a single placeholder.",
            "Master-data visibility now lives alongside payments and TMS so load-builder dependencies can be migrated in sequence.",
            "Role-permission editing now uses the same database-backed permission source that live Rust sessions resolve against.",
            "Admin loads now mirrors the Laravel approval and operations tabs inside the Rust admin shell.",
            "IBM-targeted runtime config is environment-driven so these routes can boot on fresh servers without code edits.",
        ],
    })))
}

async fn load_list(
    State(state): State<AppState>,
    Query(query): Query<AdminLoadsQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminLoadListScreen>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_loads"]).await?;

    let active_tab = normalize_admin_load_tab(query.tab.as_deref());
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminLoadListScreen {
            title: "Admin Loads".into(),
            subtitle:
                "Load oversight is unavailable because the Rust database connection is not ready."
                    .into(),
            active_tab: active_tab.clone(),
            tabs: admin_load_tabs(&active_tab, 0, 0, 0, 0, 0),
            rows: Vec::new(),
            notes: vec![format!(
                "Admin load visibility is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
            pagination: shared::Pagination {
                page: 1,
                per_page: 30,
                total: 0,
            },
        })));
    };

    let organization_id = auth_session::session_organization_id(&session);
    let all_count = count_admin_load_legs_filtered(pool, None, organization_id)
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let pending_count = count_admin_load_legs_filtered(pool, Some(&[1]), organization_id)
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let approved_count =
        count_admin_load_legs_filtered(pool, Some(&[2, 3, 4, 5, 6, 8, 9]), organization_id)
            .await
            .unwrap_or_default()
            .max(0) as u64;
    let release_count = count_admin_load_legs_filtered(pool, Some(&[10]), organization_id)
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let completed_count = count_admin_load_legs_filtered(pool, Some(&[11]), organization_id)
        .await
        .unwrap_or_default()
        .max(0) as u64;

    let filter_statuses = admin_load_tab_statuses(&active_tab);
    let rows = list_admin_load_legs_filtered(pool, filter_statuses, organization_id, 30)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(map_admin_load_row)
        .collect::<Vec<_>>();

    Ok(Json(ApiResponse::ok(AdminLoadListScreen {
        title: "Admin Loads".into(),
        subtitle: "Mirror of the Laravel admin load tabs for approval, active execution, completion, and release-readiness.".into(),
        active_tab: active_tab.clone(),
        tabs: admin_load_tabs(
            &active_tab,
            all_count,
            pending_count,
            approved_count,
            completed_count,
            release_count,
        ),
        rows,
        notes: vec![
            "Pending mirrors PHP status 1, approved-active mirrors 2,3,4,5,6,8,9, fund release mirrors 10, and completed mirrors 11.".into(),
            "Profile links now stay inside the Rust admin shell instead of bouncing back to Blade.".into(),
            "Release-ready rows deep-link into the Rust payments console until the final in-row finance actions land.".into(),
        ],
        pagination: shared::Pagination {
            page: 1,
            per_page: 30,
            total: match active_tab.as_str() {
                "pending" => pending_count,
                "approved" => approved_count,
                "completed" => completed_count,
                "release-funds" => release_count,
                _ => all_count,
            },
        },
    })))
}

async fn review_load_handler(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminReviewLoadRequest>,
) -> Result<Json<ApiResponse<AdminReviewLoadResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_loads"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
            success: false,
            load_id,
            status_label: "Unavailable".into(),
            message: format!(
                "Load review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let decision = payload.decision.trim().to_ascii_lowercase();
    let (status_id, status_label) = match decision.as_str() {
        "approve" | "approved" => (2_i16, "Approved"),
        "reject" | "rejected" => (0_i16, "Rejected"),
        "revision" | "revise" | "send_back" | "send-back" => (7_i16, "Revision Requested"),
        _ => {
            return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
                success: false,
                load_id,
                status_label: "Invalid".into(),
                message: "Choose approve, reject, or revision for the Rust load review action."
                    .into(),
            })));
        }
    };

    let remarks = payload
        .remarks
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if matches!(status_id, 0 | 7) && remarks.is_none() {
        return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
            success: false,
            load_id,
            status_label: status_label.into(),
            message: "Remarks are required when rejecting a load or sending it back for revision."
                .into(),
        })));
    }

    let reviewed = review_load_status(pool, load_id, status_id, remarks, Some(session.user.id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(scope) = reviewed else {
        return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
            success: false,
            load_id,
            status_label: "Missing".into(),
            message: "The selected load was not found in the Rust admin review flow.".into(),
        })));
    };

    let summary = match status_id {
        2 => format!(
            "{} approved load #{} from the Rust admin loads page.",
            session.user.name, load_id
        ),
        0 => format!(
            "{} rejected load #{} from the Rust admin loads page.",
            session.user.name, load_id
        ),
        _ => format!(
            "{} returned load #{} for revision from the Rust admin loads page.",
            session.user.name, load_id
        ),
    };

    let mail_note = match scope.owner_user_id {
        Some(owner_user_id) => match find_user_by_id(pool, owner_user_id).await {
            Ok(Some(owner)) => match state
                .email
                .send_load_review_status_with_request_id(
                    &owner.email,
                    &owner.name,
                    load_id,
                    status_id,
                    remarks,
                    request_id_from_headers(&headers).as_deref(),
                )
                .await
            {
                Ok(outcome) => outcome.status_note(),
                Err(error) => Some(format!("Load review email delivery failed: {}", error)),
            },
            Ok(None) => Some("Load owner was not found for email notification.".into()),
            Err(error) => Some(format!("Load owner lookup for email failed: {}", error)),
        },
        None => Some("Load owner was unavailable for email notification.".into()),
    };

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: scope.owner_user_id.map(|value| value.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys([
            "access_admin_portal",
            "manage_loads",
            "manage_dispatch_desk",
        ])
        .with_topics([
            RealtimeTopic::AdminDashboard.as_key(),
            RealtimeTopic::LoadBoard.as_key(),
        ]),
    );

    Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
        success: true,
        load_id,
        status_label: status_label.into(),
        message: append_optional_note(summary, mail_note),
    })))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("admin route group ready"))
}

async fn audit_search(
    State(state): State<AppState>,
    Query(query): Query<AdminAuditSearchQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminAuditSearchScreen>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "view_audit_events", "manage_users"],
    )
    .await?;
    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = resolve_support_target_organization_id(
        query.target_organization_id,
        actor_organization_id,
    )?;

    let filters = audit_filters_from_query(&query);
    let export_path = audit_export_path(&filters);

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminAuditSearchScreen {
            title: "Audit Search".into(),
            target_organization_id: target_organization_id.max(0) as u64,
            filters,
            result_count: 0,
            rows: Vec::new(),
            export_path,
            notes: vec![format!(
                "Audit search is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    ensure_audit_org_access(pool, &session, target_organization_id).await?;
    let rows = query_audit_events(pool, target_organization_id, &query, 200).await?;
    let result_count = rows.len() as u64;

    Ok(Json(ApiResponse::ok(AdminAuditSearchScreen {
        title: "Audit Search".into(),
        target_organization_id: target_organization_id.max(0) as u64,
        filters,
        result_count,
        rows,
        export_path,
        notes: vec![
            "Search covers actor, organization, load, document, payment, TMS handoff, action, request ID, and date filters.".into(),
            "Cross-organization audit search requires an active break-glass session.".into(),
            "Exports use the same filters and are intended for compliance request evidence packs.".into(),
        ],
    })))
}

async fn audit_export(
    State(state): State<AppState>,
    Query(query): Query<AdminAuditSearchQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminAuditExportResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "view_audit_events", "manage_users"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminAuditExportResponse {
            filename: "audit-export-unavailable.csv".into(),
            content_type: "text/csv".into(),
            row_count: 0,
            csv: "id,created_at,action,entity_type,entity_id,actor,request_id,reason\n".into(),
        })));
    };

    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = resolve_support_target_organization_id(
        query.target_organization_id,
        actor_organization_id,
    )?;
    ensure_audit_org_access(pool, &session, target_organization_id).await?;

    let rows = query_audit_events(pool, target_organization_id, &query, 1_000).await?;
    let csv = audit_rows_to_csv(&rows);
    let date = Utc::now().format("%Y%m%d%H%M%S");
    Ok(Json(ApiResponse::ok(AdminAuditExportResponse {
        filename: format!("audit-export-org-{}-{}.csv", target_organization_id, date),
        content_type: "text/csv".into(),
        row_count: rows.len() as u64,
        csv,
    })))
}

async fn support_search(
    State(state): State<AppState>,
    Query(query): Query<AdminSupportSearchQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminSupportSearchScreen>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminSupportSearchScreen {
            title: "Support Search".into(),
            query: query.q.unwrap_or_default(),
            target_organization_id: query.target_organization_id,
            result_count: 0,
            results: Vec::new(),
            notes: vec![format!(
                "Support search is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let search_term = query.q.unwrap_or_default().trim().to_string();
    if search_term.len() < 2 {
        return Ok(Json(ApiResponse::ok(AdminSupportSearchScreen {
            title: "Support Search".into(),
            query: search_term,
            target_organization_id: query.target_organization_id,
            result_count: 0,
            results: Vec::new(),
            notes: vec!["Enter at least two characters to run an audited support search.".into()],
        })));
    }

    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = query
        .target_organization_id
        .map(|value| i64::try_from(value).map_err(|_| StatusCode::BAD_REQUEST))
        .transpose()?
        .or(actor_organization_id)
        .ok_or(StatusCode::FORBIDDEN)?;

    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_organization_id,
        "support_search_cross_org",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let like_query = format!("%{}%", search_term);
    let exact_query = search_term.clone();
    let mut results = Vec::new();

    append_support_organization_results(
        pool,
        target_organization_id,
        &exact_query,
        &like_query,
        &mut results,
    )
    .await?;
    append_support_user_results(
        pool,
        target_organization_id,
        &exact_query,
        &like_query,
        &mut results,
    )
    .await?;
    append_support_load_results(
        pool,
        target_organization_id,
        &exact_query,
        &like_query,
        &mut results,
    )
    .await?;
    append_support_document_results(
        pool,
        target_organization_id,
        &exact_query,
        &like_query,
        &mut results,
    )
    .await?;
    append_support_escrow_results(
        pool,
        target_organization_id,
        &exact_query,
        &like_query,
        &mut results,
    )
    .await?;
    append_support_tms_results(
        pool,
        target_organization_id,
        &exact_query,
        &like_query,
        &mut results,
    )
    .await?;

    let request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: actor_organization_id,
            target_organization_id: Some(target_organization_id),
            entity_type: "support_search",
            entity_id: None,
            action: "support_search_performed",
            reason: Some("audited support lookup"),
            ticket_ref: None,
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "query_length": search_term.len(),
                "result_count": results.len(),
                "categories": support_result_categories(&results),
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result_count = results.len() as u64;
    Ok(Json(ApiResponse::ok(AdminSupportSearchScreen {
        title: "Support Search".into(),
        query: search_term,
        target_organization_id: Some(target_organization_id.max(0) as u64),
        result_count,
        results,
        notes: vec![
            "Results are scoped to the session organization unless a valid break-glass session authorizes the requested organization.".into(),
            "Every executed search writes an audit event with the actor, target organization, result count, and result categories.".into(),
        ],
    })))
}

async fn support_cases(
    State(state): State<AppState>,
    Query(query): Query<AdminSupportCaseQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminSupportCaseScreen>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminSupportCaseScreen {
            title: "Support Cases".into(),
            target_organization_id: query.target_organization_id.unwrap_or_default(),
            open_count: 0,
            breach_risk_count: 0,
            breached_count: 0,
            rows: Vec::new(),
            notes: vec![format!(
                "Support cases are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = resolve_support_target_organization_id(
        query.target_organization_id,
        actor_organization_id,
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;
    let status = clean_optional(query.status)
        .map(|value| normalize_support_case_status(&value))
        .transpose()?;

    let screen = load_support_case_screen(pool, target_organization_id, status.as_deref()).await?;
    Ok(Json(ApiResponse::ok(screen)))
}

async fn create_support_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminCreateSupportCaseRequest>,
) -> Result<Json<ApiResponse<AdminSupportCaseMutationResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminSupportCaseMutationResponse {
            success: false,
            message: format!(
                "Support case creation is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            case_row: None,
            screen: empty_support_case_screen(payload.target_organization_id),
        })));
    };

    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        actor_organization_id,
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;

    let severity = normalize_support_case_severity(&payload.severity)?;
    let channel = normalize_support_case_key(&payload.channel, "portal")?;
    let category = normalize_support_case_key(&payload.category, "general")?;
    let owner_team = non_empty_trimmed(&payload.owner_team, "Support")?;
    let title = non_empty_trimmed(&payload.title, "")?;
    let description = non_empty_trimmed(&payload.description, "")?;
    let customer_impact = non_empty_trimmed(&payload.customer_impact, "")?;
    let related_entity_type = payload
        .related_entity_type
        .as_deref()
        .map(normalize_support_entity_type)
        .transpose()?;
    let related_entity_id = clean_optional(payload.related_entity_id.clone());
    let now = Utc::now().naive_utc();
    let (first_response_due_at, next_update_due_at, resolution_due_at) =
        support_case_sla_deadlines(now, &severity);
    let case_number = format!(
        "SUP-{}-{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        &Uuid::new_v4().simple().to_string()[..6]
    );

    let row = sqlx::query(
        "INSERT INTO support_cases (
             case_number, organization_id, reporter_user_id, affected_user_id,
             related_entity_type, related_entity_id, channel, severity, status,
             owner_team, category, customer_impact, title, description,
             first_response_due_at, next_update_due_at, resolution_due_at,
             breach_state, created_by_user_id, updated_by_user_id, created_at, updated_at
         ) VALUES (
             $1, $2, $3, $4, $5, $6, $7, $8, 'new',
             $9, $10, $11, $12, $13, $14, $15, $16,
             'ok', $17, $17, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(case_number)
    .bind(target_organization_id)
    .bind(optional_u64_to_i64(payload.reporter_user_id)?)
    .bind(optional_u64_to_i64(payload.affected_user_id)?)
    .bind(related_entity_type.as_deref())
    .bind(related_entity_id.as_deref())
    .bind(channel)
    .bind(&severity)
    .bind(owner_team)
    .bind(category)
    .bind(customer_impact)
    .bind(title)
    .bind(description)
    .bind(first_response_due_at)
    .bind(next_update_due_at)
    .bind(resolution_due_at)
    .bind(session.user.id)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let case_id: i64 = row.get("id");

    insert_support_case_event(
        case_id,
        target_organization_id,
        Some(session.user.id),
        SupportCaseEventInput {
            event_type: "created",
            visibility: "internal",
            previous_status: None,
            new_status: Some("new"),
            note: payload
                .internal_note
                .as_deref()
                .or(Some("Support case opened.")),
            customer_update: payload.customer_update.as_deref(),
            internal_note: payload.internal_note.as_deref(),
            feedback_score: None,
        },
        pool,
    )
    .await?;
    audit_support_case_action(
        pool,
        &session,
        target_organization_id,
        case_id,
        "support_case_created",
        "support case created",
        request_id_from_headers(&headers).as_deref(),
    )
    .await?;

    support_case_mutation_response(
        pool,
        target_organization_id,
        case_id,
        "Support case created.",
    )
    .await
}

async fn update_support_case(
    State(state): State<AppState>,
    Path(case_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateSupportCaseRequest>,
) -> Result<Json<ApiResponse<AdminSupportCaseMutationResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminSupportCaseMutationResponse {
            success: false,
            message: format!(
                "Support case update is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            case_row: None,
            screen: empty_support_case_screen(None),
        })));
    };

    let Some(existing) = load_support_case_row_by_id(pool, case_id).await? else {
        return Err(StatusCode::NOT_FOUND);
    };
    let target_organization_id = existing.organization_id as i64;
    ensure_support_org_access(pool, &session, target_organization_id).await?;

    let status = payload
        .status
        .as_deref()
        .map(normalize_support_case_status)
        .transpose()?
        .unwrap_or_else(|| existing.status.clone());
    let severity = payload
        .severity
        .as_deref()
        .map(normalize_support_case_severity)
        .transpose()?
        .unwrap_or_else(|| existing.severity.clone());
    let owner_team = payload
        .owner_team
        .as_deref()
        .map(|value| non_empty_trimmed(value, "Support"))
        .transpose()?
        .unwrap_or_else(|| existing.owner_team.clone());
    let now = Utc::now().naive_utc();
    let breach_state = support_case_breach_state(
        now,
        &existing.first_response_due_at,
        &existing.resolution_due_at,
    );
    let resolution_reason =
        clean_optional(payload.resolution_reason.clone()).or(existing.resolution_reason);
    let root_cause_category = clean_optional(payload.root_cause_category.clone());
    let follow_up_action = clean_optional(payload.follow_up_action.clone());

    sqlx::query(
        "UPDATE support_cases
         SET status = $1,
             severity = $2,
             owner_team = $3,
             owner_user_id = $4,
             escalation_owner_user_id = $5,
             first_responded_at = CASE WHEN first_responded_at IS NULL AND $6 IS NOT NULL THEN CURRENT_TIMESTAMP ELSE first_responded_at END,
             resolved_at = CASE WHEN $1 IN ('resolved', 'closed') AND resolved_at IS NULL THEN CURRENT_TIMESTAMP ELSE resolved_at END,
             closed_at = CASE WHEN $1 = 'closed' AND closed_at IS NULL THEN CURRENT_TIMESTAMP ELSE closed_at END,
             breach_state = $7,
             resolution_reason = COALESCE($8, resolution_reason),
             root_cause_category = COALESCE($9, root_cause_category),
             follow_up_action = COALESCE($10, follow_up_action),
             updated_by_user_id = $11,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $12",
    )
    .bind(&status)
    .bind(&severity)
    .bind(owner_team)
    .bind(optional_u64_to_i64(payload.owner_user_id)?)
    .bind(optional_u64_to_i64(payload.escalation_owner_user_id)?)
    .bind(payload.customer_update.as_deref())
    .bind(breach_state)
    .bind(resolution_reason.as_deref())
    .bind(root_cause_category.as_deref())
    .bind(follow_up_action.as_deref())
    .bind(session.user.id)
    .bind(case_id)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let visibility = if payload
        .customer_update
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        "customer_visible"
    } else {
        "internal"
    };
    insert_support_case_event(
        case_id,
        target_organization_id,
        Some(session.user.id),
        SupportCaseEventInput {
            event_type: "updated",
            visibility,
            previous_status: Some(existing.status.as_str()),
            new_status: Some(status.as_str()),
            note: payload
                .customer_update
                .as_deref()
                .or(payload.internal_note.as_deref())
                .or(Some("Support case updated.")),
            customer_update: payload.customer_update.as_deref(),
            internal_note: payload.internal_note.as_deref(),
            feedback_score: None,
        },
        pool,
    )
    .await?;
    audit_support_case_action(
        pool,
        &session,
        target_organization_id,
        case_id,
        "support_case_updated",
        "support case updated",
        request_id_from_headers(&headers).as_deref(),
    )
    .await?;

    support_case_mutation_response(
        pool,
        target_organization_id,
        case_id,
        "Support case updated.",
    )
    .await
}

async fn record_support_case_feedback(
    State(state): State<AppState>,
    Path(case_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminSupportCaseFeedbackRequest>,
) -> Result<Json<ApiResponse<AdminSupportCaseMutationResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    if !(1..=5).contains(&payload.feedback_score) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminSupportCaseMutationResponse {
            success: false,
            message: format!(
                "Support case feedback is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            case_row: None,
            screen: empty_support_case_screen(None),
        })));
    };
    let Some(existing) = load_support_case_row_by_id(pool, case_id).await? else {
        return Err(StatusCode::NOT_FOUND);
    };
    let target_organization_id = existing.organization_id as i64;
    ensure_support_org_access(pool, &session, target_organization_id).await?;

    sqlx::query(
        "UPDATE support_cases
         SET feedback_score = $1,
             feedback_comment = $2,
             feedback_received_at = CURRENT_TIMESTAMP,
             updated_by_user_id = $3,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4",
    )
    .bind(payload.feedback_score)
    .bind(clean_optional(payload.feedback_comment.clone()).as_deref())
    .bind(session.user.id)
    .bind(case_id)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    insert_support_case_event(
        case_id,
        target_organization_id,
        Some(session.user.id),
        SupportCaseEventInput {
            event_type: "feedback_recorded",
            visibility: "internal",
            previous_status: Some(existing.status.as_str()),
            new_status: Some(existing.status.as_str()),
            note: payload
                .feedback_comment
                .as_deref()
                .or(Some("Customer feedback recorded.")),
            customer_update: None,
            internal_note: payload.feedback_comment.as_deref(),
            feedback_score: Some(payload.feedback_score),
        },
        pool,
    )
    .await?;
    audit_support_case_action(
        pool,
        &session,
        target_organization_id,
        case_id,
        "support_case_feedback_recorded",
        "support case feedback recorded",
        request_id_from_headers(&headers).as_deref(),
    )
    .await?;

    support_case_mutation_response(
        pool,
        target_organization_id,
        case_id,
        "Support case feedback recorded.",
    )
    .await
}

async fn support_timeline(
    State(state): State<AppState>,
    Query(query): Query<AdminSupportTimelineRequest>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminSupportTimelineScreen>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminSupportTimelineScreen {
            title: "Support Timeline".into(),
            target_organization_id: query.target_organization_id.unwrap_or_default(),
            entity_type: query.entity_type,
            entity_id: query.entity_id,
            entries: Vec::new(),
            notes: vec![format!(
                "Support timeline is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = resolve_support_target_organization_id(
        query.target_organization_id,
        actor_organization_id,
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;

    let timeline = load_support_timeline(
        pool,
        target_organization_id,
        &query.entity_type,
        query.entity_id.as_deref(),
    )
    .await?;

    Ok(Json(ApiResponse::ok(timeline)))
}

async fn create_support_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminCreateSupportNoteRequest>,
) -> Result<Json<ApiResponse<AdminCreateSupportNoteResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_users", "view_audit_events"],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminCreateSupportNoteResponse {
            success: false,
            note_id: None,
            message: format!(
                "Support notes are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            timeline: AdminSupportTimelineScreen {
                title: "Support Timeline".into(),
                target_organization_id: payload.target_organization_id.unwrap_or_default(),
                entity_type: payload.entity_type,
                entity_id: payload.entity_id,
                entries: Vec::new(),
                notes: Vec::new(),
            },
        })));
    };

    let actor_organization_id = auth_session::session_organization_id(&session);
    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        actor_organization_id,
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;

    let entity_type = normalize_support_entity_type(&payload.entity_type)?;
    let entity_id = payload
        .entity_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let visibility = normalize_support_note_visibility(&payload.visibility)?;
    let note = payload.note.trim();
    if note.len() < 4 {
        let timeline = load_support_timeline(
            pool,
            target_organization_id,
            &entity_type,
            entity_id.as_deref(),
        )
        .await?;
        return Ok(Json(ApiResponse::ok(AdminCreateSupportNoteResponse {
            success: false,
            note_id: None,
            message: "Support notes must include at least four characters.".into(),
            timeline,
        })));
    }

    let ticket_ref = payload
        .ticket_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let note_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO support_notes (
            organization_id, actor_user_id, entity_type, entity_id, visibility, ticket_ref, note, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(target_organization_id)
    .bind(session.user.id)
    .bind(&entity_type)
    .bind(entity_id.as_deref())
    .bind(&visibility)
    .bind(ticket_ref.as_deref())
    .bind(note)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: actor_organization_id,
            target_organization_id: Some(target_organization_id),
            entity_type: &entity_type,
            entity_id: entity_id.as_deref(),
            action: "support_note_created",
            reason: Some("support note added"),
            ticket_ref: ticket_ref.as_deref(),
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "support_note_id": note_id,
                "visibility": visibility,
                "note_length": note.len(),
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let timeline = load_support_timeline(
        pool,
        target_organization_id,
        &entity_type,
        entity_id.as_deref(),
    )
    .await?;

    Ok(Json(ApiResponse::ok(AdminCreateSupportNoteResponse {
        success: true,
        note_id: Some(note_id.max(0) as u64),
        message: "Support note saved and audit event recorded.".into(),
        timeline,
    })))
}

async fn onboarding_reviews(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminOnboardingReviewScreen>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminOnboardingReviewScreen {
            title: "Onboarding Reviews".into(),
            summary: "Database unavailable".into(),
            users: Vec::new(),
            notes: vec![format!(
                "Admin review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let users =
        list_pending_onboarding_users(pool, auth_session::session_organization_id(&session))
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|row| async move {
                let documents = list_kyc_documents_by_user_id(pool, row.user_id)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|document| KycDocumentItem {
                        blockchain_label: if document
                            .document_type
                            .eq_ignore_ascii_case("blockchain")
                        {
                            Some("SHA-256 hash stored".into())
                        } else {
                            None
                        },
                        blockchain_tone: if document
                            .document_type
                            .eq_ignore_ascii_case("blockchain")
                        {
                            Some("success".into())
                        } else {
                            None
                        },
                        blockchain_hash_preview: document.hash.as_ref().map(|hash| {
                            if hash.len() > 24 {
                                format!("{}...", &hash[..24])
                            } else {
                                hash.clone()
                            }
                        }),
                        blockchain_hash: document.hash.clone(),
                        id: document.id.max(0) as u64,
                        document_name: document.document_name,
                        document_type: document.document_type,
                        file_label: document
                            .original_name
                            .clone()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| "KYC file".into()),
                        original_name: document.original_name,
                        mime_type: document.mime_type,
                        file_size_bytes: document
                            .file_size
                            .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
                        uploaded_at_label: format_datetime(&document.created_at),
                        current_version: document.current_version.max(1) as u32,
                        version_count: document.version_count.max(1) as u64,
                        version_history_label: document_version_label(
                            document.current_version,
                            document.version_count,
                        ),
                        download_path: Some(format!(
                            "/auth/onboarding/documents/{}/file",
                            document.id.max(0) as u64
                        )),
                        can_view_file: true,
                        can_edit: false,
                        can_verify_blockchain: false,
                        can_delete: false,
                    })
                    .collect::<Vec<_>>();

                AdminOnboardingReviewUser {
                    user_id: row.user_id.max(0) as u64,
                    name: row.name,
                    email: row.email,
                    role_label: row
                        .role_id
                        .and_then(UserRole::from_legacy_id)
                        .map(|role| role.label().to_string())
                        .unwrap_or_else(|| "Unknown".into()),
                    status_label: AccountStatus::from_legacy_code(row.status)
                        .map(account_status_label)
                        .unwrap_or_else(|| format!("Status {}", row.status)),
                    company_name: row.company_name,
                    company_address: row.company_address,
                    submitted_at_label: row
                        .submitted_at
                        .as_ref()
                        .map(format_datetime)
                        .unwrap_or_else(|| "Not submitted".into()),
                    document_count: row.document_count.max(0) as u64,
                    documents,
                }
            });

    let mut review_users = Vec::new();
    for future in users {
        review_users.push(future.await);
    }

    Ok(Json(ApiResponse::ok(AdminOnboardingReviewScreen {
        title: "Onboarding Reviews".into(),
        summary: format!("{} account(s) currently need review or revision handling.", review_users.len()),
        users: review_users,
        notes: vec![
            "Uploaded KYC files are protected by the same Rust document access rule: admin users and the uploader can open them.".into(),
            "Approve moves the account to Approved, Request Revision returns it to Revision Requested, and Reject moves it to Rejected.".into(),
        ],
    })))
}

async fn webhook_delivery_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WebhookDeliveryQuery>,
) -> Result<Json<ApiResponse<WebhookDeliveryScreen>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_dispatch_desk",
        ],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let deliveries = list_webhook_delivery_logs(pool, organization_id, status, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|delivery| WebhookDeliveryRow {
            id: delivery.id.max(0) as u64,
            endpoint_id: delivery.endpoint_id.map(|value| value.max(0) as u64),
            event_type: delivery.event_type,
            event_id: delivery.event_id,
            delivery_status: delivery.delivery_status,
            attempt_count: delivery.attempt_count,
            next_retry_at: delivery.next_retry_at.map(|value| value.to_string()),
            last_attempt_at: delivery.last_attempt_at.map(|value| value.to_string()),
            response_status_code: delivery.response_status_code,
            response_latency_ms: delivery.response_latency_ms,
            response_body_excerpt: delivery.response_body_excerpt,
            dead_letter_reason: delivery.dead_letter_reason,
            replay_of_delivery_id: delivery
                .replay_of_delivery_id
                .map(|value| value.max(0) as u64),
            created_at: delivery.created_at.to_string(),
        })
        .collect();

    Ok(Json(ApiResponse::ok(WebhookDeliveryScreen {
        organization_id: organization_id.max(0) as u64,
        status_filter: status.map(str::to_string),
        deliveries,
    })))
}

async fn replay_webhook_delivery(
    State(state): State<AppState>,
    Path(delivery_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<WebhookReplayResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_dispatch_desk",
        ],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let replay = mark_webhook_delivery_for_replay(pool, organization_id, delivery_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(match replay {
        Some(row) => WebhookReplayResponse {
            success: true,
            delivery_id: delivery_id.max(0) as u64,
            replay_delivery_id: Some(row.id.max(0) as u64),
            message: "Webhook delivery replay has been queued.".into(),
        },
        None => WebhookReplayResponse {
            success: false,
            delivery_id: delivery_id.max(0) as u64,
            replay_delivery_id: None,
            message: "Webhook delivery was not found for this organization.".into(),
        },
    })))
}

async fn tms_conflict_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<TmsConflictScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_dispatch_desk",
        ],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let rules = list_tms_source_of_truth_rules(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|rule| TmsSourceRuleRow {
            field_key: rule.field_key,
            owning_system: rule.owning_system,
            conflict_policy: rule.conflict_policy,
            default_repair_action: rule.default_repair_action,
            notes: rule.notes,
        })
        .collect();

    let conflicts = list_open_tms_conflicts(pool, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|conflict| TmsConflictRow {
            id: conflict.id.max(0) as u64,
            handoff_id: conflict.handoff_id.max(0) as u64,
            field_key: conflict.field_key,
            source_of_truth: conflict.source_of_truth,
            stloads_value: conflict.stloads_value,
            tms_value: conflict.tms_value,
            conflict_status: conflict.conflict_status,
            severity: conflict.severity,
            repair_action: conflict.repair_action,
            detected_by: conflict.detected_by,
            created_at: conflict.created_at.to_string(),
        })
        .collect();

    Ok(Json(ApiResponse::ok(TmsConflictScreen {
        source_of_truth_rules: rules,
        conflicts,
    })))
}

async fn repair_tms_conflict_handler(
    State(state): State<AppState>,
    Path(conflict_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<RepairTmsConflictRequest>,
) -> Result<Json<ApiResponse<RepairTmsConflictResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_dispatch_desk",
        ],
    )
    .await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let repaired = repair_tms_conflict(
        pool,
        conflict_id,
        &session.user.email,
        payload.resolution_note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(match repaired {
        Some(conflict) => RepairTmsConflictResponse {
            success: true,
            conflict_id: conflict.id.max(0) as u64,
            status: Some(conflict.conflict_status),
            message: "TMS conflict repair action completed and recorded.".into(),
        },
        None => RepairTmsConflictResponse {
            success: false,
            conflict_id: conflict_id.max(0) as u64,
            status: None,
            message: "TMS conflict was not found.".into(),
        },
    })))
}

async fn integration_portal(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<IntegrationPortalScreen>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    Ok(Json(ApiResponse::ok(
        load_integration_portal_screen(pool, organization_id).await?,
    )))
}

async fn create_partner_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePartnerApiKeyRequest>,
) -> Result<Json<ApiResponse<CreatePartnerApiKeyResponse>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let client_name = payload.client_name.trim();
    if client_name.len() < 3 {
        return Ok(Json(ApiResponse::ok(CreatePartnerApiKeyResponse {
            success: false,
            api_key: None,
            key_prefix: None,
            message: "Enter a client name with at least 3 characters.".into(),
        })));
    }
    let scopes = normalize_partner_scopes(payload.scopes);
    if scopes.is_empty() {
        return Ok(Json(ApiResponse::ok(CreatePartnerApiKeyResponse {
            success: false,
            api_key: None,
            key_prefix: None,
            message: "Choose at least one API scope before creating a partner key.".into(),
        })));
    }
    let expires_at = parse_optional_datetime_for_admin(payload.expires_at.as_deref())?;
    let prefix = format!("pk{}", &Uuid::new_v4().simple().to_string()[..10]);
    let secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let api_key = format!("stlp_{}.{}", prefix, secret);
    let key_hash = sha256_hex(secret.as_bytes());
    let rate_limit = payload.rate_limit_per_minute.unwrap_or(60).clamp(1, 10_000);
    let require_signature = payload.require_request_signature.unwrap_or(true);

    sqlx::query(
        "INSERT INTO partner_api_clients (
             organization_id, actor_user_id, client_name, key_prefix, key_hash, scopes,
             status, rate_limit_per_minute, require_request_signature, expires_at,
             created_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, $8, $9, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(organization_id)
    .bind(session.user.id)
    .bind(client_name)
    .bind(&prefix)
    .bind(&key_hash)
    .bind(&scopes)
    .bind(rate_limit)
    .bind(require_signature)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(CreatePartnerApiKeyResponse {
        success: true,
        api_key: Some(api_key),
        key_prefix: Some(prefix),
        message: "Partner API key created. Store the secret now; it will not be shown again."
            .into(),
    })))
}

async fn upsert_webhook_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertWebhookEndpointRequest>,
) -> Result<Json<ApiResponse<IntegrationMutationResponse>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let endpoint_name = payload.endpoint_name.trim();
    let target_url = payload.target_url.trim();
    if endpoint_name.len() < 3 || !target_url_is_allowed(target_url) {
        return Ok(Json(ApiResponse::ok(IntegrationMutationResponse {
            success: false,
            message: "Enter an endpoint name and an HTTPS webhook URL.".into(),
        })));
    }
    let event_types = normalize_partner_scopes(payload.event_types);
    if event_types.is_empty() {
        return Ok(Json(ApiResponse::ok(IntegrationMutationResponse {
            success: false,
            message: "Choose at least one webhook event type.".into(),
        })));
    }
    let status = payload
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| matches!(*value, "active" | "paused" | "disabled"))
        .unwrap_or("active");

    sqlx::query(
        "INSERT INTO outbound_webhook_endpoints (
             organization_id, endpoint_name, target_url, event_types, status,
             created_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, endpoint_name)
         DO UPDATE SET
             target_url = EXCLUDED.target_url,
             event_types = EXCLUDED.event_types,
             status = EXCLUDED.status,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(organization_id)
    .bind(endpoint_name)
    .bind(target_url)
    .bind(&event_types)
    .bind(status)
    .bind(session.user.id)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(IntegrationMutationResponse {
        success: true,
        message: "Webhook endpoint saved for this organization.".into(),
    })))
}

async fn edi_integration_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<EdiIntegrationScreen>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    Ok(Json(ApiResponse::ok(
        load_edi_integration_screen(pool, organization_id).await?,
    )))
}

async fn upsert_edi_partner_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertEdiPartnerProfileRequest>,
) -> Result<Json<ApiResponse<IntegrationMutationResponse>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let partner_name = payload.partner_name.trim();
    let supported_transactions = normalize_partner_scopes(payload.supported_transactions);
    if partner_name.len() < 3 || supported_transactions.is_empty() {
        return Ok(Json(ApiResponse::ok(IntegrationMutationResponse {
            success: false,
            message: "Enter a partner name and at least one supported EDI transaction.".into(),
        })));
    }
    let transport_type = payload
        .transport_type
        .as_deref()
        .map(str::trim)
        .filter(|value| matches!(*value, "sftp" | "as2" | "api" | "manual"))
        .unwrap_or("sftp");
    let status = payload
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| matches!(*value, "draft" | "active" | "paused" | "disabled"))
        .unwrap_or("draft");
    let validation_mode = payload
        .validation_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| matches!(*value, "strict" | "warn" | "sandbox"))
        .unwrap_or("strict");

    sqlx::query(
        "INSERT INTO edi_partner_profiles (
             organization_id, partner_name, isa_id, gs_id, transport_type, status,
             supported_transactions, validation_mode, created_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, partner_name)
         DO UPDATE SET
             isa_id = EXCLUDED.isa_id,
             gs_id = EXCLUDED.gs_id,
             transport_type = EXCLUDED.transport_type,
             status = EXCLUDED.status,
             supported_transactions = EXCLUDED.supported_transactions,
             validation_mode = EXCLUDED.validation_mode,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(organization_id)
    .bind(partner_name)
    .bind(payload.isa_id.and_then(non_empty_string))
    .bind(payload.gs_id.and_then(non_empty_string))
    .bind(transport_type)
    .bind(status)
    .bind(&supported_transactions)
    .bind(validation_mode)
    .bind(session.user.id)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(IntegrationMutationResponse {
        success: true,
        message: "EDI partner profile saved with transaction and validation policy.".into(),
    })))
}

async fn validate_edi_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidateEdiMessageRequest>,
) -> Result<Json<ApiResponse<EdiValidationResponse>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let transaction_code = payload.transaction_code.trim().to_ascii_uppercase();
    let direction = payload.direction.trim().to_ascii_lowercase();
    if !matches!(direction.as_str(), "inbound" | "outbound") || transaction_code.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mapping = sqlx::query(
        "SELECT required_fields
         FROM edi_transaction_mappings
         WHERE (organization_id = $1 OR organization_id IS NULL)
           AND transaction_code = $2
           AND (direction = $3 OR direction = 'bidirectional')
           AND status = 'active'
         ORDER BY organization_id NULLS LAST
         LIMIT 1",
    )
    .bind(organization_id)
    .bind(&transaction_code)
    .bind(&direction)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(mapping) = mapping else {
        return Ok(Json(ApiResponse::ok(EdiValidationResponse {
            success: false,
            message_id: None,
            missing_fields: vec!["active_mapping".into()],
            ack_status: "rejected_997".into(),
            message: "No active EDI mapping exists for this transaction and direction.".into(),
        })));
    };

    let required_fields = mapping.get::<Vec<String>, _>("required_fields");
    let payload_excerpt = payload.payload_excerpt.unwrap_or_default();
    let payload_lower = payload_excerpt.to_ascii_lowercase();
    let missing_fields = required_fields
        .into_iter()
        .filter(|field| !payload_lower.contains(&field.to_ascii_lowercase()))
        .collect::<Vec<_>>();
    let success = missing_fields.is_empty();
    let message_status = if success { "validated" } else { "failed" };
    let ack_status = if success {
        "generated_997"
    } else {
        "rejected_997"
    };
    let error_summary = if success {
        None
    } else {
        Some(format!(
            "Missing required EDI field(s): {}",
            missing_fields.join(", ")
        ))
    };

    let row = sqlx::query(
        "INSERT INTO edi_message_logs (
             organization_id, transaction_code, direction, control_number, business_key,
             message_status, ack_status, error_summary, payload_excerpt, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, transaction_code, direction, control_number)
         WHERE control_number IS NOT NULL
         DO UPDATE SET
             business_key = EXCLUDED.business_key,
             message_status = EXCLUDED.message_status,
             ack_status = EXCLUDED.ack_status,
             error_summary = EXCLUDED.error_summary,
             payload_excerpt = EXCLUDED.payload_excerpt,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(organization_id)
    .bind(&transaction_code)
    .bind(&direction)
    .bind(payload.control_number.and_then(non_empty_string))
    .bind(payload.business_key.and_then(non_empty_string))
    .bind(message_status)
    .bind(ack_status)
    .bind(error_summary)
    .bind(payload_excerpt.chars().take(2000).collect::<String>())
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(EdiValidationResponse {
        success,
        message_id: Some(row.get::<i64, _>("id").max(0) as u64),
        missing_fields,
        ack_status: ack_status.into(),
        message: if success {
            "EDI message validated and 997 acknowledgement state recorded.".into()
        } else {
            "EDI message failed validation and is visible for repair or replay.".into()
        },
    })))
}

async fn replay_edi_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(message_id): Path<i64>,
) -> Result<Json<ApiResponse<EdiReplayResponse>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);

    let row = sqlx::query(
        "INSERT INTO edi_message_logs (
             organization_id, partner_profile_id, transaction_code, direction, control_number,
             business_key, message_status, ack_status, retry_count, payload_excerpt,
             replay_of_message_id, created_at, updated_at
         )
         SELECT organization_id, partner_profile_id, transaction_code, direction, NULL,
                business_key, 'replay_queued', 'pending_997', 0, payload_excerpt,
                id, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         FROM edi_message_logs
         WHERE id = $1 AND organization_id = $2
           AND message_status IN ('failed', 'dead_letter', 'retrying', 'validated', 'mapped')
         RETURNING id",
    )
    .bind(message_id)
    .bind(organization_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(match row {
        Some(row) => EdiReplayResponse {
            success: true,
            message_id: message_id.max(0) as u64,
            replay_message_id: Some(row.get::<i64, _>("id").max(0) as u64),
            message: "EDI message replay has been queued for integration processing.".into(),
        },
        None => EdiReplayResponse {
            success: false,
            message_id: message_id.max(0) as u64,
            replay_message_id: None,
            message: "EDI message was not found or is not replayable.".into(),
        },
    })))
}

async fn sandbox_governance_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SandboxGovernanceScreen>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    ensure_default_sandbox_environment(pool, organization_id).await?;
    Ok(Json(ApiResponse::ok(
        load_sandbox_governance_screen(pool, organization_id).await?,
    )))
}

async fn queue_sandbox_reset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<QueueSandboxResetRequest>,
) -> Result<Json<ApiResponse<SandboxResetResponse>>, StatusCode> {
    let session = require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let sandbox_environment_id = payload.sandbox_environment_id as i64;

    let environment = sqlx::query(
        "SELECT id, pii_allowed, production_payment_blocked, production_tms_push_blocked,
                production_notification_blocked, reset_status
         FROM sandbox_tenant_environments
         WHERE id = $1 AND organization_id = $2",
    )
    .bind(sandbox_environment_id)
    .bind(organization_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(environment) = environment else {
        return Ok(Json(ApiResponse::ok(SandboxResetResponse {
            success: false,
            reset_job_id: None,
            message: "Sandbox environment was not found for this organization.".into(),
        })));
    };

    let safe = !environment.get::<bool, _>("pii_allowed")
        && environment.get::<bool, _>("production_payment_blocked")
        && environment.get::<bool, _>("production_tms_push_blocked")
        && environment.get::<bool, _>("production_notification_blocked")
        && environment.get::<String, _>("reset_status") != "disabled";
    if !safe {
        return Ok(Json(ApiResponse::ok(SandboxResetResponse {
            success: false,
            reset_job_id: None,
            message: "Sandbox reset refused because safety controls are not all enabled.".into(),
        })));
    }

    let safety_checks = serde_json::json!({
        "pii_allowed": false,
        "production_payment_blocked": true,
        "production_tms_push_blocked": true,
        "production_notification_blocked": true
    });
    let reset_reason = payload.reset_reason.and_then(non_empty_string);
    let row = sqlx::query(
        "INSERT INTO sandbox_reset_jobs (
             organization_id, sandbox_environment_id, requested_by_user_id, reset_reason,
             job_status, safety_checks, created_at
         )
         VALUES ($1, $2, $3, $4, 'queued', $5, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(organization_id)
    .bind(sandbox_environment_id)
    .bind(session.user.id)
    .bind(reset_reason)
    .bind(safety_checks)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        "UPDATE sandbox_tenant_environments
         SET reset_status = 'reset_queued', updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(sandbox_environment_id)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(SandboxResetResponse {
        success: true,
        reset_job_id: Some(row.get::<i64, _>("id").max(0) as u64),
        message: "Sandbox reset queued after safety checks passed.".into(),
    })))
}

async fn api_lifecycle_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ApiLifecycleScreen>>, StatusCode> {
    require_integration_admin(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    Ok(Json(ApiResponse::ok(
        load_api_lifecycle_screen(pool).await?,
    )))
}

async fn user_directory(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminUserDirectoryScreen>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUserDirectoryScreen {
            title: "User Directory".into(),
            summary: "Database unavailable".into(),
            role_options: admin_role_options(),
            status_options: admin_status_options(),
            users: Vec::new(),
            notes: vec![format!(
                "User management is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let users = list_admin_users(pool, auth_session::session_organization_id(&session))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| AdminUserDirectoryUser {
            user_id: row.user_id.max(0) as u64,
            name: row.name,
            email: row.email,
            role_key: row
                .role_id
                .and_then(UserRole::from_legacy_id)
                .map(admin_role_key)
                .unwrap_or_else(|| "unknown".into()),
            role_label: row
                .role_id
                .and_then(UserRole::from_legacy_id)
                .map(|role| role.label().to_string())
                .unwrap_or_else(|| "Unknown".into()),
            status_key: AccountStatus::from_legacy_code(row.status)
                .map(admin_status_key)
                .unwrap_or_else(|| "unknown".into()),
            status_label: AccountStatus::from_legacy_code(row.status)
                .map(account_status_label)
                .unwrap_or_else(|| format!("Status {}", row.status)),
            company_name: row.company_name,
            phone_no: row.phone_no,
            joined_at_label: format_datetime(&row.joined_at),
            document_count: row.document_count.max(0) as u64,
            latest_review_note: row.latest_review_note,
        })
        .collect::<Vec<_>>();

    Ok(Json(ApiResponse::ok(AdminUserDirectoryScreen {
        title: "User Directory".into(),
        summary: format!(
            "{} user account(s) are currently available in the Rust admin directory.",
            users.len()
        ),
        role_options: admin_role_options(),
        status_options: admin_status_options(),
        users,
        notes: vec![
            "Role changes update both users.role_id and the Spatie role pivot so Rust session permissions stay consistent.".into(),
            "Status changes are written to user_history so account review context is preserved alongside onboarding actions.".into(),
        ],
    })))
}

async fn user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminUserProfileScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUserProfileScreen {
            user_id: user_id.max(0) as u64,
            name: "Unavailable".into(),
            email: String::new(),
            role_key: "unknown".into(),
            role_label: "Unknown".into(),
            status_key: "unknown".into(),
            status_label: "Unavailable".into(),
            phone_no: None,
            address: None,
            dob_label: None,
            gender: None,
            joined_at_label: "Unavailable".into(),
            company_name: None,
            company_address: None,
            image_path: None,
            personal_facts: Vec::new(),
            company_facts: Vec::new(),
            documents: Vec::new(),
            history: Vec::new(),
            notes: vec![format!(
                "User profile is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let Some(user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    if !ensure_admin_can_access_organization(
        pool,
        &_session,
        user.organization_id,
        "admin_user_profile_viewed",
        "user",
        Some(user.id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let details = find_user_detail_by_user_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let documents = list_kyc_documents_by_user_id(pool, user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|document| KycDocumentItem {
            blockchain_label: if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("SHA-256 hash stored".into())
            } else {
                None
            },
            blockchain_tone: if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("success".into())
            } else {
                None
            },
            blockchain_hash_preview: document.hash.as_ref().map(|hash| {
                if hash.len() > 24 {
                    format!("{}...", &hash[..24])
                } else {
                    hash.clone()
                }
            }),
            blockchain_hash: document.hash.clone(),
            id: document.id.max(0) as u64,
            document_name: document.document_name,
            document_type: document.document_type,
            file_label: document
                .original_name
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "KYC file".into()),
            original_name: document.original_name,
            mime_type: document.mime_type,
            file_size_bytes: document
                .file_size
                .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
            uploaded_at_label: format_datetime(&document.created_at),
            current_version: document.current_version.max(1) as u32,
            version_count: document.version_count.max(1) as u64,
            version_history_label: document_version_label(
                document.current_version,
                document.version_count,
            ),
            download_path: Some(format!(
                "/auth/onboarding/documents/{}/file",
                document.id.max(0) as u64
            )),
            can_view_file: true,
            can_edit: false,
            can_verify_blockchain: false,
            can_delete: false,
        })
        .collect::<Vec<_>>();

    let history = list_user_history_entries(pool, user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|entry| AdminUserHistoryItem {
            status_label: AccountStatus::from_legacy_code(entry.status)
                .map(account_status_label)
                .unwrap_or_else(|| format!("Status {}", entry.status)),
            remarks: entry.remarks,
            created_at_label: format_datetime(&entry.created_at),
            admin_name: entry.admin_name,
        })
        .collect::<Vec<_>>();

    let role = user.primary_role();
    let status = user.account_status();

    Ok(Json(ApiResponse::ok(AdminUserProfileScreen {
        user_id: user.id.max(0) as u64,
        name: user.name.clone(),
        email: user.email.clone(),
        role_key: role.map(admin_role_key).unwrap_or_else(|| "unknown".into()),
        role_label: role
            .map(|value| value.label().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        status_key: status
            .map(admin_status_key)
            .unwrap_or_else(|| "unknown".into()),
        status_label: status
            .map(account_status_label)
            .unwrap_or_else(|| format!("Status {}", user.status)),
        phone_no: user.phone_no.clone(),
        address: user.address.clone(),
        dob_label: user.dob.map(|value| value.format("%b %d, %Y").to_string()),
        gender: user.gender.clone(),
        joined_at_label: format_datetime(&user.created_at),
        company_name: details
            .as_ref()
            .and_then(|detail| detail.company_name.clone())
            .or(user.company_name.clone()),
        company_address: details
            .as_ref()
            .and_then(|detail| detail.company_address.clone())
            .or(user.company_address.clone()),
        image_path: user.image.clone(),
        personal_facts: admin_personal_facts(&user),
        company_facts: admin_company_facts(&user, details.as_ref()),
        documents,
        history,
        notes: vec![
            "This Rust profile folds the old modal profile, user detail page, and KYC visibility into one admin surface.".into(),
            "KYC downloads still use the protected uploader-plus-admin access rule.".into(),
        ],
    })))
}

async fn create_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<Json<ApiResponse<AdminCreateUserResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: format!(
                "User creation is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Name and email are required before creating a user.".into(),
        })));
    }

    if payload.password != payload.password_confirmation {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Password confirmation does not match.".into(),
        })));
    }

    if payload.password.trim().len() < 8 {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Choose a password with at least 8 characters.".into(),
        })));
    }

    let Some(role) = parse_admin_role_key(&payload.role_key) else {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Choose a valid role before creating the user.".into(),
        })));
    };

    let Some(status) = parse_admin_status_key(&payload.status_key) else {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Choose a valid account status before creating the user.".into(),
        })));
    };

    let password_hash =
        hash(&payload.password, 12).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let created = create_admin_user_account(
        pool,
        session.user.id,
        &CreateAdminUserInput {
            organization_id: auth_session::session_organization_id(&session)
                .ok_or(StatusCode::FORBIDDEN)?,
            name: payload.name.trim().to_string(),
            email: payload.email.trim().to_ascii_lowercase(),
            password_hash,
            role_id: role.legacy_id(),
            status: status.legacy_code(),
            phone_no: payload.phone_no.clone(),
            address: payload.address.clone(),
        },
    )
    .await;

    match created {
        Ok(user) => {
            let summary = format!(
                "{} created {} in the Rust admin directory.",
                session.user.name, user.email
            );
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::AdminDashboardUpdated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: summary.clone(),
                })
                .for_permission_keys(["manage_users", "access_admin_portal"])
                .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
            );

            Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
                success: true,
                user_id: Some(user.id.max(0) as u64),
                message: summary,
            })))
        }
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
            Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
                success: false,
                user_id: None,
                message: "That email address is already in use.".into(),
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_user_profile_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateUserProfileRequest>,
) -> Result<Json<ApiResponse<AdminUpdateUserProfileResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: format!(
                "Profile update is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "Name and email are required before saving the profile.".into(),
        })));
    }

    let password_hash = match (
        payload
            .password
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        payload
            .password_confirmation
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(password), Some(password_confirmation)) => {
            if password != password_confirmation {
                return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                    success: false,
                    user_id: user_id.max(0) as u64,
                    message: "Password confirmation does not match.".into(),
                })));
            }
            if password.len() < 8 {
                return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                    success: false,
                    user_id: user_id.max(0) as u64,
                    message: "Choose a password with at least 8 characters.".into(),
                })));
            }
            Some(hash(password, 12).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        }
        (Some(_), None) | (None, Some(_)) => {
            return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                success: false,
                user_id: user_id.max(0) as u64,
                message: "Provide both password fields before changing the password.".into(),
            })));
        }
        (None, None) => None,
    };

    let Some(target_user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The selected user account was not found.".into(),
        })));
    };
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_user.organization_id,
        "admin_user_profile_updated",
        "user",
        Some(user_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let updated = update_admin_user_profile(
        pool,
        &UpdateAdminUserProfileInput {
            user_id,
            admin_id: session.user.id,
            name: payload.name.trim().to_string(),
            email: payload.email.trim().to_ascii_lowercase(),
            password_hash,
            phone_no: payload.phone_no.clone(),
            address: payload.address.clone(),
            remarks: payload.remarks.clone(),
        },
    )
    .await;

    match updated {
        Ok(Some(user)) => {
            let summary = format!(
                "{} updated the Rust profile for {}.",
                session.user.name, user.email
            );
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::AdminDashboardUpdated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: summary.clone(),
                })
                .for_permission_keys(["manage_users", "access_admin_portal"])
                .for_user_ids([user.id.max(0) as u64])
                .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
            );
            Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                success: true,
                user_id: user.id.max(0) as u64,
                message: summary,
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The selected user account was not found.".into(),
        }))),
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
            Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                success: false,
                user_id: user_id.max(0) as u64,
                message: "That email address is already in use.".into(),
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminDeleteUserResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: format!(
                "Delete action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if session.user.id == user_id {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The Rust admin portal will not let you delete your own account.".into(),
        })));
    }

    let Some(target_user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The selected user account was not found.".into(),
        })));
    };
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_user.organization_id,
        "admin_user_deleted",
        "user",
        Some(user_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let deleted = delete_admin_user_account(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !deleted {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The selected user account was not found.".into(),
        })));
    }

    let summary = format!(
        "{} deleted user #{} from the Rust admin directory.",
        session.user.name, user_id
    );
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(user_id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
        success: true,
        user_id: user_id.max(0) as u64,
        message: summary,
    })))
}

async fn start_break_glass_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminBreakGlassRequest>,
) -> Result<Json<ApiResponse<AdminBreakGlassResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminBreakGlassResponse {
            success: false,
            session_id: None,
            target_organization_id: payload.target_organization_id,
            expires_at: None,
            message: format!(
                "Break-glass is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let target_organization_id =
        i64::try_from(payload.target_organization_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let reason = payload.reason.trim();
    let ticket_ref = payload.ticket_ref.trim();
    if reason.len() < 12 || ticket_ref.len() < 3 {
        return Ok(Json(ApiResponse::ok(AdminBreakGlassResponse {
            success: false,
            session_id: None,
            target_organization_id: payload.target_organization_id,
            expires_at: None,
            message: "Break-glass requires a clear reason and ticket reference.".into(),
        })));
    }

    let actor_organization_id = auth_session::session_organization_id(&session);
    if actor_organization_id == Some(target_organization_id) {
        return Ok(Json(ApiResponse::ok(AdminBreakGlassResponse {
            success: false,
            session_id: None,
            target_organization_id: payload.target_organization_id,
            expires_at: None,
            message: "Break-glass is only for cross-organization access.".into(),
        })));
    }

    let duration_minutes = payload.duration_minutes.unwrap_or(30).clamp(5, 60);
    let expires_at = Utc::now().naive_utc() + chrono::Duration::minutes(duration_minutes as i64);
    let session_id = format!("bg_{}", Uuid::new_v4().simple());
    let request_id = request_id_from_headers(&headers);
    let break_glass = create_break_glass_session(
        pool,
        &CreateBreakGlassSessionInput {
            id: &session_id,
            actor_user_id: session.user.id,
            actor_organization_id,
            target_organization_id,
            reason,
            ticket_ref,
            request_id: request_id.as_deref(),
            expires_at,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminBreakGlassResponse {
        success: true,
        session_id: Some(break_glass.id),
        target_organization_id: payload.target_organization_id,
        expires_at: Some(break_glass.expires_at.to_string()),
        message: "Break-glass access started, time-boxed, and written to the audit ledger.".into(),
    })))
}

async fn identity_screen(
    State(state): State<AppState>,
    Query(query): Query<AdminIdentityQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminIdentityScreen>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminIdentityScreen {
            title: "Enterprise Identity".into(),
            target_organization_id: query.target_organization_id.unwrap_or_default(),
            domains: Vec::new(),
            providers: Vec::new(),
            scim_events: Vec::new(),
            notes: vec![format!(
                "Enterprise identity configuration is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let target_organization_id = resolve_support_target_organization_id(
        query.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;
    Ok(Json(ApiResponse::ok(
        load_identity_screen(pool, target_organization_id).await?,
    )))
}

async fn upsert_identity_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpsertIdentityDomainRequest>,
) -> Result<Json<ApiResponse<AdminIdentityMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;
    let domain = normalize_domain(&payload.domain)?;
    let verification_token = format!("stloads-domain-{}", Uuid::new_v4());

    if let Some(existing_id) = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM organization_domains WHERE LOWER(domain) = LOWER($1)",
    )
    .bind(&domain)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        sqlx::query(
            "UPDATE organization_domains
             SET organization_id = $2,
                 verification_status = CASE WHEN verification_status = 'verified' THEN 'verified' ELSE 'pending' END,
                 verification_token = CASE WHEN verification_status = 'verified' THEN verification_token ELSE $3 END,
                 login_routing_enabled = $4,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1",
        )
        .bind(existing_id)
        .bind(target_organization_id)
        .bind(&verification_token)
        .bind(payload.login_routing_enabled)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        sqlx::query(
            "INSERT INTO organization_domains (
                organization_id, domain, verification_status, verification_token, login_routing_enabled, created_at, updated_at
             )
             VALUES ($1, $2, 'pending', $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(target_organization_id)
        .bind(&domain)
        .bind(&verification_token)
        .bind(payload.login_routing_enabled)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(&session),
            target_organization_id: Some(target_organization_id),
            entity_type: "enterprise_identity_domain",
            entity_id: Some(&domain),
            action: "identity_domain_upserted",
            reason: Some("enterprise identity domain configured"),
            ticket_ref: None,
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "login_routing_enabled": payload.login_routing_enabled,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminIdentityMutationResponse {
        success: true,
        message:
            "Identity domain saved. Publish the verification token before enforcing SSO routing."
                .into(),
        screen: load_identity_screen(pool, target_organization_id).await?,
    })))
}

async fn verify_identity_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminVerifyIdentityDomainRequest>,
) -> Result<Json<ApiResponse<AdminIdentityMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;
    let domain = normalize_domain(&payload.domain)?;
    let result = sqlx::query(
        "UPDATE organization_domains
         SET verification_status = 'verified',
             verified_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE organization_id = $1
           AND LOWER(domain) = LOWER($2)
           AND verification_token = $3",
    )
    .bind(target_organization_id)
    .bind(&domain)
    .bind(payload.verification_token.trim())
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let success = result.rows_affected() > 0;
    Ok(Json(ApiResponse::ok(AdminIdentityMutationResponse {
        success,
        message: if success {
            "Identity domain verified and ready for SSO routing controls.".into()
        } else {
            "Domain verification token did not match.".into()
        },
        screen: load_identity_screen(pool, target_organization_id).await?,
    })))
}

async fn check_identity_domain_dns(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminCheckIdentityDomainDnsRequest>,
) -> Result<Json<ApiResponse<AdminIdentityMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;
    let domain = normalize_domain(&payload.domain)?;
    let verification_token = sqlx::query_scalar::<_, String>(
        "SELECT verification_token
         FROM organization_domains
         WHERE organization_id = $1
           AND LOWER(domain) = LOWER($2)",
    )
    .bind(target_organization_id)
    .bind(&domain)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let dns_name = format!("_stloads-domain.{}", domain);
    let verified = dns_txt_contains_token(&dns_name, &verification_token)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if verified {
        sqlx::query(
            "UPDATE organization_domains
             SET verification_status = 'verified',
                 verified_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1
               AND LOWER(domain) = LOWER($2)",
        )
        .bind(target_organization_id)
        .bind(&domain)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(ApiResponse::ok(AdminIdentityMutationResponse {
        success: verified,
        message: if verified {
            "DNS TXT verification succeeded and the domain was marked verified.".into()
        } else {
            format!(
                "DNS TXT record _stloads-domain.{domain} did not contain the verification token."
            )
        },
        screen: load_identity_screen(pool, target_organization_id).await?,
    })))
}

async fn upsert_identity_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpsertIdentityProviderRequest>,
) -> Result<Json<ApiResponse<AdminIdentityMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    ensure_support_org_access(pool, &session, target_organization_id).await?;
    let provider_type = normalize_identity_provider_type(&payload.provider_type)?;
    let status = normalize_identity_provider_status(&payload.status)?;
    let display_name = payload.display_name.trim();
    if display_name.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let default_role_key = normalize_organization_role_key(&payload.default_role_key)?;

    if let Some(provider_id) = payload.provider_id {
        sqlx::query(
            "UPDATE enterprise_identity_providers
             SET provider_type = $3,
                 status = $4,
                 display_name = $5,
                 issuer = $6,
                 sso_url = $7,
                 jwks_url = $8,
                 metadata_url = $9,
                 client_id = $10,
                 jit_enabled = $11,
                 default_role_key = $12,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
               AND organization_id = $2",
        )
        .bind(provider_id as i64)
        .bind(target_organization_id)
        .bind(&provider_type)
        .bind(&status)
        .bind(display_name)
        .bind(optional_trimmed(payload.issuer.as_deref()))
        .bind(optional_trimmed(payload.sso_url.as_deref()))
        .bind(optional_trimmed(payload.jwks_url.as_deref()))
        .bind(optional_trimmed(payload.metadata_url.as_deref()))
        .bind(optional_trimmed(payload.client_id.as_deref()))
        .bind(payload.jit_enabled)
        .bind(&default_role_key)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        sqlx::query(
            "INSERT INTO enterprise_identity_providers (
                organization_id, provider_type, status, display_name, issuer, sso_url, jwks_url,
                metadata_url, client_id, jit_enabled, default_role_key, created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(target_organization_id)
        .bind(&provider_type)
        .bind(&status)
        .bind(display_name)
        .bind(optional_trimmed(payload.issuer.as_deref()))
        .bind(optional_trimmed(payload.sso_url.as_deref()))
        .bind(optional_trimmed(payload.jwks_url.as_deref()))
        .bind(optional_trimmed(payload.metadata_url.as_deref()))
        .bind(optional_trimmed(payload.client_id.as_deref()))
        .bind(payload.jit_enabled)
        .bind(&default_role_key)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let provider_entity_id = payload.provider_id.map(|id| id.to_string());
    let request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(&session),
            target_organization_id: Some(target_organization_id),
            entity_type: "enterprise_identity_provider",
            entity_id: provider_entity_id.as_deref(),
            action: "identity_provider_upserted",
            reason: Some("enterprise identity provider configured"),
            ticket_ref: None,
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "provider_type": provider_type,
                "status": status,
                "jit_enabled": payload.jit_enabled,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminIdentityMutationResponse {
        success: true,
        message: "Identity provider saved.".into(),
        screen: load_identity_screen(pool, target_organization_id).await?,
    })))
}

async fn access_reviews_screen(
    State(state): State<AppState>,
    Query(query): Query<AdminAccessReviewQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminAccessReviewScreen>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminAccessReviewScreen {
            title: "Access Reviews".into(),
            summary: "Database unavailable".into(),
            target_organization_id: 0,
            reviews: Vec::new(),
            items: Vec::new(),
            elevation_requests: Vec::new(),
            notes: vec![format!(
                "Access reviews are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let target_organization_id = resolve_support_target_organization_id(
        query.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_organization_id,
        "access_reviews_viewed",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(ApiResponse::ok(
        load_access_review_screen(pool, target_organization_id).await?,
    )))
}

async fn start_access_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminStartAccessReviewRequest>,
) -> Result<Json<ApiResponse<AdminAccessReviewMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_organization_id,
        "access_review_started_cross_org",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let title = payload.title.trim();
    if title.len() < 3 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let due_days = payload.due_days.unwrap_or(14).clamp(1, 90);
    let due_at = Utc::now().naive_utc() + chrono::Duration::days(due_days as i64);
    let (review, items) = create_access_review(
        pool,
        &CreateAccessReviewInput {
            organization_id: target_organization_id,
            title,
            review_type: "quarterly",
            created_by_user_id: session.user.id,
            due_at: Some(due_at),
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let review_entity_id = review.id.to_string();
    let request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(&session),
            target_organization_id: Some(target_organization_id),
            entity_type: "access_review",
            entity_id: Some(review_entity_id.as_str()),
            action: "access_review_started",
            reason: Some("least-privilege recertification campaign started"),
            ticket_ref: None,
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "title": review.title,
                "item_count": items.len(),
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminAccessReviewMutationResponse {
        success: true,
        message: format!(
            "Access review started with {} privileged access item(s).",
            items.len()
        ),
        screen: load_access_review_screen(pool, target_organization_id).await?,
    })))
}

async fn decide_access_review(
    State(state): State<AppState>,
    Path(item_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminAccessReviewDecisionRequest>,
) -> Result<Json<ApiResponse<AdminAccessReviewMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_organization_id,
        "access_review_decision_cross_org",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let decision = normalize_access_review_decision(&payload.decision)?;
    let decision_reason = optional_trimmed(payload.reason.as_deref());
    let item = decide_access_review_item(
        pool,
        &AccessReviewDecisionInput {
            item_id,
            organization_id: target_organization_id,
            decided_by_user_id: session.user.id,
            decision: &decision,
            decision_reason: decision_reason.as_deref(),
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if decision == "revoke" {
        sqlx::query(
            "UPDATE organization_memberships
             SET status = 'revoked',
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1
               AND user_id = $2
               AND role_key = $3",
        )
        .bind(target_organization_id)
        .bind(item.user_id)
        .bind(&item.role_key)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if ["admin", "owner", "finance"].contains(&item.role_key.as_str()) {
            sqlx::query(
                "UPDATE users
                 SET status = $2,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1
                   AND organization_id = $3",
            )
            .bind(item.user_id)
            .bind(AccountStatus::Rejected.legacy_code())
            .bind(target_organization_id)
            .execute(pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        revoke_all_access_artifacts_for_user(pool, item.user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let item_entity_id = item.id.to_string();
    let request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(&session),
            target_organization_id: Some(target_organization_id),
            entity_type: "access_review_item",
            entity_id: Some(item_entity_id.as_str()),
            action: "access_review_item_decided",
            reason: payload.reason.as_deref(),
            ticket_ref: None,
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "review_id": item.review_id,
                "user_id": item.user_id,
                "decision": decision,
                "risk_flags": item.risk_flags,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminAccessReviewMutationResponse {
        success: true,
        message: format!("Access review decision saved for {}.", item.user_email),
        screen: load_access_review_screen(pool, target_organization_id).await?,
    })))
}

async fn create_access_elevation_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminCreateAccessElevationRequest>,
) -> Result<Json<ApiResponse<AdminAccessReviewMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_organization_id,
        "access_elevation_request_cross_org",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let requested_role_key = normalize_organization_role_key(&payload.requested_role_key)?;
    let justification = payload.business_justification.trim();
    if justification.len() < 12 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let target_user_id =
        i64::try_from(payload.target_user_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let current_role_key = sqlx::query_scalar::<_, Option<String>>(
        "SELECT role_key
         FROM organization_memberships
         WHERE organization_id = $1
           AND user_id = $2
         ORDER BY updated_at DESC
         LIMIT 1",
    )
    .bind(target_organization_id)
    .bind(target_user_id)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let expires_at = payload
        .expires_in_days
        .map(|days| Utc::now().naive_utc() + chrono::Duration::days(days.clamp(1, 90) as i64));
    let request_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO access_elevation_requests (
            organization_id, requester_user_id, target_user_id, current_role_key,
            requested_role_key, status, business_justification, expires_at, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(target_organization_id)
    .bind(session.user.id)
    .bind(target_user_id)
    .bind(current_role_key.as_deref())
    .bind(&requested_role_key)
    .bind(justification)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request_entity_id = request_id.to_string();
    let correlation_request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(&session),
            target_organization_id: Some(target_organization_id),
            entity_type: "access_elevation_request",
            entity_id: Some(request_entity_id.as_str()),
            action: "access_elevation_requested",
            reason: Some(justification),
            ticket_ref: None,
            request_id: correlation_request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "target_user_id": target_user_id,
                "requested_role_key": requested_role_key,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminAccessReviewMutationResponse {
        success: true,
        message: "Privilege elevation request created for approval.".into(),
        screen: load_access_review_screen(pool, target_organization_id).await?,
    })))
}

async fn decide_access_elevation_request(
    State(state): State<AppState>,
    Path(request_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminAccessElevationDecisionRequest>,
) -> Result<Json<ApiResponse<AdminAccessReviewMutationResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };
    let target_organization_id = resolve_support_target_organization_id(
        payload.target_organization_id,
        auth_session::session_organization_id(&session),
    )?;
    let decision = match payload.decision.trim().to_ascii_lowercase().as_str() {
        "approve" | "approved" => "approved",
        "reject" | "rejected" | "deny" => "rejected",
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let reason = optional_trimmed(payload.reason.as_deref());

    let request = sqlx::query(
        "UPDATE access_elevation_requests
         SET status = $3,
             decision_reason = $4,
             decided_by_user_id = $5,
             decided_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
           AND organization_id = $2
           AND status = 'pending'
         RETURNING id, target_user_id, requested_role_key",
    )
    .bind(request_id)
    .bind(target_organization_id)
    .bind(decision)
    .bind(reason.as_deref())
    .bind(session.user.id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let target_user_id: i64 = request.get("target_user_id");
    let requested_role_key: String = request.get("requested_role_key");
    if decision == "approved" {
        sqlx::query(
            "INSERT INTO organization_memberships (
                organization_id, user_id, role_key, status, joined_at, created_at, updated_at
             ) VALUES ($1, $2, $3, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (organization_id, user_id) DO UPDATE SET
                role_key = EXCLUDED.role_key,
                status = 'active',
                updated_at = CURRENT_TIMESTAMP",
        )
        .bind(target_organization_id)
        .bind(target_user_id)
        .bind(&requested_role_key)
        .execute(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        revoke_all_access_artifacts_for_user(pool, target_user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let request_entity_id = request_id.to_string();
    let correlation_request_id = request_id_from_headers(&headers);
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(&session),
            target_organization_id: Some(target_organization_id),
            entity_type: "access_elevation_request",
            entity_id: Some(request_entity_id.as_str()),
            action: "access_elevation_decided",
            reason: reason.as_deref(),
            ticket_ref: None,
            request_id: correlation_request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "decision": decision,
                "target_user_id": target_user_id,
                "requested_role_key": requested_role_key,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(AdminAccessReviewMutationResponse {
        success: true,
        message: "Privilege elevation decision saved.".into(),
        screen: load_access_review_screen(pool, target_organization_id).await?,
    })))
}

async fn role_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminRolePermissionScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminRolePermissionScreen {
            title: "Role Permissions".into(),
            summary: "Database unavailable".into(),
            permissions: permission_option_catalog(),
            roles: Vec::new(),
            notes: vec![format!(
                "Role-permission management is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let mut roles = Vec::new();
    for role in role_descriptors() {
        let assigned_permission_keys =
            list_permission_names_for_role(pool, i64::from(role.legacy_id))
                .await
                .unwrap_or_default();
        let assigned_permission_keys = if assigned_permission_keys.is_empty() {
            role_permission_contracts()
                .iter()
                .find(|contract| contract.role == role.role)
                .map(|contract| {
                    contract
                        .permissions
                        .iter()
                        .map(|permission| admin_permission_key(*permission).to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            assigned_permission_keys
        };

        roles.push(AdminRolePermissionRow {
            role_key: admin_role_key(role.role),
            role_label: role.label.into(),
            assigned_permission_keys,
        });
    }

    Ok(Json(ApiResponse::ok(AdminRolePermissionScreen {
        title: "Role Permissions".into(),
        summary: format!(
            "{} role profiles currently have editable permission sets in the Rust admin surface.",
            roles.len()
        ),
        permissions: permission_option_catalog(),
        roles,
        notes: vec![
            "Rust sessions now resolve permission keys from role_has_permissions first, with the static role contract as a safe fallback if the database has no assignment yet.".into(),
            "Saving a role permission set invalidates active sessions for users on that role so websocket scopes and route guards reconnect with the updated permissions.".into(),
        ],
    })))
}

async fn review_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ReviewOnboardingRequest>,
) -> Result<Json<ApiResponse<ReviewOnboardingResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            status_label: None,
            message: format!(
                "Review action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let next_status = match payload.decision.trim().to_ascii_lowercase().as_str() {
        "approve" | "approved" => AccountStatus::Approved,
        "reject" | "rejected" => AccountStatus::Rejected,
        "revision" | "revise" | "request_revision" => AccountStatus::RevisionRequested,
        _ => {
            return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
                success: false,
                user_id: user_id.max(0) as u64,
                status_label: None,
                message: "Choose approve, reject, or revision for the Rust review action.".into(),
            })));
        }
    };

    let Some(target_user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            status_label: None,
            message: "The selected onboarding account was not found.".into(),
        })));
    };
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_user.organization_id,
        "admin_onboarding_reviewed",
        "user",
        Some(user_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let updated = review_onboarding_user(
        pool,
        user_id,
        session.user.id,
        next_status.legacy_code(),
        payload.remarks.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(updated_user) = updated else {
        return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            status_label: None,
            message: "The selected onboarding account was not found.".into(),
        })));
    };

    let mail_note = send_account_review_notification(
        &state,
        &updated_user,
        next_status,
        payload.remarks.as_deref(),
        request_id_from_headers(&headers).as_deref(),
    )
    .await;

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(updated_user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} changed {} to {}.",
                session.user.name,
                updated_user.email,
                account_status_label(next_status)
            ),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"])
        .for_user_ids([updated_user.id.max(0) as u64])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
        success: true,
        user_id: updated_user.id.max(0) as u64,
        status_label: Some(account_status_label(next_status)),
        message: append_optional_note(
            format!(
                "Rust review updated {} to {}.",
                updated_user.email,
                account_status_label(next_status)
            ),
            mail_note,
        ),
    })))
}

async fn update_user_account_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<Json<ApiResponse<AdminUpdateUserResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: format!(
                "User update is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(role) = parse_admin_role_key(&payload.role_key) else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "Choose a valid role before saving this user.".into(),
        })));
    };

    let Some(next_status) = parse_admin_status_key(&payload.status_key) else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "Choose a valid account status before saving this user.".into(),
        })));
    };

    let Some(target_user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "The selected user account was not found.".into(),
        })));
    };
    if !ensure_admin_can_access_organization(
        pool,
        &session,
        target_user.organization_id,
        "admin_user_account_updated",
        "user",
        Some(user_id.to_string()),
    )
    .await?
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let updated = update_admin_user_account(
        pool,
        user_id,
        session.user.id,
        role.legacy_id(),
        next_status.legacy_code(),
        payload.remarks.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(updated_user) = updated else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "The selected user account was not found.".into(),
        })));
    };

    let updated_role = updated_user.primary_role();
    let updated_status = updated_user.account_status();
    let _ = delete_personal_access_tokens_for_user(pool, updated_user.id).await;
    let mail_note = match updated_status {
        Some(
            AccountStatus::Approved | AccountStatus::Rejected | AccountStatus::RevisionRequested,
        ) => {
            send_account_review_notification(
                &state,
                &updated_user,
                updated_status.unwrap(),
                payload.remarks.as_deref(),
                request_id_from_headers(&headers).as_deref(),
            )
            .await
        }
        _ => None,
    };
    let summary = format!(
        "{} updated {} to role {} and status {}.",
        session.user.name,
        updated_user.email,
        updated_role
            .map(|value| value.label().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        account_status_label(updated_status.unwrap_or(AccountStatus::PendingReview))
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(updated_user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"])
        .for_user_ids([updated_user.id.max(0) as u64])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::SessionInvalidated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(updated_user.id.max(0) as u64),
            subject_user_id: Some(updated_user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: "Your role or account status changed in the Rust admin directory. Refresh your session.".into(),
        })
        .for_user_ids([updated_user.id.max(0) as u64]),
    );

    Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
        success: true,
        user_id: updated_user.id.max(0) as u64,
        role_label: updated_role.map(|value| value.label().to_string()),
        status_label: updated_status.map(account_status_label),
        message: append_optional_note(summary, mail_note),
    })))
}

async fn update_role_permissions_handler(
    State(state): State<AppState>,
    Path(role_key): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateRolePermissionsRequest>,
) -> Result<Json<ApiResponse<AdminUpdateRolePermissionsResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    require_mfa_step_up(&session)?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
            success: false,
            role_key,
            assigned_permission_keys: Vec::new(),
            message: format!(
                "Role permission updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(role) = parse_admin_role_key(&role_key) else {
        return Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
            success: false,
            role_key,
            assigned_permission_keys: Vec::new(),
            message: "Choose a valid role before saving permissions.".into(),
        })));
    };

    let valid_permission_keys = permission_descriptors()
        .iter()
        .map(|descriptor| admin_permission_key(descriptor.permission).to_string())
        .collect::<Vec<_>>();

    let filtered_permission_keys = payload
        .permission_keys
        .into_iter()
        .map(|permission| permission.trim().to_string())
        .filter(|permission| {
            valid_permission_keys
                .iter()
                .any(|candidate| candidate == permission)
        })
        .collect::<Vec<_>>();

    if filtered_permission_keys.is_empty() {
        return Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
            success: false,
            role_key: admin_role_key(role),
            assigned_permission_keys: Vec::new(),
            message: "Assign at least one permission before saving a Rust role.".into(),
        })));
    }

    let mut filtered_permission_keys = filtered_permission_keys;
    if role == UserRole::Admin {
        for required_permission in ["access_admin_portal", "manage_roles"] {
            if !filtered_permission_keys
                .iter()
                .any(|permission| permission == required_permission)
            {
                filtered_permission_keys.push(required_permission.into());
            }
        }
        filtered_permission_keys.sort();
        filtered_permission_keys.dedup();
    }

    let assigned_permission_keys =
        replace_role_permissions(pool, i64::from(role.legacy_id()), &filtered_permission_keys)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let affected_user_ids_raw = list_user_ids_for_role(pool, i64::from(role.legacy_id()))
        .await
        .unwrap_or_default();
    for affected_user_id in &affected_user_ids_raw {
        let _ = delete_personal_access_tokens_for_user(pool, *affected_user_id).await;
    }
    let affected_user_ids = affected_user_ids_raw
        .into_iter()
        .map(|value| value.max(0) as u64)
        .collect::<Vec<_>>();

    let summary = format!(
        "{} updated the {} role permission set in the Rust admin portal.",
        session.user.name,
        role.label()
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_roles", "access_admin_portal"])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::SessionInvalidated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "Your role permissions changed for the {} role. Refresh your Rust session.",
                role.label()
            ),
        })
        .for_user_ids(affected_user_ids),
    );

    Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
        success: true,
        role_key: admin_role_key(role),
        assigned_permission_keys,
        message: summary,
    })))
}

async fn stloads_operations(
    State(state): State<AppState>,
    Query(query): Query<OperationsQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StloadsOperationsScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        screen_data::stloads_operations_screen(&state, query.status).await,
    )))
}

async fn stloads_reconciliation(
    State(state): State<AppState>,
    Query(query): Query<ReconciliationQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StloadsReconciliationScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        screen_data::stloads_reconciliation_screen(&state, query.action).await,
    )))
}

async fn resolve_sync_error_handler(
    State(state): State<AppState>,
    Path(sync_error_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ResolveSyncErrorRequest>,
) -> Result<Json<ApiResponse<ResolveSyncErrorResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: None,
            message: format!(
                "Resolve action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let resolved_by = format!("{} <{}>", session.user.name, session.user.email);
    let resolved_record = resolve_sync_error(
        pool,
        sync_error_id,
        &resolved_by,
        payload.resolution_note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(sync_error) = resolved_record else {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: None,
            message: "The requested sync error was not found.".into(),
        })));
    };

    if !sync_error.resolved {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: sync_error.handoff_id,
            message: "The selected sync error could not be resolved yet.".into(),
        })));
    }

    let summary = format!(
        "{} resolved {} on sync error #{}.",
        session.user.name, sync_error.error_class, sync_error_id
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::TmsOperationsUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([RealtimeTopic::AdminTmsOperations.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::TmsReconciliationUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([RealtimeTopic::AdminTmsReconciliation.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
        success: true,
        sync_error_id,
        handoff_id: sync_error.handoff_id,
        message: "Sync error resolved from the Rust STLOADS admin route.".into(),
    })))
}

async fn require_any_permission(
    state: &AppState,
    headers: &HeaderMap,
    permission_keys: &[&str],
) -> Result<ResolvedSession, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(state, headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let allowed = session.session.permissions.iter().any(|permission| {
        permission_keys
            .iter()
            .any(|expected| permission == expected)
    });

    if allowed {
        Ok(session)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn require_integration_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<ResolvedSession, StatusCode> {
    require_any_permission(
        state,
        headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_dispatch_desk",
        ],
    )
    .await
}

async fn load_integration_portal_screen(
    pool: &db::DbPool,
    organization_id: i64,
) -> Result<IntegrationPortalScreen, StatusCode> {
    let api_keys = sqlx::query(
        "SELECT id, client_name, key_prefix, scopes, status, rate_limit_per_minute,
                require_request_signature, last_used_at, expires_at, created_at
         FROM partner_api_clients
         WHERE organization_id = $1
         ORDER BY created_at DESC
         LIMIT 50",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| IntegrationApiKeyRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        client_name: row.get("client_name"),
        key_prefix: row.get("key_prefix"),
        scopes: row.get::<Vec<String>, _>("scopes"),
        status: row.get("status"),
        rate_limit_per_minute: row.get("rate_limit_per_minute"),
        require_request_signature: row.get("require_request_signature"),
        last_used_at: row
            .get::<Option<NaiveDateTime>, _>("last_used_at")
            .map(|value| value.to_string()),
        expires_at: row
            .get::<Option<NaiveDateTime>, _>("expires_at")
            .map(|value| value.to_string()),
        created_at: row.get::<NaiveDateTime, _>("created_at").to_string(),
    })
    .collect::<Vec<_>>();

    let webhook_endpoints = sqlx::query(
        "SELECT id, endpoint_name, target_url, event_types, status, created_at
         FROM outbound_webhook_endpoints
         WHERE organization_id = $1
         ORDER BY created_at DESC
         LIMIT 50",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| IntegrationWebhookEndpointRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        endpoint_name: row.get("endpoint_name"),
        target_url: row.get("target_url"),
        event_types: row.get::<Vec<String>, _>("event_types"),
        status: row.get("status"),
        created_at: row.get::<NaiveDateTime, _>("created_at").to_string(),
    })
    .collect::<Vec<_>>();

    let recent_deliveries = list_webhook_delivery_logs(pool, organization_id, None, 20)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|delivery| WebhookDeliveryRow {
            id: delivery.id.max(0) as u64,
            endpoint_id: delivery.endpoint_id.map(|value| value.max(0) as u64),
            event_type: delivery.event_type,
            event_id: delivery.event_id,
            delivery_status: delivery.delivery_status,
            attempt_count: delivery.attempt_count,
            next_retry_at: delivery.next_retry_at.map(|value| value.to_string()),
            last_attempt_at: delivery.last_attempt_at.map(|value| value.to_string()),
            response_status_code: delivery.response_status_code,
            response_latency_ms: delivery.response_latency_ms,
            response_body_excerpt: delivery.response_body_excerpt,
            dead_letter_reason: delivery.dead_letter_reason,
            replay_of_delivery_id: delivery
                .replay_of_delivery_id
                .map(|value| value.max(0) as u64),
            created_at: delivery.created_at.to_string(),
        })
        .collect();

    Ok(IntegrationPortalScreen {
        organization_id: organization_id.max(0) as u64,
        api_version: crate::api_contract::API_VERSION,
        docs: vec![
            IntegrationDocLink {
                label: "OpenAPI",
                href: "/openapi.json",
                description: "Machine-readable API contract for supported enterprise routes.",
            },
            IntegrationDocLink {
                label: "API Contract V1",
                href: "/docs/API_CONTRACT_V1.md",
                description: "Versioning, idempotency, signing, and lifecycle policy.",
            },
            IntegrationDocLink {
                label: "Operations Runbook",
                href: "/admin/stloads/reconciliation",
                description: "TMS drift, webhook replay, and repair operations.",
            },
        ],
        sandbox: IntegrationSandboxSummary {
            base_url: "https://sandbox-api.stloads.com",
            production_safety: "Sandbox credentials and webhooks must never trigger production payments, production TMS pushes, or live notifications.",
            reset_policy: "Sandbox reset tooling and seeded demo tenants are completed in ENT-1108.",
        },
        api_keys,
        webhook_endpoints,
        recent_deliveries,
    })
}

fn normalize_partner_scopes(values: Vec<String>) -> Vec<String> {
    let mut normalized = values
        .into_iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn non_empty_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

async fn load_edi_integration_screen(
    pool: &db::DbPool,
    organization_id: i64,
) -> Result<EdiIntegrationScreen, StatusCode> {
    let supported_transactions = sqlx::query(
        "SELECT id, transaction_code, direction, stloads_model, mapping_version, status,
                required_fields, notes
         FROM edi_transaction_mappings
         WHERE organization_id = $1 OR organization_id IS NULL
         ORDER BY transaction_code, direction",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| EdiTransactionMappingRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        transaction_code: row.get("transaction_code"),
        direction: row.get("direction"),
        stloads_model: row.get("stloads_model"),
        mapping_version: row.get("mapping_version"),
        status: row.get("status"),
        required_fields: row.get::<Vec<String>, _>("required_fields"),
        notes: row.get("notes"),
    })
    .collect();

    let partner_profiles = sqlx::query(
        "SELECT id, partner_name, isa_id, gs_id, transport_type, status,
                supported_transactions, validation_mode, created_at
         FROM edi_partner_profiles
         WHERE organization_id = $1
         ORDER BY updated_at DESC, partner_name ASC
         LIMIT 50",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| EdiPartnerProfileRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        partner_name: row.get("partner_name"),
        isa_id: row.get("isa_id"),
        gs_id: row.get("gs_id"),
        transport_type: row.get("transport_type"),
        status: row.get("status"),
        supported_transactions: row.get::<Vec<String>, _>("supported_transactions"),
        validation_mode: row.get("validation_mode"),
        created_at: row.get::<NaiveDateTime, _>("created_at").to_string(),
    })
    .collect();

    let message_logs = sqlx::query(
        "SELECT id, partner_profile_id, transaction_code, direction, control_number,
                business_key, message_status, ack_status, retry_count, next_retry_at,
                error_summary, replay_of_message_id, created_at
         FROM edi_message_logs
         WHERE organization_id = $1
         ORDER BY created_at DESC
         LIMIT 50",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| EdiMessageLogRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        partner_profile_id: row
            .get::<Option<i64>, _>("partner_profile_id")
            .map(|value| value.max(0) as u64),
        transaction_code: row.get("transaction_code"),
        direction: row.get("direction"),
        control_number: row.get("control_number"),
        business_key: row.get("business_key"),
        message_status: row.get("message_status"),
        ack_status: row.get("ack_status"),
        retry_count: row.get("retry_count"),
        next_retry_at: row
            .get::<Option<NaiveDateTime>, _>("next_retry_at")
            .map(|value| value.to_string()),
        error_summary: row.get("error_summary"),
        replay_of_message_id: row
            .get::<Option<i64>, _>("replay_of_message_id")
            .map(|value| value.max(0) as u64),
        created_at: row.get::<NaiveDateTime, _>("created_at").to_string(),
    })
    .collect();

    Ok(EdiIntegrationScreen {
        supported_transactions,
        partner_profiles,
        message_logs,
        replay_policy: "Failed, retrying, dead-letter, mapped, and validated EDI messages can be replay-queued with full lineage.",
    })
}

async fn ensure_default_sandbox_environment(
    pool: &db::DbPool,
    organization_id: i64,
) -> Result<(), StatusCode> {
    sqlx::query(
        "INSERT INTO sandbox_tenant_environments (
             organization_id, environment_key, display_name, base_url, data_classification,
             pii_allowed, production_payment_blocked, production_tms_push_blocked,
             production_notification_blocked, seeded_dataset_version, reset_status,
             created_at, updated_at
         )
         VALUES (
             $1, 'default-sandbox', 'Default Enterprise Sandbox', 'https://sandbox-api.stloads.com',
             'synthetic', FALSE, TRUE, TRUE, TRUE, 'demo-v1', 'ready',
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (organization_id, environment_key)
         DO UPDATE SET
             data_classification = 'synthetic',
             pii_allowed = FALSE,
             production_payment_blocked = TRUE,
             production_tms_push_blocked = TRUE,
             production_notification_blocked = TRUE,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(organization_id)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn load_sandbox_governance_screen(
    pool: &db::DbPool,
    organization_id: i64,
) -> Result<SandboxGovernanceScreen, StatusCode> {
    let environments = sqlx::query(
        "SELECT id, environment_key, display_name, base_url, data_classification,
                pii_allowed, production_payment_blocked, production_tms_push_blocked,
                production_notification_blocked, seeded_dataset_version, reset_status, last_reset_at
         FROM sandbox_tenant_environments
         WHERE organization_id = $1
         ORDER BY created_at DESC",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| SandboxEnvironmentRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        environment_key: row.get("environment_key"),
        display_name: row.get("display_name"),
        base_url: row.get("base_url"),
        data_classification: row.get("data_classification"),
        pii_allowed: row.get("pii_allowed"),
        production_payment_blocked: row.get("production_payment_blocked"),
        production_tms_push_blocked: row.get("production_tms_push_blocked"),
        production_notification_blocked: row.get("production_notification_blocked"),
        seeded_dataset_version: row.get("seeded_dataset_version"),
        reset_status: row.get("reset_status"),
        last_reset_at: row
            .get::<Option<NaiveDateTime>, _>("last_reset_at")
            .map(|value| value.to_string()),
    })
    .collect::<Vec<_>>();

    let reset_jobs = sqlx::query(
        "SELECT id, sandbox_environment_id, job_status, reset_reason, result_summary,
                created_at, completed_at
         FROM sandbox_reset_jobs
         WHERE organization_id = $1
         ORDER BY created_at DESC
         LIMIT 25",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| SandboxResetJobRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        sandbox_environment_id: row.get::<i64, _>("sandbox_environment_id").max(0) as u64,
        job_status: row.get("job_status"),
        reset_reason: row.get("reset_reason"),
        result_summary: row.get("result_summary"),
        created_at: row.get::<NaiveDateTime, _>("created_at").to_string(),
        completed_at: row
            .get::<Option<NaiveDateTime>, _>("completed_at")
            .map(|value| value.to_string()),
    })
    .collect();

    Ok(SandboxGovernanceScreen {
        environments,
        reset_jobs,
        policy_notes: vec![
            "Sandbox data must remain synthetic or masked and cannot contain real PII, payment credentials, documents, or customer freight.",
            "Sandbox events are blocked from production payments, production TMS pushes, and live notifications by database-level safety checks.",
            "Reset jobs are queued with safety evidence so sales, support, QA, and customers can retest without developer database access.",
        ],
    })
}

async fn load_api_lifecycle_screen(pool: &db::DbPool) -> Result<ApiLifecycleScreen, StatusCode> {
    let policies = sqlx::query(
        "SELECT api_version, release_status, released_on, sunset_on, minimum_notice_days,
                emergency_breaking_change_policy, changelog_url, postman_collection_url,
                sdk_strategy, compatibility_test_status
         FROM api_lifecycle_policies
         ORDER BY released_on DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| ApiLifecyclePolicyRow {
        api_version: row.get("api_version"),
        release_status: row.get("release_status"),
        released_on: row.get::<NaiveDate, _>("released_on").to_string(),
        sunset_on: row
            .get::<Option<NaiveDate>, _>("sunset_on")
            .map(|value| value.to_string()),
        minimum_notice_days: row.get("minimum_notice_days"),
        emergency_breaking_change_policy: row.get("emergency_breaking_change_policy"),
        changelog_url: row.get("changelog_url"),
        postman_collection_url: row.get("postman_collection_url"),
        sdk_strategy: row.get("sdk_strategy"),
        compatibility_test_status: row.get("compatibility_test_status"),
    })
    .collect();

    let examples = sqlx::query(
        "SELECT api_version, example_key, surface, method, path, sandbox_runnable
         FROM api_partner_examples
         ORDER BY surface, example_key",
    )
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| ApiPartnerExampleRow {
        api_version: row.get("api_version"),
        example_key: row.get("example_key"),
        surface: row.get("surface"),
        method: row.get("method"),
        path: row.get("path"),
        sandbox_runnable: row.get("sandbox_runnable"),
    })
    .collect();

    Ok(ApiLifecycleScreen {
        policies,
        examples,
        upgrade_paths: vec![
            "Public REST integrations upgrade by pinning `stloads-api-version`, replaying compatibility examples in sandbox, then moving production keys.",
            "Webhook consumers upgrade by adding handlers for new event fields before the sunset window ends; removed fields require notice and examples.",
            "EDI partners upgrade mapping versions in sandbox, verify 997 acknowledgements, then enable the partner profile in production.",
            "TMS sync partners upgrade status payloads through sandbox validation and reconciliation checks before live pushes are allowed.",
        ],
    })
}

fn target_url_is_allowed(target_url: &str) -> bool {
    target_url.starts_with("https://")
        && !target_url.contains(' ')
        && !target_url.contains("localhost")
        && !target_url.contains("127.0.0.1")
}

fn parse_optional_datetime_for_admin(
    value: Option<&str>,
) -> Result<Option<NaiveDateTime>, StatusCode> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S"))
        .map(Some)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

async fn append_support_organization_results(
    pool: &db::DbPool,
    target_organization_id: i64,
    exact_query: &str,
    like_query: &str,
    results: &mut Vec<AdminSupportSearchResult>,
) -> Result<(), StatusCode> {
    let rows = sqlx::query(
        "SELECT id, name, slug, account_type, status, support_tier
         FROM organizations
         WHERE id = $1
           AND (
             id::text = $2
             OR name ILIKE $3
             OR slug ILIKE $3
             OR account_type ILIKE $3
             OR support_tier ILIKE $3
           )
         ORDER BY updated_at DESC
         LIMIT 5",
    )
    .bind(target_organization_id)
    .bind(exact_query)
    .bind(like_query)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let id: i64 = row.get("id");
        results.push(AdminSupportSearchResult {
            category: "organization".into(),
            id: id.max(0) as u64,
            label: row.get("name"),
            detail: format!(
                "{} / {}",
                row.get::<String, _>("account_type"),
                row.get::<String, _>("status")
            ),
            organization_id: Some(id.max(0) as u64),
            href: Some(format!("/admin/support/search?target_organization_id={id}")),
            facts: vec![
                support_fact("Slug", row.get::<String, _>("slug")),
                support_fact("Support tier", row.get::<String, _>("support_tier")),
            ],
        });
    }

    Ok(())
}

fn resolve_support_target_organization_id(
    requested: Option<u64>,
    actor_organization_id: Option<i64>,
) -> Result<i64, StatusCode> {
    requested
        .map(|value| i64::try_from(value).map_err(|_| StatusCode::BAD_REQUEST))
        .transpose()?
        .or(actor_organization_id)
        .ok_or(StatusCode::FORBIDDEN)
}

async fn ensure_support_org_access(
    pool: &db::DbPool,
    session: &ResolvedSession,
    target_organization_id: i64,
) -> Result<(), StatusCode> {
    if ensure_admin_can_access_organization(
        pool,
        session,
        target_organization_id,
        "support_cross_org_access",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn ensure_audit_org_access(
    pool: &db::DbPool,
    session: &ResolvedSession,
    target_organization_id: i64,
) -> Result<(), StatusCode> {
    if ensure_admin_can_access_organization(
        pool,
        session,
        target_organization_id,
        "audit_search_cross_org",
        "organization",
        Some(target_organization_id.to_string()),
    )
    .await?
    {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn normalize_support_entity_type(value: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    let allowed = [
        "organization",
        "user",
        "load",
        "document",
        "payment",
        "tms_handoff",
        "support_case",
        "general",
    ];
    if allowed.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

fn normalize_support_note_visibility(value: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    match normalized.as_str() {
        "internal" | "customer_visible" => Ok(normalized),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn normalize_support_case_status(value: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    let allowed = [
        "new",
        "triage",
        "waiting_on_customer",
        "waiting_on_stloads",
        "escalated",
        "engineering_review",
        "product_review",
        "resolved",
        "closed",
        "reopened",
    ];
    if allowed.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

fn normalize_support_case_severity(value: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "");
    match normalized.as_str() {
        "sev1" | "severe1" | "1" => Ok("sev1".into()),
        "sev2" | "severe2" | "2" => Ok("sev2".into()),
        "sev3" | "severe3" | "3" => Ok("sev3".into()),
        "sev4" | "severe4" | "4" => Ok("sev4".into()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn normalize_support_case_key(value: &str, fallback: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    if normalized.is_empty() {
        Ok(fallback.into())
    } else if normalized.len() <= 64
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        Ok(normalized)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

fn non_empty_trimmed(value: &str, fallback: &str) -> Result<String, StatusCode> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        if fallback.is_empty() {
            Err(StatusCode::BAD_REQUEST)
        } else {
            Ok(fallback.into())
        }
    } else {
        Ok(trimmed.to_string())
    }
}

fn optional_u64_to_i64(value: Option<u64>) -> Result<Option<i64>, StatusCode> {
    value
        .map(|value| i64::try_from(value).map_err(|_| StatusCode::BAD_REQUEST))
        .transpose()
}

fn support_case_sla_deadlines(
    now: NaiveDateTime,
    severity: &str,
) -> (NaiveDateTime, NaiveDateTime, NaiveDateTime) {
    let (first_response_hours, next_update_hours, resolution_hours) = match severity {
        "sev1" => (1, 2, 8),
        "sev2" => (4, 8, 24),
        "sev3" => (8, 24, 72),
        _ => (24, 48, 168),
    };
    (
        now + chrono::Duration::hours(first_response_hours),
        now + chrono::Duration::hours(next_update_hours),
        now + chrono::Duration::hours(resolution_hours),
    )
}

fn support_case_breach_state(
    now: NaiveDateTime,
    first_response_due_at: &str,
    resolution_due_at: &str,
) -> &'static str {
    let first_response_due_at = parse_naive_datetime(first_response_due_at);
    let resolution_due_at = parse_naive_datetime(resolution_due_at);
    if first_response_due_at.is_some_and(|due| due < now)
        || resolution_due_at.is_some_and(|due| due < now)
    {
        "breached"
    } else if first_response_due_at.is_some_and(|due| due <= now + chrono::Duration::hours(4))
        || resolution_due_at.is_some_and(|due| due <= now + chrono::Duration::hours(24))
    {
        "at_risk"
    } else {
        "ok"
    }
}

fn parse_naive_datetime(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f").ok()
}

fn empty_support_case_screen(target_organization_id: Option<u64>) -> AdminSupportCaseScreen {
    AdminSupportCaseScreen {
        title: "Support Cases".into(),
        target_organization_id: target_organization_id.unwrap_or_default(),
        open_count: 0,
        breach_risk_count: 0,
        breached_count: 0,
        rows: Vec::new(),
        notes: Vec::new(),
    }
}

async fn load_support_case_screen(
    pool: &db::DbPool,
    target_organization_id: i64,
    status: Option<&str>,
) -> Result<AdminSupportCaseScreen, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, case_number, organization_id, title, severity, status, channel,
                category, owner_team, breach_state, first_response_due_at,
                next_update_due_at, resolution_due_at, related_entity_type,
                related_entity_id, customer_impact, resolution_reason,
                feedback_score, updated_at
         FROM support_cases
         WHERE organization_id = $1
           AND ($2::text IS NULL OR status = $2)
         ORDER BY
           CASE WHEN breach_state = 'breached' THEN 0 WHEN breach_state = 'at_risk' THEN 1 ELSE 2 END,
           updated_at DESC
         LIMIT 100",
    )
    .bind(target_organization_id)
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows: Vec<AdminSupportCaseRow> = rows.iter().map(support_case_row_from_sql).collect();

    let open_count = rows
        .iter()
        .filter(|row| row.status != "resolved" && row.status != "closed")
        .count() as u64;
    let breach_risk_count = rows
        .iter()
        .filter(|row| row.breach_state == "at_risk")
        .count() as u64;
    let breached_count = rows
        .iter()
        .filter(|row| row.breach_state == "breached")
        .count() as u64;

    Ok(AdminSupportCaseScreen {
        title: "Support Cases".into(),
        target_organization_id: target_organization_id.max(0) as u64,
        open_count,
        breach_risk_count,
        breached_count,
        rows,
        notes: vec![
            "Support cases are tenant-scoped and audited.".into(),
            "Customer-visible updates and internal notes are stored separately in the case event log.".into(),
            "Resolved cases can capture CSAT feedback for the product feedback loop.".into(),
        ],
    })
}

async fn load_support_case_row_by_id(
    pool: &db::DbPool,
    case_id: i64,
) -> Result<Option<AdminSupportCaseRow>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, case_number, organization_id, title, severity, status, channel,
                category, owner_team, breach_state, first_response_due_at,
                next_update_due_at, resolution_due_at, related_entity_type,
                related_entity_id, customer_impact, resolution_reason,
                feedback_score, updated_at
         FROM support_cases
         WHERE id = $1",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(row.as_ref().map(support_case_row_from_sql))
}

fn support_case_row_from_sql(row: &sqlx::postgres::PgRow) -> AdminSupportCaseRow {
    AdminSupportCaseRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        case_number: row.get("case_number"),
        organization_id: row.get::<i64, _>("organization_id").max(0) as u64,
        title: row.get("title"),
        severity: row.get("severity"),
        status: row.get("status"),
        channel: row.get("channel"),
        category: row.get("category"),
        owner_team: row.get("owner_team"),
        breach_state: row.get("breach_state"),
        first_response_due_at: row
            .get::<NaiveDateTime, _>("first_response_due_at")
            .to_string(),
        next_update_due_at: row
            .get::<NaiveDateTime, _>("next_update_due_at")
            .to_string(),
        resolution_due_at: row.get::<NaiveDateTime, _>("resolution_due_at").to_string(),
        related_entity_type: row.get("related_entity_type"),
        related_entity_id: row.get("related_entity_id"),
        customer_impact: row.get("customer_impact"),
        resolution_reason: row.get("resolution_reason"),
        feedback_score: row.get("feedback_score"),
        updated_at: row.get::<NaiveDateTime, _>("updated_at").to_string(),
    }
}

struct SupportCaseEventInput<'a> {
    event_type: &'a str,
    visibility: &'a str,
    previous_status: Option<&'a str>,
    new_status: Option<&'a str>,
    note: Option<&'a str>,
    customer_update: Option<&'a str>,
    internal_note: Option<&'a str>,
    feedback_score: Option<i32>,
}

async fn insert_support_case_event(
    case_id: i64,
    organization_id: i64,
    actor_user_id: Option<i64>,
    input: SupportCaseEventInput<'_>,
    pool: &db::DbPool,
) -> Result<(), StatusCode> {
    sqlx::query(
        "INSERT INTO support_case_events (
             support_case_id, organization_id, actor_user_id, event_type, visibility,
             previous_status, new_status, note, customer_update, internal_note,
             feedback_score, created_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP)",
    )
    .bind(case_id)
    .bind(organization_id)
    .bind(actor_user_id)
    .bind(input.event_type)
    .bind(input.visibility)
    .bind(input.previous_status)
    .bind(input.new_status)
    .bind(input.note)
    .bind(input.customer_update)
    .bind(input.internal_note)
    .bind(input.feedback_score)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn audit_support_case_action(
    pool: &db::DbPool,
    session: &ResolvedSession,
    target_organization_id: i64,
    case_id: i64,
    action: &str,
    reason: &str,
    request_id: Option<&str>,
) -> Result<(), StatusCode> {
    let case_id_string = case_id.to_string();
    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: auth_session::session_organization_id(session),
            target_organization_id: Some(target_organization_id),
            entity_type: "support_case",
            entity_id: Some(&case_id_string),
            action,
            reason: Some(reason),
            ticket_ref: None,
            request_id,
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({ "support_case_id": case_id })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

async fn support_case_mutation_response(
    pool: &db::DbPool,
    target_organization_id: i64,
    case_id: i64,
    message: &str,
) -> Result<Json<ApiResponse<AdminSupportCaseMutationResponse>>, StatusCode> {
    let case_row = load_support_case_row_by_id(pool, case_id).await?;
    let screen = load_support_case_screen(pool, target_organization_id, None).await?;
    Ok(Json(ApiResponse::ok(AdminSupportCaseMutationResponse {
        success: true,
        message: message.into(),
        case_row,
        screen,
    })))
}

fn audit_filters_from_query(query: &AdminAuditSearchQuery) -> AdminAuditSearchFilters {
    AdminAuditSearchFilters {
        q: clean_optional(query.q.clone()),
        target_organization_id: query.target_organization_id,
        actor_user_id: query.actor_user_id,
        entity_type: clean_optional(query.entity_type.clone()),
        entity_id: clean_optional(query.entity_id.clone()),
        action: clean_optional(query.action.clone()),
        request_id: clean_optional(query.request_id.clone()),
        date_from: clean_optional(query.date_from.clone()),
        date_to: clean_optional(query.date_to.clone()),
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn audit_export_path(filters: &AdminAuditSearchFilters) -> String {
    let mut params = Vec::new();
    if let Some(value) = filters.target_organization_id {
        params.push(format!("target_organization_id={value}"));
    }
    if let Some(value) = filters.actor_user_id {
        params.push(format!("actor_user_id={value}"));
    }
    push_query_param(&mut params, "q", filters.q.as_deref());
    push_query_param(&mut params, "entity_type", filters.entity_type.as_deref());
    push_query_param(&mut params, "entity_id", filters.entity_id.as_deref());
    push_query_param(&mut params, "action", filters.action.as_deref());
    push_query_param(&mut params, "request_id", filters.request_id.as_deref());
    push_query_param(&mut params, "date_from", filters.date_from.as_deref());
    push_query_param(&mut params, "date_to", filters.date_to.as_deref());
    if params.is_empty() {
        "/admin/audit/export".into()
    } else {
        format!("/admin/audit/export?{}", params.join("&"))
    }
}

fn push_query_param(params: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        params.push(format!("{}={}", key, percent_encode_query_value(value)));
    }
}

fn percent_encode_query_value(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{byte:02X}").chars().collect::<Vec<_>>(),
        })
        .collect()
}

fn parse_audit_date_start(value: Option<&String>) -> Result<Option<NaiveDateTime>, StatusCode> {
    let Some(value) = value
        .map(String::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
        .ok_or(StatusCode::BAD_REQUEST)
        .map(Some)
}

fn parse_audit_date_end(value: Option<&String>) -> Result<Option<NaiveDateTime>, StatusCode> {
    let Some(value) = value
        .map(String::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return Ok(None);
    };
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date
            .succ_opt()
            .and_then(|next| next.and_hms_opt(0, 0, 0))
            .ok_or(StatusCode::BAD_REQUEST)
            .map(Some);
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .map(Some)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn query_audit_events(
    pool: &db::DbPool,
    target_organization_id: i64,
    query: &AdminAuditSearchQuery,
    limit: i64,
) -> Result<Vec<AdminAuditEventRow>, StatusCode> {
    let q = clean_optional(query.q.clone()).map(|value| format!("%{}%", value));
    let actor_user_id = query
        .actor_user_id
        .map(|value| i64::try_from(value).map_err(|_| StatusCode::BAD_REQUEST))
        .transpose()?;
    let entity_type = clean_optional(query.entity_type.clone());
    let entity_id = clean_optional(query.entity_id.clone());
    let action = clean_optional(query.action.clone()).map(|value| format!("%{}%", value));
    let request_id = clean_optional(query.request_id.clone());
    let date_from = parse_audit_date_start(query.date_from.as_ref())?;
    let date_to = parse_audit_date_end(query.date_to.as_ref())?;

    let rows = sqlx::query(
        "SELECT ae.id, ae.actor_user_id, actor.name AS actor_name, actor.email AS actor_email,
                ae.organization_id, ae.target_organization_id, ae.entity_type, ae.entity_id,
                ae.action, ae.reason, ae.ticket_ref, ae.request_id, ae.source, ae.metadata,
                ae.before_state, ae.after_state, ae.created_at
         FROM audit_events ae
         LEFT JOIN users actor ON actor.id = ae.actor_user_id
         WHERE (ae.organization_id = $1 OR ae.target_organization_id = $1)
           AND ($2::bigint IS NULL OR ae.actor_user_id = $2)
           AND ($3::text IS NULL OR ae.entity_type = $3)
           AND ($4::text IS NULL OR ae.entity_id = $4)
           AND ($5::text IS NULL OR ae.action ILIKE $5)
           AND ($6::text IS NULL OR ae.request_id = $6)
           AND ($7::timestamp IS NULL OR ae.created_at >= $7)
           AND ($8::timestamp IS NULL OR ae.created_at < $8)
           AND (
               $9::text IS NULL
               OR ae.entity_type ILIKE $9
               OR ae.entity_id ILIKE $9
               OR ae.action ILIKE $9
               OR ae.reason ILIKE $9
               OR ae.ticket_ref ILIKE $9
               OR ae.request_id ILIKE $9
               OR ae.metadata::text ILIKE $9
               OR actor.email ILIKE $9
               OR actor.name ILIKE $9
           )
         ORDER BY ae.created_at DESC, ae.id DESC
         LIMIT $10",
    )
    .bind(target_organization_id)
    .bind(actor_user_id)
    .bind(entity_type.as_deref())
    .bind(entity_id.as_deref())
    .bind(action.as_deref())
    .bind(request_id.as_deref())
    .bind(date_from)
    .bind(date_to)
    .bind(q.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let actor_user_id = row.get::<Option<i64>, _>("actor_user_id");
            let actor_name = row.get::<Option<String>, _>("actor_name");
            let actor_email = row.get::<Option<String>, _>("actor_email");
            let metadata = row.get::<Option<serde_json::Value>, _>("metadata");
            let before_state = row.get::<Option<serde_json::Value>, _>("before_state");
            let after_state = row.get::<Option<serde_json::Value>, _>("after_state");
            AdminAuditEventRow {
                id: row.get::<i64, _>("id").max(0) as u64,
                actor_user_id: actor_user_id.map(|value| value.max(0) as u64),
                actor_label: audit_actor_label(actor_user_id, actor_name, actor_email),
                organization_id: row
                    .get::<Option<i64>, _>("organization_id")
                    .map(|value| value.max(0) as u64),
                target_organization_id: row
                    .get::<Option<i64>, _>("target_organization_id")
                    .map(|value| value.max(0) as u64),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                action: row.get("action"),
                reason: row.get("reason"),
                ticket_ref: row.get("ticket_ref"),
                request_id: row.get("request_id"),
                source: row.get("source"),
                metadata_preview: metadata.map(audit_json_preview),
                before_after_label: audit_before_after_label(
                    before_state.as_ref(),
                    after_state.as_ref(),
                ),
                created_at: row
                    .get::<chrono::NaiveDateTime, _>("created_at")
                    .to_string(),
            }
        })
        .collect())
}

fn audit_actor_label(
    actor_user_id: Option<i64>,
    actor_name: Option<String>,
    actor_email: Option<String>,
) -> String {
    match (actor_name, actor_email, actor_user_id) {
        (Some(name), Some(email), _) if !name.trim().is_empty() => format!("{name} <{email}>"),
        (_, Some(email), _) => email,
        (Some(name), _, _) if !name.trim().is_empty() => name,
        (_, _, Some(id)) => format!("User {id}"),
        _ => "System".into(),
    }
}

fn audit_json_preview(value: serde_json::Value) -> String {
    let text = value.to_string();
    if text.len() > 240 {
        format!("{}...", &text[..240])
    } else {
        text
    }
}

fn audit_before_after_label(
    before_state: Option<&serde_json::Value>,
    after_state: Option<&serde_json::Value>,
) -> String {
    match (before_state.is_some(), after_state.is_some()) {
        (true, true) => "before and after evidence".into(),
        (true, false) => "before evidence only".into(),
        (false, true) => "after evidence only".into(),
        (false, false) => "metadata only".into(),
    }
}

fn audit_rows_to_csv(rows: &[AdminAuditEventRow]) -> String {
    let mut csv = String::from(
        "id,created_at,actor_user_id,actor,organization_id,target_organization_id,entity_type,entity_id,action,reason,ticket_ref,request_id,source,evidence,metadata\n",
    );
    for row in rows {
        let fields = [
            row.id.to_string(),
            row.created_at.clone(),
            row.actor_user_id
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.actor_label.clone(),
            row.organization_id
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.target_organization_id
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.entity_type.clone(),
            row.entity_id.clone().unwrap_or_default(),
            row.action.clone(),
            row.reason.clone().unwrap_or_default(),
            row.ticket_ref.clone().unwrap_or_default(),
            row.request_id.clone().unwrap_or_default(),
            row.source.clone(),
            row.before_after_label.clone(),
            row.metadata_preview.clone().unwrap_or_default(),
        ];
        csv.push_str(
            &fields
                .into_iter()
                .map(csv_escape)
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
    }
    csv
}

fn csv_escape(value: String) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

async fn load_support_timeline(
    pool: &db::DbPool,
    target_organization_id: i64,
    entity_type: &str,
    entity_id: Option<&str>,
) -> Result<AdminSupportTimelineScreen, StatusCode> {
    let entity_type = normalize_support_entity_type(entity_type)?;
    let entity_id = entity_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let mut entries = Vec::new();

    let note_rows = sqlx::query(
        "SELECT actor_user_id, visibility, ticket_ref, note, created_at
         FROM support_notes
         WHERE organization_id = $1
           AND entity_type = $2
           AND (($3::text IS NULL AND entity_id IS NULL) OR entity_id = $3)
         ORDER BY created_at DESC
         LIMIT 25",
    )
    .bind(target_organization_id)
    .bind(&entity_type)
    .bind(entity_id.as_deref())
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for row in note_rows {
        entries.push(AdminSupportTimelineEntry {
            source: "support_note".into(),
            action: "note".into(),
            actor_user_id: row
                .get::<Option<i64>, _>("actor_user_id")
                .map(|value| value.max(0) as u64),
            visibility: row.get("visibility"),
            ticket_ref: row.get("ticket_ref"),
            summary: row.get("note"),
            created_at: row
                .get::<chrono::NaiveDateTime, _>("created_at")
                .to_string(),
        });
    }

    let audit_rows = sqlx::query(
        "SELECT actor_user_id, action, reason, ticket_ref, created_at
         FROM audit_events
         WHERE (organization_id = $1 OR target_organization_id = $1)
           AND entity_type = $2
           AND (($3::text IS NULL AND entity_id IS NULL) OR entity_id = $3)
         ORDER BY created_at DESC
         LIMIT 25",
    )
    .bind(target_organization_id)
    .bind(&entity_type)
    .bind(entity_id.as_deref())
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for row in audit_rows {
        entries.push(AdminSupportTimelineEntry {
            source: "audit_event".into(),
            action: row.get("action"),
            actor_user_id: row
                .get::<Option<i64>, _>("actor_user_id")
                .map(|value| value.max(0) as u64),
            visibility: "internal".into(),
            ticket_ref: row.get("ticket_ref"),
            summary: row
                .get::<Option<String>, _>("reason")
                .unwrap_or_else(|| "Audit event recorded.".into()),
            created_at: row
                .get::<chrono::NaiveDateTime, _>("created_at")
                .to_string(),
        });
    }

    entries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    entries.truncate(40);

    Ok(AdminSupportTimelineScreen {
        title: "Support Timeline".into(),
        target_organization_id: target_organization_id.max(0) as u64,
        entity_type,
        entity_id,
        entries,
        notes: vec![
            "Timeline entries combine support notes and audit events for the selected entity.".into(),
            "Internal notes are never customer-visible; customer-visible notes are explicitly labeled.".into(),
            "Impersonation is intentionally not enabled in this slice.".into(),
        ],
    })
}

async fn append_support_user_results(
    pool: &db::DbPool,
    target_organization_id: i64,
    exact_query: &str,
    like_query: &str,
    results: &mut Vec<AdminSupportSearchResult>,
) -> Result<(), StatusCode> {
    let rows = sqlx::query(
        "SELECT id, name, email, role_id, status, organization_id, company_name, phone_no
         FROM users
         WHERE organization_id = $1
           AND (
             id::text = $2
             OR name ILIKE $3
             OR email ILIKE $3
             OR COALESCE(company_name, '') ILIKE $3
             OR COALESCE(phone_no, '') ILIKE $3
           )
         ORDER BY updated_at DESC
         LIMIT 10",
    )
    .bind(target_organization_id)
    .bind(exact_query)
    .bind(like_query)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let id: i64 = row.get("id");
        let role_label = row
            .get::<Option<i16>, _>("role_id")
            .and_then(UserRole::from_legacy_id)
            .map(|role| role.label().to_string())
            .unwrap_or_else(|| "Unknown role".into());
        results.push(AdminSupportSearchResult {
            category: "user".into(),
            id: id.max(0) as u64,
            label: row.get("name"),
            detail: row.get("email"),
            organization_id: row
                .get::<Option<i64>, _>("organization_id")
                .map(|value| value.max(0) as u64),
            href: Some(format!("/admin/users/{id}/profile")),
            facts: vec![
                support_fact("Role", role_label),
                support_fact("Status", row.get::<i16, _>("status")),
                support_fact(
                    "Company",
                    row.get::<Option<String>, _>("company_name")
                        .unwrap_or_else(|| "None".into()),
                ),
                support_fact(
                    "Phone",
                    row.get::<Option<String>, _>("phone_no")
                        .unwrap_or_else(|| "None".into()),
                ),
            ],
        });
    }

    Ok(())
}

async fn append_support_load_results(
    pool: &db::DbPool,
    target_organization_id: i64,
    exact_query: &str,
    like_query: &str,
    results: &mut Vec<AdminSupportSearchResult>,
) -> Result<(), StatusCode> {
    let rows = sqlx::query(
        "SELECT id, load_number, title, status, user_id, organization_id
         FROM loads
         WHERE organization_id = $1
           AND deleted_at IS NULL
           AND (
             id::text = $2
             OR COALESCE(load_number, '') ILIKE $3
             OR title ILIKE $3
           )
         ORDER BY updated_at DESC
         LIMIT 10",
    )
    .bind(target_organization_id)
    .bind(exact_query)
    .bind(like_query)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let id: i64 = row.get("id");
        results.push(AdminSupportSearchResult {
            category: "load".into(),
            id: id.max(0) as u64,
            label: row.get("title"),
            detail: row
                .get::<Option<String>, _>("load_number")
                .unwrap_or_else(|| format!("Load #{id}")),
            organization_id: row
                .get::<Option<i64>, _>("organization_id")
                .map(|value| value.max(0) as u64),
            href: Some(format!("/loads/{id}")),
            facts: vec![
                support_fact("Status", row.get::<i16, _>("status")),
                support_fact(
                    "Owner user",
                    row.get::<Option<i64>, _>("user_id")
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "None".into()),
                ),
            ],
        });
    }

    Ok(())
}

async fn append_support_document_results(
    pool: &db::DbPool,
    target_organization_id: i64,
    exact_query: &str,
    like_query: &str,
    results: &mut Vec<AdminSupportSearchResult>,
) -> Result<(), StatusCode> {
    let rows = sqlx::query(
        "SELECT id, load_id, document_name, document_type, original_name, storage_provider, organization_id
         FROM load_documents
         WHERE organization_id = $1
           AND (
             id::text = $2
             OR load_id::text = $2
             OR document_name ILIKE $3
             OR COALESCE(original_name, '') ILIKE $3
             OR COALESCE(file_path, '') ILIKE $3
           )
         ORDER BY updated_at DESC
         LIMIT 10",
    )
    .bind(target_organization_id)
    .bind(exact_query)
    .bind(like_query)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let id: i64 = row.get("id");
        let load_id: i64 = row.get("load_id");
        results.push(AdminSupportSearchResult {
            category: "document".into(),
            id: id.max(0) as u64,
            label: row.get("document_name"),
            detail: row
                .get::<Option<String>, _>("original_name")
                .unwrap_or_else(|| format!("Load document #{id}")),
            organization_id: row
                .get::<Option<i64>, _>("organization_id")
                .map(|value| value.max(0) as u64),
            href: Some(format!("/loads/{load_id}")),
            facts: vec![
                support_fact("Load", load_id),
                support_fact("Type", row.get::<String, _>("document_type")),
                support_fact(
                    "Storage",
                    row.get::<Option<String>, _>("storage_provider")
                        .unwrap_or_else(|| "unknown".into()),
                ),
            ],
        });
    }

    Ok(())
}

async fn append_support_escrow_results(
    pool: &db::DbPool,
    target_organization_id: i64,
    exact_query: &str,
    like_query: &str,
    results: &mut Vec<AdminSupportSearchResult>,
) -> Result<(), StatusCode> {
    let rows = sqlx::query(
        "SELECT id, leg_id, status, currency, amount, payer_user_id, payee_user_id, payment_intent_id, transfer_id, organization_id
         FROM escrows
         WHERE organization_id = $1
           AND (
             id::text = $2
             OR leg_id::text = $2
             OR status ILIKE $3
             OR COALESCE(payment_intent_id, '') ILIKE $3
             OR COALESCE(transfer_id, '') ILIKE $3
           )
         ORDER BY updated_at DESC
         LIMIT 10",
    )
    .bind(target_organization_id)
    .bind(exact_query)
    .bind(like_query)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let id: i64 = row.get("id");
        results.push(AdminSupportSearchResult {
            category: "payment".into(),
            id: id.max(0) as u64,
            label: format!("Escrow #{id}"),
            detail: format!(
                "{} {}",
                row.get::<String, _>("currency"),
                row.get::<i64, _>("amount")
            ),
            organization_id: row
                .get::<Option<i64>, _>("organization_id")
                .map(|value| value.max(0) as u64),
            href: Some("/admin/payments".into()),
            facts: vec![
                support_fact("Status", row.get::<String, _>("status")),
                support_fact("Leg", row.get::<i64, _>("leg_id")),
                support_fact("Payer", row.get::<i64, _>("payer_user_id")),
                support_fact("Payee", row.get::<i64, _>("payee_user_id")),
            ],
        });
    }

    Ok(())
}

async fn append_support_tms_results(
    pool: &db::DbPool,
    target_organization_id: i64,
    exact_query: &str,
    like_query: &str,
    results: &mut Vec<AdminSupportSearchResult>,
) -> Result<(), StatusCode> {
    let rows = sqlx::query(
        "SELECT id, tms_load_id, external_handoff_id, load_id, status, tms_status, tenant_id, organization_id
         FROM stloads_handoffs
         WHERE organization_id = $1
           AND (
             id::text = $2
             OR COALESCE(load_id::text, '') = $2
             OR tms_load_id ILIKE $3
             OR COALESCE(external_handoff_id, '') ILIKE $3
             OR status ILIKE $3
             OR COALESCE(tms_status, '') ILIKE $3
           )
         ORDER BY updated_at DESC
         LIMIT 10",
    )
    .bind(target_organization_id)
    .bind(exact_query)
    .bind(like_query)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for row in rows {
        let id: i64 = row.get("id");
        results.push(AdminSupportSearchResult {
            category: "tms_handoff".into(),
            id: id.max(0) as u64,
            label: row.get("tms_load_id"),
            detail: row.get("status"),
            organization_id: row
                .get::<Option<i64>, _>("organization_id")
                .map(|value| value.max(0) as u64),
            href: Some("/admin/stloads/operations".into()),
            facts: vec![
                support_fact(
                    "External handoff",
                    row.get::<Option<String>, _>("external_handoff_id")
                        .unwrap_or_else(|| "None".into()),
                ),
                support_fact(
                    "Load",
                    row.get::<Option<i64>, _>("load_id")
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "None".into()),
                ),
                support_fact(
                    "TMS status",
                    row.get::<Option<String>, _>("tms_status")
                        .unwrap_or_else(|| "None".into()),
                ),
                support_fact(
                    "Tenant",
                    row.get::<Option<String>, _>("tenant_id")
                        .unwrap_or_else(|| "None".into()),
                ),
            ],
        });
    }

    Ok(())
}

fn support_fact(label: impl Into<String>, value: impl ToString) -> AdminSupportSearchFact {
    AdminSupportSearchFact {
        label: label.into(),
        value: value.to_string(),
    }
}

fn support_result_categories(results: &[AdminSupportSearchResult]) -> Vec<String> {
    let mut categories = results
        .iter()
        .map(|result| result.category.clone())
        .collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories
}

async fn load_identity_screen(
    pool: &db::DbPool,
    target_organization_id: i64,
) -> Result<AdminIdentityScreen, StatusCode> {
    let domain_rows = sqlx::query(
        "SELECT id, domain, verification_status, verification_token, login_routing_enabled, verified_at
         FROM organization_domains
         WHERE organization_id = $1
         ORDER BY domain",
    )
    .bind(target_organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let domains = domain_rows
        .into_iter()
        .map(|row| AdminIdentityDomainRow {
            id: row.get::<i64, _>("id").max(0) as u64,
            domain: row.get("domain"),
            verification_status: row.get("verification_status"),
            verification_token: row.get("verification_token"),
            login_routing_enabled: row.get("login_routing_enabled"),
            verified_at: row
                .get::<Option<chrono::NaiveDateTime>, _>("verified_at")
                .map(|value| value.to_string()),
        })
        .collect::<Vec<_>>();

    let provider_rows = sqlx::query(
        "SELECT id, provider_type, status, display_name, issuer, sso_url, jwks_url, metadata_url,
                client_id, jit_enabled, default_role_key
         FROM enterprise_identity_providers
         WHERE organization_id = $1
         ORDER BY updated_at DESC, id DESC",
    )
    .bind(target_organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let providers = provider_rows
        .into_iter()
        .map(|row| AdminIdentityProviderRow {
            id: row.get::<i64, _>("id").max(0) as u64,
            provider_type: row.get("provider_type"),
            status: row.get("status"),
            display_name: row.get("display_name"),
            issuer: row.get("issuer"),
            sso_url: row.get("sso_url"),
            jwks_url: row.get("jwks_url"),
            metadata_url: row.get("metadata_url"),
            client_id: row.get("client_id"),
            jit_enabled: row.get("jit_enabled"),
            default_role_key: row.get("default_role_key"),
        })
        .collect::<Vec<_>>();

    let scim_rows = sqlx::query(
        "SELECT id, action, outcome, external_id, user_id, reason, created_at
         FROM scim_events
         WHERE organization_id = $1
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(target_organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let scim_events = scim_rows
        .into_iter()
        .map(|row| AdminIdentityScimEventRow {
            id: row.get::<i64, _>("id").max(0) as u64,
            action: row.get("action"),
            outcome: row.get("outcome"),
            external_id: row.get("external_id"),
            user_id: row
                .get::<Option<i64>, _>("user_id")
                .map(|value| value.max(0) as u64),
            reason: row.get("reason"),
            created_at: row
                .get::<chrono::NaiveDateTime, _>("created_at")
                .to_string(),
        })
        .collect::<Vec<_>>();

    Ok(AdminIdentityScreen {
        title: "Enterprise Identity".into(),
        target_organization_id: target_organization_id.max(0) as u64,
        domains,
        providers,
        scim_events,
        notes: vec![
            "Domain verification gates enforced SSO routing.".into(),
            "Provider metadata is stored first; assertion validation and callback handling remain separate implementation work.".into(),
            "SCIM deprovisioning already revokes active Rust sessions.".into(),
        ],
    })
}

async fn load_access_review_screen(
    pool: &db::DbPool,
    target_organization_id: i64,
) -> Result<AdminAccessReviewScreen, StatusCode> {
    let reviews = list_access_reviews(pool, target_organization_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = latest_access_review_items(pool, target_organization_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut counts_by_review = HashMap::<i64, (u64, u64, u64, u64)>::new();
    if !reviews.is_empty() {
        let review_ids = reviews.iter().map(|review| review.id).collect::<Vec<_>>();
        let count_rows = sqlx::query(
            "SELECT review_id, decision, COUNT(*)::bigint AS total
             FROM access_review_items
             WHERE review_id = ANY($1)
             GROUP BY review_id, decision",
        )
        .bind(&review_ids)
        .fetch_all(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for row in count_rows {
            let review_id: i64 = row.get("review_id");
            let decision: String = row.get("decision");
            let total = row.get::<i64, _>("total").max(0) as u64;
            let entry = counts_by_review.entry(review_id).or_default();
            match decision.as_str() {
                "pending" => entry.0 = total,
                "approve" => entry.1 = total,
                "exception" => entry.2 = total,
                "revoke" => entry.3 = total,
                _ => {}
            }
        }
    }

    let review_rows = reviews
        .into_iter()
        .map(|review| {
            let (pending_count, approved_count, exception_count, revoke_count) = counts_by_review
                .get(&review.id)
                .copied()
                .unwrap_or_default();
            AdminAccessReviewRow {
                id: review.id.max(0) as u64,
                title: review.title,
                status: review.status,
                review_type: review.review_type,
                due_at_label: review.due_at.as_ref().map(format_datetime),
                completed_at_label: review.completed_at.as_ref().map(format_datetime),
                created_at_label: format_datetime(&review.created_at),
                pending_count,
                approved_count,
                exception_count,
                revoke_count,
            }
        })
        .collect::<Vec<_>>();

    let item_rows = items
        .into_iter()
        .map(|item| AdminAccessReviewItemRow {
            id: item.id.max(0) as u64,
            review_id: item.review_id.max(0) as u64,
            user_id: item.user_id.max(0) as u64,
            user_name: item.user_name,
            user_email: item.user_email,
            role_key: item.role_key,
            role_label: item.role_label,
            account_status_label: AccountStatus::from_legacy_code(item.account_status)
                .map(account_status_label)
                .unwrap_or_else(|| format!("Status {}", item.account_status)),
            membership_status: item.membership_status,
            last_activity_label: item.last_activity_at.as_ref().map(format_datetime),
            risk_flags: item.risk_flags,
            decision: item.decision,
            decision_reason: item.decision_reason,
            decided_at_label: item.decided_at.as_ref().map(format_datetime),
            revoked_at_label: item.revoked_at.as_ref().map(format_datetime),
        })
        .collect::<Vec<_>>();

    let elevation_rows = sqlx::query(
        "SELECT aer.id, aer.requester_user_id, requester.email AS requester_email,
            aer.target_user_id, target.email AS target_email, aer.current_role_key,
            aer.requested_role_key, aer.status, aer.business_justification,
            aer.decision_reason, aer.decided_at, aer.expires_at, aer.created_at
         FROM access_elevation_requests aer
         INNER JOIN users requester ON requester.id = aer.requester_user_id
         INNER JOIN users target ON target.id = aer.target_user_id
         WHERE aer.organization_id = $1
         ORDER BY
           CASE aer.status WHEN 'pending' THEN 0 ELSE 1 END,
           aer.created_at DESC
         LIMIT 25",
    )
    .bind(target_organization_id)
    .fetch_all(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|row| AdminAccessElevationRequestRow {
        id: row.get::<i64, _>("id").max(0) as u64,
        requester_user_id: row.get::<i64, _>("requester_user_id").max(0) as u64,
        requester_email: row.get("requester_email"),
        target_user_id: row.get::<i64, _>("target_user_id").max(0) as u64,
        target_email: row.get("target_email"),
        current_role_key: row.get("current_role_key"),
        requested_role_key: row.get("requested_role_key"),
        status: row.get("status"),
        business_justification: row.get("business_justification"),
        decision_reason: row.get("decision_reason"),
        decided_at_label: row
            .get::<Option<chrono::NaiveDateTime>, _>("decided_at")
            .as_ref()
            .map(format_datetime),
        expires_at_label: row
            .get::<Option<chrono::NaiveDateTime>, _>("expires_at")
            .as_ref()
            .map(format_datetime),
        created_at_label: format_datetime(&row.get::<chrono::NaiveDateTime, _>("created_at")),
    })
    .collect::<Vec<_>>();

    Ok(AdminAccessReviewScreen {
        title: "Access Reviews".into(),
        summary: if let Some(latest) = review_rows.first() {
            format!(
                "{} is {} with {} pending privileged access item(s).",
                latest.title, latest.status, latest.pending_count
            )
        } else {
            "No access review has been started for this organization yet.".into()
        },
        target_organization_id: target_organization_id.max(0) as u64,
        reviews: review_rows,
        items: item_rows,
        elevation_requests: elevation_rows,
        notes: vec![
            "Starting a review snapshots active privileged memberships, legacy admins, stale accounts, and active break-glass access.".into(),
            "Approve, exception, and revoke decisions are retained as audit evidence; revoke also removes local access artifacts.".into(),
            "A review closes automatically after every item has a decision.".into(),
        ],
    })
}

fn normalize_domain(value: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().trim_start_matches('@').to_ascii_lowercase();
    if normalized.len() < 3
        || normalized.len() > 255
        || normalized.contains('/')
        || normalized.contains(' ')
        || !normalized.contains('.')
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(normalized)
}

fn normalize_identity_provider_type(value: &str) -> Result<String, StatusCode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "oidc" | "saml" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn normalize_identity_provider_status(value: &str) -> Result<String, StatusCode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "draft" | "active" | "disabled" => Ok(value.trim().to_ascii_lowercase()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn normalize_access_review_decision(value: &str) -> Result<String, StatusCode> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "approve" | "approved" | "keep" => Ok("approve".into()),
        "exception" | "approve_exception" | "risk_accept" => Ok("exception".into()),
        "revoke" | "remove" | "disable" => Ok("revoke".into()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn normalize_organization_role_key(value: &str) -> Result<String, StatusCode> {
    let normalized = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    match normalized.as_str() {
        "owner" | "admin" | "finance" | "operator" | "support" | "integration_admin" | "member"
        | "auditor" => Ok(normalized),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn optional_trimmed(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[derive(Debug, Deserialize)]
struct DnsGoogleResponse {
    #[serde(rename = "Answer")]
    answer: Option<Vec<DnsGoogleAnswer>>,
}

#[derive(Debug, Deserialize)]
struct DnsGoogleAnswer {
    data: String,
}

async fn dns_txt_contains_token(name: &str, token: &str) -> Result<bool, String> {
    let url = format!("https://dns.google/resolve?name={name}&type=TXT");
    let response = reqwest::get(&url)
        .await
        .map_err(|error| format!("DNS query failed: {error}"))?
        .json::<DnsGoogleResponse>()
        .await
        .map_err(|error| format!("DNS response parse failed: {error}"))?;
    Ok(dns_answers_contain_token(response.answer.as_deref(), token))
}

fn dns_answers_contain_token(answers: Option<&[DnsGoogleAnswer]>, token: &str) -> bool {
    answers
        .unwrap_or_default()
        .iter()
        .any(|answer| normalize_dns_txt(&answer.data).contains(token))
}

fn normalize_dns_txt(value: &str) -> String {
    value.replace('"', "").replace("\\032", " ")
}

async fn ensure_admin_can_access_organization(
    pool: &db::DbPool,
    session: &ResolvedSession,
    target_organization_id: i64,
    action: &'static str,
    entity_type: &'static str,
    entity_id: Option<String>,
) -> Result<bool, StatusCode> {
    let actor_organization_id = auth_session::session_organization_id(session);
    if actor_organization_id == Some(target_organization_id) {
        return Ok(true);
    }

    let has_break_glass =
        has_active_break_glass_session(pool, session.user.id, target_organization_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !has_break_glass {
        return Ok(false);
    }

    insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: actor_organization_id,
            target_organization_id: Some(target_organization_id),
            entity_type,
            entity_id: entity_id.as_deref(),
            action,
            reason: Some("time-boxed break-glass access"),
            ticket_ref: None,
            request_id: None,
            ip_address: None,
            user_agent: None,
            source: "rust-backend",
            metadata: Some(serde_json::json!({
                "actor_email": session.user.email,
            })),
            before_state: None,
            after_state: None,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(true)
}

fn require_mfa_step_up(session: &ResolvedSession) -> Result<(), StatusCode> {
    if session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "mfa_verified")
    {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn send_account_review_notification(
    state: &AppState,
    user: &db::auth::UserRecord,
    status: AccountStatus,
    remarks: Option<&str>,
    request_id: Option<&str>,
) -> Option<String> {
    let role_label = user
        .primary_role()
        .map(|role| role.label().to_string())
        .unwrap_or_else(|| "STLoads".into());

    match state
        .email
        .send_account_review_status_with_request_id(
            &user.email,
            &user.name,
            &role_label,
            status,
            remarks,
            request_id,
        )
        .await
    {
        Ok(outcome) => outcome.status_note(),
        Err(error) => Some(format!("Account review email delivery failed: {}", error)),
    }
}

fn append_optional_note(base: impl Into<String>, note: Option<String>) -> String {
    let base = base.into();
    match note {
        Some(note) if !note.trim().is_empty() => format!("{} {}", base, note),
        _ => base,
    }
}

fn account_status_label(status: AccountStatus) -> String {
    match status {
        AccountStatus::EmailVerifiedPendingOnboarding => "Email Verified".into(),
        AccountStatus::Approved => "Approved".into(),
        AccountStatus::Rejected => "Rejected".into(),
        AccountStatus::PendingReview => "Pending Review".into(),
        AccountStatus::PendingOtp => "Pending OTP".into(),
        AccountStatus::RevisionRequested => "Revision Requested".into(),
    }
}

fn admin_role_options() -> Vec<AdminUserDirectoryRoleOption> {
    vec![
        AdminUserDirectoryRoleOption {
            key: "admin".into(),
            label: "Admin".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "shipper".into(),
            label: "Shipper".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "carrier".into(),
            label: "Carrier".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "broker".into(),
            label: "Broker".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "freight_forwarder".into(),
            label: "Freight Forwarder".into(),
        },
    ]
}

fn permission_option_catalog() -> Vec<AdminRolePermissionOption> {
    permission_descriptors()
        .iter()
        .map(|descriptor| AdminRolePermissionOption {
            key: admin_permission_key(descriptor.permission).into(),
            label: descriptor.label.into(),
            description: descriptor.description.into(),
        })
        .collect()
}

fn admin_status_options() -> Vec<AdminUserDirectoryStatusOption> {
    vec![
        AdminUserDirectoryStatusOption {
            key: "email_verified".into(),
            label: "Email Verified".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "approved".into(),
            label: "Approved".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "rejected".into(),
            label: "Rejected".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "pending_review".into(),
            label: "Pending Review".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "pending_otp".into(),
            label: "Pending OTP".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "revision_requested".into(),
            label: "Revision Requested".into(),
        },
    ]
}

fn normalize_admin_load_tab(value: Option<&str>) -> String {
    match value.unwrap_or("all").trim().to_ascii_lowercase().as_str() {
        "pending" => "pending".into(),
        "approved" => "approved".into(),
        "completed" => "completed".into(),
        "release-funds" | "release_funds" => "release-funds".into(),
        _ => "all".into(),
    }
}

fn admin_load_tab_statuses(active_tab: &str) -> Option<&'static [i16]> {
    match active_tab {
        "pending" => Some(&[1]),
        "approved" => Some(&[2, 3, 4, 5, 6, 8, 9]),
        "completed" => Some(&[11]),
        "release-funds" => Some(&[10]),
        _ => None,
    }
}

fn admin_load_tabs(
    active_tab: &str,
    all_count: u64,
    pending_count: u64,
    approved_count: u64,
    completed_count: u64,
    release_count: u64,
) -> Vec<AdminLoadTab> {
    vec![
        AdminLoadTab {
            key: "all".into(),
            label: "All Loads".into(),
            count: all_count,
            is_active: active_tab == "all",
        },
        AdminLoadTab {
            key: "pending".into(),
            label: "Pending Approval".into(),
            count: pending_count,
            is_active: active_tab == "pending",
        },
        AdminLoadTab {
            key: "approved".into(),
            label: "Approved / Active".into(),
            count: approved_count,
            is_active: active_tab == "approved",
        },
        AdminLoadTab {
            key: "completed".into(),
            label: "Completed".into(),
            count: completed_count,
            is_active: active_tab == "completed",
        },
        AdminLoadTab {
            key: "release-funds".into(),
            label: "Fund Release".into(),
            count: release_count,
            is_active: active_tab == "release-funds",
        },
    ]
}

fn map_admin_load_row(row: db::dispatch::AdminLoadLegRecord) -> AdminLoadRow {
    let amount = row.booked_amount.or(row.price);
    let payment_label = row
        .escrow_status
        .as_ref()
        .map(|status| match status.as_str() {
            "released" | "paid_out" => "Released".to_string(),
            "funded" => "Funded".to_string(),
            "pending" | "hold" => "Pending".to_string(),
            "unfunded" => "Unfunded".to_string(),
            other => other.replace('_', " "),
        });
    let (finance_action_key, finance_action_label, finance_action_enabled, finance_note) =
        admin_load_finance_action(
            row.status_id,
            row.escrow_status.as_deref(),
            row.carrier_name.is_some(),
        );
    let payments_action = finance_action_key
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "release".into());

    AdminLoadRow {
        load_id: row.load_id.max(0) as u64,
        leg_id: row.leg_id.max(0) as u64,
        status_code: row.status_id,
        leg_code: row
            .leg_code
            .unwrap_or_else(|| format!("LEG-{}", row.leg_id.max(0))),
        owner_label: row.owner_name.unwrap_or_else(|| "Unknown owner".into()),
        carrier_label: row.carrier_name,
        origin_label: row
            .pickup_location_name
            .unwrap_or_else(|| "Unknown pickup".into()),
        destination_label: row
            .delivery_location_name
            .unwrap_or_else(|| "Unknown delivery".into()),
        pickup_date_label: row
            .pickup_date
            .map(|value| value.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "TBD".into()),
        delivery_date_label: row
            .delivery_date
            .map(|value| value.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "TBD".into()),
        status_label: admin_load_status_label(row.status_id),
        status_tone: admin_load_status_tone(row.status_id).into(),
        bid_status_label: row
            .bid_status
            .map(|value| value.replace('_', " "))
            .unwrap_or_else(|| "Open".into()),
        amount_label: amount
            .map(|value| format!("${:.0}", value))
            .unwrap_or_else(|| "Rate unavailable".into()),
        payment_label,
        finance_action_key,
        finance_action_label,
        finance_action_enabled,
        finance_note,
        can_review: row.status_id == 1,
        primary_action_label: if row.status_id == 10 {
            Some("Open Payments".into())
        } else if matches!(row.status_id, 2 | 3 | 4 | 5 | 6 | 8 | 9 | 11) {
            Some("Track".into())
        } else {
            Some("Profile".into())
        },
        load_href: format!("/admin/loads/{}", row.load_id.max(0) as u64),
        track_href: matches!(row.status_id, 2 | 3 | 4 | 5 | 6 | 8 | 9 | 10 | 11)
            .then(|| format!("/execution/legs/{}", row.leg_id.max(0) as u64)),
        payments_href: (row.status_id == 10 || row.escrow_id.is_some()).then(|| {
            format!(
                "/admin/payments?leg_id={}&action={}&source=admin-loads&load_id={}",
                row.leg_id.max(0) as u64,
                payments_action,
                row.load_id.max(0) as u64
            )
        }),
    }
}

fn admin_load_finance_action(
    status_id: i16,
    escrow_status: Option<&str>,
    has_carrier: bool,
) -> (Option<String>, Option<String>, bool, Option<String>) {
    if !has_carrier {
        return (
            None,
            None,
            false,
            Some("Finance actions stay locked until a carrier is assigned.".into()),
        );
    }

    match escrow_status {
        Some("released" | "paid_out") => (
            None,
            None,
            false,
            Some("Funds already released for this leg.".into()),
        ),
        Some("funded") if status_id >= 10 => (
            Some("release".into()),
            Some("Release Funds".into()),
            true,
            Some("This leg is in the release-ready stage and can be paid out directly from the Rust admin list.".into()),
        ),
        Some("funded") => (
            Some("hold".into()),
            Some("Place On Hold".into()),
            true,
            Some("Escrow is funded, but the load is not yet fully release-ready.".into()),
        ),
        Some("pending" | "hold") => (
            Some("hold".into()),
            Some("Refresh Hold".into()),
            true,
            Some("This escrow already needs finance review, so hold flow is the safest direct action.".into()),
        ),
        Some("unfunded") | None if matches!(status_id, 4 | 5 | 6 | 8 | 9 | 10) => (
            Some("fund".into()),
            Some("Fund Escrow".into()),
            true,
            Some("This booked or live leg still needs the initial escrow funding step.".into()),
        ),
        Some(other) => (
            None,
            None,
            false,
            Some(format!(
                "Finance status is {}. Use the payments console for exception handling.",
                other.replace('_', " ")
            )),
        ),
        None => (None, None, false, None),
    }
}

fn admin_load_status_label(status_id: i16) -> String {
    match status_id {
        0 => "Rejected".into(),
        1 => "Pending Approval".into(),
        2 => "Approved".into(),
        7 => "Revision Requested".into(),
        _ => match domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id) {
            Some(domain::dispatch::LegacyLoadLegStatusCode::Reviewed) => "Reviewed".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::OfferReady) => "Offer Ready".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::Booked) => "Booked".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::PickupStarted) => {
                "Pickup Started".into()
            }
            Some(domain::dispatch::LegacyLoadLegStatusCode::AtPickup) => "At Pickup".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::InTransit) => "In Transit".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded) => {
                "Ready for Fund Release".into()
            }
            Some(domain::dispatch::LegacyLoadLegStatusCode::AtDelivery) => "At Delivery".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered) => "Completed".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut) => "Paid Out".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::Draft) => "Draft".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::New) => "New".into(),
            None => format!("Status {}", status_id),
        },
    }
}

fn admin_load_status_tone(status_id: i16) -> &'static str {
    match status_id {
        0 => "danger",
        1 => "warning",
        2 => "success",
        7 => "warning",
        10 => "primary",
        11 => "secondary",
        _ => match domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id) {
            Some(domain::dispatch::LegacyLoadLegStatusCode::Booked) => "primary",
            Some(
                domain::dispatch::LegacyLoadLegStatusCode::PickupStarted
                | domain::dispatch::LegacyLoadLegStatusCode::AtPickup
                | domain::dispatch::LegacyLoadLegStatusCode::InTransit
                | domain::dispatch::LegacyLoadLegStatusCode::AtDelivery,
            ) => "info",
            Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered) => "secondary",
            Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut) => "dark",
            Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded) => "primary",
            Some(domain::dispatch::LegacyLoadLegStatusCode::OfferReady) => "info",
            Some(domain::dispatch::LegacyLoadLegStatusCode::Reviewed) => "info",
            Some(domain::dispatch::LegacyLoadLegStatusCode::Draft) => "secondary",
            Some(domain::dispatch::LegacyLoadLegStatusCode::New) => "success",
            None => "secondary",
        },
    }
}

fn parse_admin_role_key(value: &str) -> Option<UserRole> {
    match value.trim().to_ascii_lowercase().as_str() {
        "admin" => Some(UserRole::Admin),
        "shipper" => Some(UserRole::Shipper),
        "carrier" => Some(UserRole::Carrier),
        "broker" => Some(UserRole::Broker),
        "freight_forwarder" | "freight forwarder" => Some(UserRole::FreightForwarder),
        _ => None,
    }
}

fn admin_role_key(role: UserRole) -> String {
    match role {
        UserRole::Admin => "admin",
        UserRole::Shipper => "shipper",
        UserRole::Carrier => "carrier",
        UserRole::Broker => "broker",
        UserRole::FreightForwarder => "freight_forwarder",
    }
    .into()
}

fn admin_permission_key(permission: domain::auth::Permission) -> &'static str {
    match permission {
        domain::auth::Permission::AccessAdminPortal => "access_admin_portal",
        domain::auth::Permission::ManageUsers => "manage_users",
        domain::auth::Permission::ManageRoles => "manage_roles",
        domain::auth::Permission::ManageMasterData => "manage_master_data",
        domain::auth::Permission::ManageLoads => "manage_loads",
        domain::auth::Permission::ManageDispatchDesk => "manage_dispatch_desk",
        domain::auth::Permission::ManageMarketplace => "manage_marketplace",
        domain::auth::Permission::ManageTracking => "manage_tracking",
        domain::auth::Permission::ManagePayments => "manage_payments",
        domain::auth::Permission::ManageTmsOperations => "manage_tms_operations",
    }
}

fn parse_admin_status_key(value: &str) -> Option<AccountStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "email_verified" => Some(AccountStatus::EmailVerifiedPendingOnboarding),
        "approved" => Some(AccountStatus::Approved),
        "rejected" => Some(AccountStatus::Rejected),
        "pending_review" => Some(AccountStatus::PendingReview),
        "pending_otp" => Some(AccountStatus::PendingOtp),
        "revision_requested" => Some(AccountStatus::RevisionRequested),
        _ => None,
    }
}

fn admin_status_key(status: AccountStatus) -> String {
    match status {
        AccountStatus::EmailVerifiedPendingOnboarding => "email_verified",
        AccountStatus::Approved => "approved",
        AccountStatus::Rejected => "rejected",
        AccountStatus::PendingReview => "pending_review",
        AccountStatus::PendingOtp => "pending_otp",
        AccountStatus::RevisionRequested => "revision_requested",
    }
    .into()
}

fn admin_personal_facts(user: &db::auth::UserRecord) -> Vec<AdminUserProfileFact> {
    let mut facts = Vec::new();

    push_profile_fact(&mut facts, "Phone", user.phone_no.clone());
    push_profile_fact(
        &mut facts,
        "Date of birth",
        user.dob.map(|value| value.format("%b %d, %Y").to_string()),
    );
    push_profile_fact(&mut facts, "Gender", user.gender.clone());
    push_profile_fact(&mut facts, "Address", user.address.clone());
    push_profile_fact(&mut facts, "Nationality", user.nationality.clone());
    push_profile_fact(
        &mut facts,
        "Joined",
        Some(format_datetime(&user.created_at)),
    );

    facts
}

fn admin_company_facts(
    user: &db::auth::UserRecord,
    details: Option<&db::auth::UserDetailRecord>,
) -> Vec<AdminUserProfileFact> {
    let mut facts = Vec::new();

    let detail_company_name = details.and_then(|detail| detail.company_name.clone());
    let detail_company_address = details.and_then(|detail| detail.company_address.clone());
    push_profile_fact(
        &mut facts,
        "Company",
        detail_company_name.or(user.company_name.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Company address",
        detail_company_address.or(user.company_address.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Registration number",
        user.registration_number.clone(),
    );
    push_profile_fact(&mut facts, "Tax ID", user.tax_id.clone());
    push_profile_fact(
        &mut facts,
        "Country of incorporation",
        user.country_of_incorporation.clone(),
    );
    push_profile_fact(
        &mut facts,
        "DOT number",
        details.and_then(|detail| detail.dot_number.clone()),
    );
    push_profile_fact(
        &mut facts,
        "MC number",
        details
            .and_then(|detail| detail.mc_number.clone())
            .or(user.mc_number.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Equipment types",
        details.and_then(|detail| detail.equipment_types.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Business entity ID",
        details.and_then(|detail| detail.business_entity_id.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Facility address",
        details.and_then(|detail| detail.facility_address.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Fulfillment contact",
        details.and_then(|detail| detail.fulfillment_contact_info.clone()),
    );
    push_profile_fact(&mut facts, "USDOT", user.usdot_number.clone());
    push_profile_fact(&mut facts, "CDL number", user.cdl_number.clone());
    push_profile_fact(&mut facts, "Vehicle", user.vehicle_make_model.clone());
    push_profile_fact(&mut facts, "Policy number", user.policy_number.clone());
    push_profile_fact(&mut facts, "Transport modes", user.transport_modes.clone());
    push_profile_fact(
        &mut facts,
        "Countries served",
        user.countries_served.clone(),
    );
    push_profile_fact(
        &mut facts,
        "Years in operation",
        user.years_in_operation.clone(),
    );

    facts
}

fn push_profile_fact(facts: &mut Vec<AdminUserProfileFact>, label: &str, value: Option<String>) {
    if let Some(value) = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        facts.push(AdminUserProfileFact {
            label: label.into(),
            value,
        });
    }
}

fn format_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_session;
    use crate::test_support::{
        auth_headers_for_user, insert_load_fixture, insert_user_with_role_status, prepare_pool,
        read_leg_status, test_state,
    };
    use domain::auth::{AccountStatus, UserRole};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn review_user_handler_updates_status_and_reports_email_delivery()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Admin Reviewer",
            "admin-reviewer@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let review_target = insert_user_with_role_status(
            &pool,
            "Pending Carrier",
            "pending-carrier@example.com",
            UserRole::Carrier,
            AccountStatus::PendingReview,
        )
        .await?;
        let admin_headers = auth_headers_for_user(&state, &admin_user).await?;

        let response = review_user_handler(
            State(state),
            Path(review_target.id),
            admin_headers,
            Json(ReviewOnboardingRequest {
                decision: "revision".into(),
                remarks: Some("Need clearer carrier compliance detail.".into()),
            }),
        )
        .await
        .expect("admin review user request should succeed")
        .0
        .data;

        assert!(response.success);
        assert_eq!(response.status_label.as_deref(), Some("Revision Requested"));
        assert!(response.message.contains("Email notification logged"));
        let updated = db::auth::find_user_by_id(&pool, review_target.id)
            .await?
            .expect("review target still exists");
        assert_eq!(
            updated.account_status(),
            Some(AccountStatus::RevisionRequested)
        );

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn review_load_handler_updates_legs_and_reports_email_delivery()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Admin Load Reviewer",
            "admin-load-reviewer@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let fixture = insert_load_fixture(&pool, 1).await?;
        let admin_headers = auth_headers_for_user(&state, &admin_user).await?;

        let response = review_load_handler(
            State(state),
            Path(fixture.load_id),
            admin_headers,
            Json(AdminReviewLoadRequest {
                decision: "approve".into(),
                remarks: Some("Approved from backend route acceptance test.".into()),
            }),
        )
        .await
        .expect("admin review load request should succeed")
        .0
        .data;

        assert!(response.success);
        assert_eq!(response.status_label, "Approved");
        assert!(response.message.contains("Email notification logged"));
        assert_eq!(read_leg_status(&pool, fixture.leg_id).await?, 2);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn break_glass_allows_audited_cross_org_user_profile()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Break Glass Admin",
            "break-glass-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let target_user = insert_user_with_role_status(
            &pool,
            "Other Tenant Carrier",
            "other-tenant-carrier@example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        let target_org_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO organizations (name, slug, account_type, status, support_tier, created_at, updated_at)
             VALUES ('Other Tenant', 'other-tenant-break-glass', 'customer', 'active', 'enterprise', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await?;
        sqlx::query("UPDATE users SET organization_id = $1 WHERE id = $2")
            .bind(target_org_id)
            .bind(target_user.id)
            .execute(&pool)
            .await?;

        let normal_headers = auth_headers_for_user(&state, &admin_user).await?;
        let forbidden =
            user_profile(State(state.clone()), Path(target_user.id), normal_headers).await;
        assert!(matches!(forbidden, Err(StatusCode::FORBIDDEN)));

        let mfa_token =
            auth_session::issue_session_token_with_mfa(&state, &admin_user, true).await?;
        let mut mfa_headers = HeaderMap::new();
        mfa_headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", mfa_token))?,
        );

        let break_glass = start_break_glass_handler(
            State(state.clone()),
            mfa_headers.clone(),
            Json(AdminBreakGlassRequest {
                target_organization_id: target_org_id as u64,
                reason: "Customer support incident requires profile inspection.".into(),
                ticket_ref: "SUP-1001".into(),
                duration_minutes: Some(15),
            }),
        )
        .await
        .map_err(|status| format!("break-glass request failed with status {status}"))?
        .0
        .data;
        assert!(break_glass.success);
        assert!(break_glass.session_id.is_some());

        let allowed = user_profile(State(state), Path(target_user.id), mfa_headers)
            .await
            .map_err(|status| format!("profile request failed with status {status}"))?;
        assert_eq!(allowed.0.data.user_id, target_user.id as u64);

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND target_organization_id = $2
               AND action IN ('break_glass_started', 'admin_user_profile_viewed')",
        )
        .bind(admin_user.id)
        .bind(target_org_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 2);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn support_search_is_scoped_and_audited() -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Support Search Admin",
            "support-search-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let target_user = insert_user_with_role_status(
            &pool,
            "Searchable Carrier",
            "searchable-carrier@example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        let headers = auth_headers_for_user(&state, &admin_user).await?;

        let response = support_search(
            State(state),
            Query(AdminSupportSearchQuery {
                q: Some("searchable-carrier".into()),
                target_organization_id: None,
            }),
            headers,
        )
        .await
        .map_err(|status| format!("support search failed with status {status}"))?
        .0
        .data;

        assert!(response.result_count >= 1);
        assert!(response.results.iter().any(|result| {
            result.category == "user" && result.id == target_user.id.max(0) as u64
        }));

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND action = 'support_search_performed'
               AND entity_type = 'support_search'",
        )
        .bind(admin_user.id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 1);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn audit_search_filters_by_entity_and_exports_csv()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Audit Search Admin",
            "audit-search-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let headers = auth_headers_for_user(&state, &admin_user).await?;
        let request_id = "req_audit_search_test";
        let event_id = insert_audit_event(
            &pool,
            &AuditEventInput {
                actor_user_id: Some(admin_user.id),
                organization_id: Some(admin_user.organization_id),
                target_organization_id: Some(admin_user.organization_id),
                entity_type: "load",
                entity_id: Some("9001"),
                action: "load_status_approved",
                reason: Some("Compliance evidence for audit search test."),
                ticket_ref: Some("AUD-3001"),
                request_id: Some(request_id),
                ip_address: None,
                user_agent: None,
                source: "rust-backend-test",
                metadata: Some(serde_json::json!({
                    "load_id": 9001,
                    "document_id": 7001,
                    "payment_id": 6001,
                    "tms_handoff_id": 5001
                })),
                before_state: Some(serde_json::json!({"status": "pending"})),
                after_state: Some(serde_json::json!({"status": "approved"})),
            },
        )
        .await?;

        let search = audit_search(
            State(state.clone()),
            Query(AdminAuditSearchQuery {
                q: Some("document_id".into()),
                target_organization_id: None,
                actor_user_id: Some(admin_user.id as u64),
                entity_type: Some("load".into()),
                entity_id: Some("9001".into()),
                action: Some("approved".into()),
                request_id: Some(request_id.into()),
                date_from: None,
                date_to: None,
            }),
            headers.clone(),
        )
        .await
        .map_err(|status| format!("audit search failed with status {status}"))?
        .0
        .data;

        assert_eq!(search.result_count, 1);
        assert_eq!(search.rows[0].id, event_id as u64);
        assert_eq!(search.rows[0].request_id.as_deref(), Some(request_id));
        assert_eq!(
            search.rows[0].before_after_label,
            "before and after evidence"
        );
        assert!(search.export_path.contains("/admin/audit/export"));

        let export = audit_export(
            State(state),
            Query(AdminAuditSearchQuery {
                q: Some("document_id".into()),
                target_organization_id: None,
                actor_user_id: Some(admin_user.id as u64),
                entity_type: Some("load".into()),
                entity_id: Some("9001".into()),
                action: Some("approved".into()),
                request_id: Some(request_id.into()),
                date_from: None,
                date_to: None,
            }),
            headers,
        )
        .await
        .map_err(|status| format!("audit export failed with status {status}"))?
        .0
        .data;

        assert_eq!(export.row_count, 1);
        assert!(export.filename.starts_with("audit-export-org-"));
        assert!(export.csv.contains("load_status_approved"));
        assert!(export.csv.contains(request_id));

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn support_note_is_persisted_timeline_returned_and_audited()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Support Note Admin",
            "support-note-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let target_user = insert_user_with_role_status(
            &pool,
            "Timeline Carrier",
            "timeline-carrier@example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        let headers = auth_headers_for_user(&state, &admin_user).await?;

        let response = create_support_note(
            State(state.clone()),
            headers.clone(),
            Json(AdminCreateSupportNoteRequest {
                target_organization_id: None,
                entity_type: "user".into(),
                entity_id: Some(target_user.id.to_string()),
                visibility: "internal".into(),
                ticket_ref: Some("SUP-2001".into()),
                note: "Carrier asked why their onboarding status changed.".into(),
            }),
        )
        .await
        .map_err(|status| format!("support note request failed with status {status}"))?
        .0
        .data;

        assert!(response.success);
        assert!(response.note_id.is_some());
        assert!(response.timeline.entries.iter().any(|entry| {
            entry.source == "support_note"
                && entry.ticket_ref.as_deref() == Some("SUP-2001")
                && entry.summary.contains("onboarding status")
        }));
        assert!(response.timeline.entries.iter().any(|entry| {
            entry.source == "audit_event" && entry.action == "support_note_created"
        }));

        let timeline = support_timeline(
            State(state),
            Query(AdminSupportTimelineRequest {
                target_organization_id: None,
                entity_type: "user".into(),
                entity_id: Some(target_user.id.to_string()),
            }),
            headers,
        )
        .await
        .map_err(|status| format!("support timeline request failed with status {status}"))?
        .0
        .data;
        assert!(timeline.entries.iter().any(|entry| {
            entry.source == "support_note" && entry.ticket_ref.as_deref() == Some("SUP-2001")
        }));

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND action = 'support_note_created'
               AND entity_type = 'user'
               AND entity_id = $2",
        )
        .bind(admin_user.id)
        .bind(target_user.id.to_string())
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 1);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn support_case_tracks_sla_updates_customer_notes_and_feedback()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Support Case Admin",
            "support-case-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let target_user = insert_user_with_role_status(
            &pool,
            "Support Case Shipper",
            "support-case-shipper@example.com",
            UserRole::Shipper,
            AccountStatus::Approved,
        )
        .await?;
        let headers = auth_headers_for_user(&state, &admin_user).await?;

        let created = create_support_case(
            State(state.clone()),
            headers.clone(),
            Json(AdminCreateSupportCaseRequest {
                target_organization_id: None,
                reporter_user_id: Some(target_user.id as u64),
                affected_user_id: Some(target_user.id as u64),
                related_entity_type: Some("user".into()),
                related_entity_id: Some(target_user.id.to_string()),
                channel: "portal".into(),
                severity: "sev2".into(),
                category: "payments".into(),
                owner_team: "Support".into(),
                title: "Invoice question during pilot".into(),
                description: "Customer needs help understanding an invoice state.".into(),
                customer_impact: "Finance user cannot reconcile a pilot invoice.".into(),
                customer_update: Some("We are reviewing the invoice state.".into()),
                internal_note: Some("Check payment ledger before responding.".into()),
            }),
        )
        .await
        .map_err(|status| format!("support case create failed with status {status}"))?
        .0
        .data;
        assert!(created.success);
        let case_row = created.case_row.expect("created case row");
        assert_eq!(case_row.status, "new");
        assert_eq!(case_row.severity, "sev2");
        assert_eq!(created.screen.open_count, 1);

        let listed = support_cases(
            State(state.clone()),
            Query(AdminSupportCaseQuery {
                target_organization_id: None,
                status: Some("new".into()),
            }),
            headers.clone(),
        )
        .await
        .map_err(|status| format!("support case list failed with status {status}"))?
        .0
        .data;
        assert_eq!(listed.rows.len(), 1);
        assert_eq!(listed.rows[0].id, case_row.id);

        let updated = update_support_case(
            State(state.clone()),
            Path(case_row.id as i64),
            headers.clone(),
            Json(AdminUpdateSupportCaseRequest {
                status: Some("resolved".into()),
                severity: None,
                owner_team: Some("Product Support".into()),
                owner_user_id: Some(admin_user.id as u64),
                escalation_owner_user_id: None,
                customer_update: Some("Invoice state was explained and reconciled.".into()),
                internal_note: Some("Linked to product feedback for invoice copy.".into()),
                resolution_reason: Some("customer_educated".into()),
                root_cause_category: Some("confusing_invoice_copy".into()),
                follow_up_action: Some("Review invoice label language.".into()),
            }),
        )
        .await
        .map_err(|status| format!("support case update failed with status {status}"))?
        .0
        .data;
        assert!(updated.success);
        let updated_row = updated.case_row.expect("updated case row");
        assert_eq!(updated_row.status, "resolved");
        assert_eq!(
            updated_row.resolution_reason.as_deref(),
            Some("customer_educated")
        );

        let feedback = record_support_case_feedback(
            State(state),
            Path(case_row.id as i64),
            headers,
            Json(AdminSupportCaseFeedbackRequest {
                feedback_score: 5,
                feedback_comment: Some("Helpful support response.".into()),
            }),
        )
        .await
        .map_err(|status| format!("support case feedback failed with status {status}"))?
        .0
        .data;
        assert_eq!(
            feedback.case_row.expect("feedback case row").feedback_score,
            Some(5)
        );

        let customer_visible_events = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM support_case_events
             WHERE support_case_id = $1
               AND visibility = 'customer_visible'
               AND customer_update IS NOT NULL",
        )
        .bind(case_row.id as i64)
        .fetch_one(&pool)
        .await?;
        assert_eq!(customer_visible_events, 1);

        let internal_events = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM support_case_events
             WHERE support_case_id = $1
               AND internal_note IS NOT NULL",
        )
        .bind(case_row.id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(internal_events >= 2);

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND entity_type = 'support_case'
               AND entity_id = $2
               AND action IN (
                   'support_case_created',
                   'support_case_updated',
                   'support_case_feedback_recorded'
               )",
        )
        .bind(admin_user.id)
        .bind(case_row.id.to_string())
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 3);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn identity_admin_can_register_verify_domain_and_save_provider()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Identity Admin",
            "identity-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let headers = auth_headers_for_user(&state, &admin_user).await?;

        let domain_response = upsert_identity_domain(
            State(state.clone()),
            headers.clone(),
            Json(AdminUpsertIdentityDomainRequest {
                target_organization_id: None,
                domain: "Example-SSO.COM".into(),
                login_routing_enabled: true,
            }),
        )
        .await
        .map_err(|status| format!("identity domain save failed with status {status}"))?
        .0
        .data;
        assert!(domain_response.success);
        let domain = domain_response
            .screen
            .domains
            .iter()
            .find(|domain| domain.domain == "example-sso.com")
            .expect("saved domain");
        assert_eq!(domain.verification_status, "pending");
        let verification_token = domain.verification_token.clone();

        let verify_response = verify_identity_domain(
            State(state.clone()),
            headers.clone(),
            Json(AdminVerifyIdentityDomainRequest {
                target_organization_id: None,
                domain: "example-sso.com".into(),
                verification_token,
            }),
        )
        .await
        .map_err(|status| format!("identity domain verify failed with status {status}"))?
        .0
        .data;
        assert!(verify_response.success);
        assert!(
            verify_response
                .screen
                .domains
                .iter()
                .any(|domain| domain.domain == "example-sso.com"
                    && domain.verification_status == "verified"
                    && domain.login_routing_enabled)
        );

        let provider_response = upsert_identity_provider(
            State(state.clone()),
            headers.clone(),
            Json(AdminUpsertIdentityProviderRequest {
                target_organization_id: None,
                provider_id: None,
                provider_type: "oidc".into(),
                status: "active".into(),
                display_name: "Example Workforce IdP".into(),
                issuer: Some("https://idp.example-sso.com".into()),
                sso_url: Some("https://idp.example-sso.com/login".into()),
                jwks_url: Some("https://idp.example-sso.com/.well-known/jwks.json".into()),
                metadata_url: None,
                client_id: Some("stloads-client".into()),
                jit_enabled: true,
                default_role_key: "member".into(),
            }),
        )
        .await
        .map_err(|status| format!("identity provider save failed with status {status}"))?
        .0
        .data;
        assert!(provider_response.success);
        assert!(provider_response.screen.providers.iter().any(|provider| {
            provider.provider_type == "oidc"
                && provider.status == "active"
                && provider.display_name == "Example Workforce IdP"
                && provider.jit_enabled
        }));

        let screen = identity_screen(
            State(state),
            Query(AdminIdentityQuery {
                target_organization_id: None,
            }),
            headers,
        )
        .await
        .map_err(|status| format!("identity screen failed with status {status}"))?
        .0
        .data;
        assert_eq!(screen.domains.len(), 1);
        assert_eq!(screen.providers.len(), 1);

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND action IN ('identity_domain_upserted', 'identity_provider_upserted')",
        )
        .bind(admin_user.id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 2);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn access_review_snapshots_privileged_users_and_revokes_access()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let reviewer = insert_user_with_role_status(
            &pool,
            "Access Review Owner",
            "access-review-owner@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let privileged_user = insert_user_with_role_status(
            &pool,
            "Privileged Target",
            "privileged-target@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let target_token = auth_session::issue_session_token(&state, &privileged_user).await?;

        let reviewer_token =
            auth_session::issue_session_token_with_mfa(&state, &reviewer, true).await?;
        let mut reviewer_headers = HeaderMap::new();
        reviewer_headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", reviewer_token))?,
        );

        let started = start_access_review(
            State(state.clone()),
            reviewer_headers.clone(),
            Json(AdminStartAccessReviewRequest {
                target_organization_id: None,
                title: "Quarterly privileged access review".into(),
                due_days: Some(10),
            }),
        )
        .await
        .map_err(|status| format!("start access review failed with status {status}"))?
        .0
        .data;
        assert!(started.success);
        let target_item = started
            .screen
            .items
            .iter()
            .find(|item| item.user_id == privileged_user.id as u64)
            .expect("privileged target should be snapshotted");
        assert!(
            target_item
                .risk_flags
                .iter()
                .any(|flag| flag == "privileged_role")
        );

        let decided = decide_access_review(
            State(state.clone()),
            Path(target_item.id as i64),
            reviewer_headers,
            Json(AdminAccessReviewDecisionRequest {
                target_organization_id: None,
                decision: "revoke".into(),
                reason: Some("No longer requires administrator access.".into()),
            }),
        )
        .await
        .map_err(|status| format!("decide access review failed with status {status}"))?
        .0
        .data;
        assert!(decided.success);
        assert!(decided.screen.items.iter().any(|item| {
            item.user_id == privileged_user.id as u64 && item.decision == "revoke"
        }));

        let updated_user = db::auth::find_user_by_id(&pool, privileged_user.id)
            .await?
            .expect("target user still exists");
        assert_eq!(updated_user.account_status(), Some(AccountStatus::Rejected));
        assert!(
            auth_session::resolve_session_from_token(&state, &target_token)
                .await?
                .is_none()
        );

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND action IN ('access_review_started', 'access_review_item_decided')",
        )
        .bind(reviewer.id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 2);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn access_elevation_request_requires_approval_and_rotates_access()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let reviewer = insert_user_with_role_status(
            &pool,
            "Elevation Reviewer",
            "elevation-reviewer@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let target_user = insert_user_with_role_status(
            &pool,
            "Elevation Target",
            "elevation-target@example.com",
            UserRole::Carrier,
            AccountStatus::Approved,
        )
        .await?;
        let target_token = auth_session::issue_session_token(&state, &target_user).await?;

        let reviewer_token =
            auth_session::issue_session_token_with_mfa(&state, &reviewer, true).await?;
        let mut reviewer_headers = HeaderMap::new();
        reviewer_headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {}", reviewer_token))?,
        );

        let created = create_access_elevation_request(
            State(state.clone()),
            reviewer_headers.clone(),
            Json(AdminCreateAccessElevationRequest {
                target_organization_id: None,
                target_user_id: target_user.id as u64,
                requested_role_key: "finance".into(),
                business_justification: "Temporary billing investigation coverage.".into(),
                expires_in_days: Some(7),
            }),
        )
        .await
        .map_err(|status| format!("create elevation request failed with status {status}"))?
        .0
        .data;
        assert!(created.success);
        let request = created
            .screen
            .elevation_requests
            .iter()
            .find(|request| request.target_user_id == target_user.id as u64)
            .expect("elevation request should be visible");
        assert_eq!(request.status, "pending");

        let decided = decide_access_elevation_request(
            State(state.clone()),
            Path(request.id as i64),
            reviewer_headers,
            Json(AdminAccessElevationDecisionRequest {
                target_organization_id: None,
                decision: "approve".into(),
                reason: Some("Coverage approved for billing exception queue.".into()),
            }),
        )
        .await
        .map_err(|status| format!("decide elevation request failed with status {status}"))?
        .0
        .data;
        assert!(decided.success);
        assert!(decided.screen.elevation_requests.iter().any(|request| {
            request.target_user_id == target_user.id as u64 && request.status == "approved"
        }));

        let role_key = sqlx::query_scalar::<_, String>(
            "SELECT role_key
             FROM organization_memberships
             WHERE user_id = $1
               AND organization_id = $2",
        )
        .bind(target_user.id)
        .bind(target_user.organization_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(role_key, "finance");
        assert!(
            auth_session::resolve_session_from_token(&state, &target_token)
                .await?
                .is_none()
        );

        let audit_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM audit_events
             WHERE actor_user_id = $1
               AND action IN ('access_elevation_requested', 'access_elevation_decided')",
        )
        .bind(reviewer.id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(audit_count, 2);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn audit_ledger_is_append_only_and_stores_before_after_evidence()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let admin_user = insert_user_with_role_status(
            &pool,
            "Audit Ledger Admin",
            "audit-ledger-admin@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;

        let entity_id = admin_user.id.to_string();
        let event_id = insert_audit_event(
            &pool,
            &AuditEventInput {
                actor_user_id: Some(admin_user.id),
                organization_id: Some(admin_user.organization_id),
                target_organization_id: Some(admin_user.organization_id),
                entity_type: "user",
                entity_id: Some(entity_id.as_str()),
                action: "audit_append_only_test",
                reason: Some("verifying enterprise audit ledger hardening"),
                ticket_ref: Some("AUD-1"),
                request_id: Some("req-audit-1"),
                ip_address: Some("127.0.0.1"),
                user_agent: Some("rust-test"),
                source: "rust-backend-test",
                metadata: Some(serde_json::json!({"scope": "ent-0301"})),
                before_state: Some(serde_json::json!({"status": "before"})),
                after_state: Some(serde_json::json!({"status": "after"})),
            },
        )
        .await?;

        let before_after = sqlx::query(
            "SELECT before_state, after_state
             FROM audit_events
             WHERE id = $1",
        )
        .bind(event_id)
        .fetch_one(&pool)
        .await?;
        assert_eq!(
            before_after.get::<serde_json::Value, _>("before_state"),
            serde_json::json!({"status": "before"})
        );
        assert_eq!(
            before_after.get::<serde_json::Value, _>("after_state"),
            serde_json::json!({"status": "after"})
        );

        let update_result =
            sqlx::query("UPDATE audit_events SET action = 'tampered' WHERE id = $1")
                .bind(event_id)
                .execute(&pool)
                .await;
        assert!(update_result.is_err());

        let delete_result = sqlx::query("DELETE FROM audit_events WHERE id = $1")
            .bind(event_id)
            .execute(&pool)
            .await;
        assert!(delete_result.is_err());

        Ok(())
    }

    #[test]
    fn dns_txt_parser_matches_split_google_txt_answers() {
        let answers = vec![DnsGoogleAnswer {
            data: "\"stloads-domain-abc\"".into(),
        }];
        assert!(dns_answers_contain_token(
            Some(&answers),
            "stloads-domain-abc"
        ));

        let split_answers = vec![DnsGoogleAnswer {
            data: "\"stloads-domain\"\"-split\"".into(),
        }];
        assert!(dns_answers_contain_token(
            Some(&split_answers),
            "stloads-domain-split"
        ));
        assert!(!dns_answers_contain_token(Some(&split_answers), "missing"));
    }
}
