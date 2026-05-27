use std::time::Duration as StdDuration;

use crate::{
    auth_session, document_validation::validate_uploaded_document, partner_auth,
    rate_limit::RateLimitPolicy, realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState,
};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{NaiveDate, NaiveDateTime};
use db::{
    dispatch::{
        CreateLoadLegParams, CreateLoadParams, UpsertLoadDocumentParams,
        append_dispatch_desk_follow_up, book_load_leg, clone_load_as_draft,
        create_load_document as insert_load_document, create_load_with_legs,
        find_active_customer_contract_lane, find_load_by_id, find_load_document_by_id,
        find_load_document_scope, find_load_id_and_status_for_leg, find_load_leg_by_id,
        find_load_leg_scope, list_active_freight_document_templates,
        list_load_builder_legs_for_load, list_load_documents_for_load, list_load_history_for_load,
        list_load_legs_for_load, list_load_profile_legs_for_load, load_freight_document_context,
        record_generated_freight_document, resolve_dispatch_exception,
        update_load_document as persist_load_document_updates, update_load_lifecycle,
        update_load_with_legs,
        verify_load_document_blockchain as persist_load_document_blockchain_verification,
    },
    master_data::{
        ensure_city_by_name, ensure_country_by_name, list_commodity_types, list_equipments,
        list_load_types, list_locations, upsert_location,
    },
    tms::find_latest_handoff_for_load,
};
use domain::{
    auth::UserRole,
    dispatch::{
        LegacyLoadLegStatusDescriptor, LoadModuleContract, legacy_load_leg_status_descriptors,
        load_module_contract,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use shared::{
    ApiPostLoadRequest, ApiPostLoadResponse, ApiResponse, BookLoadLegRequest, BookLoadLegResponse,
    BulkLoadImportCommitRequest, BulkLoadImportPreviewRequest, BulkLoadImportResponse,
    BulkLoadImportRowResult, CarrierMatchRow, CarrierMatchScreen, CreateLoadLegRequest,
    CreateLoadRequest, CreateLoadResponse, DispatchDeskFollowUpRequest,
    DispatchDeskFollowUpResponse, DispatchDeskScreen, FacilityAppointmentRequest,
    FacilityAppointmentResponse, GenerateFreightDocumentsRequest, GenerateFreightDocumentsResponse,
    GeneratedFreightDocumentItem, LoadBoardScreen, LoadBuilderDraft, LoadBuilderLegDraft,
    LoadBuilderOption, LoadBuilderScreen, LoadDocumentRow, LoadHandoffSummary, LoadHistoryRow,
    LoadLifecycleAction, LoadLifecycleActionRequest, LoadLifecycleActionResponse, LoadProfileField,
    LoadProfileLegRow, LoadProfileScreen, RateAccessorialLine, RateCalculationRequest,
    RateCalculationResponse, RealtimeEvent, RealtimeEventKind, RealtimeTopic,
    RequiredDocumentChecklistItem, ResolveDispatchExceptionRequest,
    ResolveDispatchExceptionResponse, UpsertCarrierNetworkRequest, UpsertCarrierNetworkResponse,
    UpsertLoadDocumentRequest, UpsertLoadDocumentResponse, VerifyLoadDocumentRequest,
    VerifyLoadDocumentResponse,
};
use std::collections::HashMap;
use tracing::{info, warn};

fn dispatch_document_policy(name: &'static str) -> RateLimitPolicy {
    RateLimitPolicy::new(name, 60, StdDuration::from_secs(60 * 60))
}

fn rate_limit_message(flow: &str, retry_after_seconds: u64) -> String {
    format!(
        "Too many {} attempts. Wait about {} seconds before trying again.",
        flow, retry_after_seconds
    )
}
#[derive(Debug, Serialize)]
struct DispatchOverview {
    contract: LoadModuleContract,
    legacy_status_count: usize,
    document_kinds: usize,
    screen_routes: Vec<&'static str>,
}
#[derive(Debug, Deserialize)]
struct LoadBoardQuery {
    tab: Option<String>,
    origin: Option<String>,
    destination: Option<String>,
    radius_miles: Option<u32>,
    pickup_date: Option<String>,
    delivery_date: Option<String>,
    equipment_id: Option<u64>,
    commodity_type_id: Option<u64>,
    min_rate: Option<f64>,
    max_rate: Option<f64>,
    customer: Option<String>,
    status: Option<String>,
    compliance: Option<String>,
    visibility: Option<String>,
    page: Option<u64>,
    per_page: Option<u64>,
}

fn log_dispatch_failure(
    action: &str,
    user_id: Option<i64>,
    load_id: Option<i64>,
    leg_id: Option<i64>,
    reason: &str,
) {
    warn!(
        action,
        user_id, load_id, leg_id, reason, "dispatch action failed"
    );
}

fn log_dispatch_success(
    action: &str,
    user_id: Option<i64>,
    load_id: Option<i64>,
    leg_id: Option<i64>,
) {
    info!(
        action,
        user_id, load_id, leg_id, "dispatch action succeeded"
    );
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/legacy-statuses", get(legacy_statuses))
        .route("/load-board", get(load_board))
        .route("/carrier-network", get(carrier_network_screen))
        .route("/carrier-network", post(upsert_carrier_network))
        .route("/load-board/{leg_id}/carrier-matches", get(carrier_matches))
        .route("/desk/{desk_key}", get(dispatch_desk))
        .route(
            "/desk/legs/{leg_id}/follow-up",
            post(add_dispatch_desk_follow_up),
        )
        .route(
            "/desk/legs/{leg_id}/exceptions/resolve",
            post(resolve_dispatch_exception_handler),
        )
        .route("/load-builder", get(load_builder))
        .route("/loads", post(create_load))
        .route("/loads/import/preview", post(preview_bulk_load_import))
        .route("/loads/import/commit", post(commit_bulk_load_import))
        .route("/loads/api-post", post(api_post_load))
        .route("/loads/{load_id}/builder", get(edit_load_builder))
        .route("/loads/{load_id}/update", post(update_load))
        .route("/loads/{load_id}/lifecycle", post(load_lifecycle_action))
        .route(
            "/loads/{load_id}/rating/calculate",
            post(calculate_load_rate),
        )
        .route(
            "/loads/{load_id}/appointments",
            post(schedule_facility_appointment),
        )
        .route("/loads/{load_id}", get(load_profile))
        .route(
            "/loads/{load_id}/documents",
            post(create_load_document_handler),
        )
        .route(
            "/loads/{load_id}/documents/upload",
            post(upload_load_document_handler).layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/loads/{load_id}/documents/generate-standard",
            post(generate_standard_freight_documents_handler),
        )
        .route(
            "/documents/{document_id}",
            post(update_load_document_handler),
        )
        .route(
            "/documents/{document_id}/file",
            get(download_load_document_file),
        )
        .route(
            "/documents/{document_id}/verify-blockchain",
            post(verify_load_document_handler),
        )
        .route("/load-board/{leg_id}/book", post(book_leg))
}
async fn index() -> Json<ApiResponse<DispatchOverview>> {
    let contract = load_module_contract();
    Json(ApiResponse::ok(DispatchOverview {
        document_kinds: contract.document_kinds.len(),
        legacy_status_count: legacy_load_leg_status_descriptors().len(),
        screen_routes: vec![
            "/dispatch/load-board",
            "/dispatch/load-builder",
            "/dispatch/loads/{load_id}",
        ],
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("dispatch route group ready"))
}

async fn contract() -> Json<ApiResponse<LoadModuleContract>> {
    Json(ApiResponse::ok(load_module_contract()))
}

async fn legacy_statuses() -> Json<ApiResponse<Vec<LegacyLoadLegStatusDescriptor>>> {
    Json(ApiResponse::ok(
        legacy_load_leg_status_descriptors().to_vec(),
    ))
}

async fn load_board(
    State(state): State<AppState>,
    Query(query): Query<LoadBoardQuery>,
    headers: HeaderMap,
) -> Json<ApiResponse<LoadBoardScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        screen_data::load_board_screen(&state, viewer.as_ref(), query.tab.clone(), query.into())
            .await,
    ))
}

async fn carrier_network_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<shared::CarrierNetworkScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        build_carrier_network_screen(&state, viewer.as_ref()).await,
    ))
}

async fn upsert_carrier_network(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertCarrierNetworkRequest>,
) -> Json<ApiResponse<UpsertCarrierNetworkResponse>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();
    let Some(viewer) = viewer else {
        return Json(ApiResponse::ok(UpsertCarrierNetworkResponse {
            success: false,
            message: "Sign in before managing carrier networks.".into(),
            screen: build_carrier_network_screen(&state, None).await,
        }));
    };
    if !can_manage_carrier_network(&viewer) {
        return Json(ApiResponse::ok(UpsertCarrierNetworkResponse {
            success: false,
            message: "Only shippers, brokers, freight forwarders, and admins can manage private carrier networks.".into(),
            screen: build_carrier_network_screen(&state, Some(&viewer)).await,
        }));
    }
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpsertCarrierNetworkResponse {
            success: false,
            message: format!(
                "Carrier network updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            screen: build_carrier_network_screen(&state, Some(&viewer)).await,
        }));
    };
    let status = payload.relationship_status.trim().to_ascii_lowercase();
    if !matches!(
        status.as_str(),
        "approved" | "preferred" | "backup" | "blocked"
    ) {
        return Json(ApiResponse::ok(UpsertCarrierNetworkResponse {
            success: false,
            message: "Relationship status must be approved, preferred, backup, or blocked.".into(),
            screen: build_carrier_network_screen(&state, Some(&viewer)).await,
        }));
    }
    let carrier_user_id = payload.carrier_user_id as i64;
    let carrier_is_valid = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM users
            WHERE id = $1 AND role_id = $2
        )",
    )
    .bind(carrier_user_id)
    .bind(UserRole::Carrier.legacy_id())
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    if !carrier_is_valid || carrier_user_id == viewer.user.id {
        return Json(ApiResponse::ok(UpsertCarrierNetworkResponse {
            success: false,
            message: "Choose a valid carrier account before saving this network relationship."
                .into(),
            screen: build_carrier_network_screen(&state, Some(&viewer)).await,
        }));
    }

    let group_key = payload
        .carrier_group_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase().replace([' ', '-'], "_"));
    let notes = payload
        .notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let effective_to = payload
        .effective_to
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok());

    let result = sqlx::query(
        "INSERT INTO carrier_network_memberships (
            organization_id, owner_user_id, carrier_user_id, relationship_status,
            carrier_group_key, notes, effective_to, created_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (owner_user_id, carrier_user_id)
         DO UPDATE SET
            organization_id = EXCLUDED.organization_id,
            relationship_status = EXCLUDED.relationship_status,
            carrier_group_key = EXCLUDED.carrier_group_key,
            notes = EXCLUDED.notes,
            effective_to = EXCLUDED.effective_to,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(viewer.user.organization_id)
    .bind(viewer.user.id)
    .bind(carrier_user_id)
    .bind(&status)
    .bind(group_key)
    .bind(notes)
    .bind(effective_to)
    .execute(pool)
    .await;

    let success = result.is_ok();
    Json(ApiResponse::ok(UpsertCarrierNetworkResponse {
        success,
        message: match result {
            Ok(_) => format!("Saved carrier network relationship as {}.", status),
            Err(error) => format!("Carrier network save failed: {}", error),
        },
        screen: build_carrier_network_screen(&state, Some(&viewer)).await,
    }))
}

async fn carrier_matches(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<CarrierMatchScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();
    Json(ApiResponse::ok(
        build_carrier_match_screen(&state, viewer.as_ref(), leg_id).await,
    ))
}

impl From<LoadBoardQuery> for shared::LoadBoardFilters {
    fn from(query: LoadBoardQuery) -> Self {
        Self {
            origin: query.origin,
            destination: query.destination,
            radius_miles: query.radius_miles,
            pickup_date: query.pickup_date,
            delivery_date: query.delivery_date,
            equipment_id: query.equipment_id,
            commodity_type_id: query.commodity_type_id,
            min_rate: query.min_rate,
            max_rate: query.max_rate,
            customer: query.customer,
            status: query.status,
            compliance: query.compliance,
            visibility: query.visibility,
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(20),
        }
    }
}

async fn load_builder(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LoadBuilderScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        build_load_builder_screen(&state, viewer.as_ref(), None).await,
    ))
}

async fn dispatch_desk(
    State(state): State<AppState>,
    Path(desk_key): Path<String>,
    headers: HeaderMap,
) -> Json<ApiResponse<DispatchDeskScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        screen_data::dispatch_desk_screen(&state, viewer.as_ref(), Some(desk_key)).await,
    ))
}

async fn add_dispatch_desk_follow_up(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<DispatchDeskFollowUpRequest>,
) -> Json<ApiResponse<DispatchDeskFollowUpResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "Sign in before adding a Rust dispatch desk follow-up.".into(),
        }));
    };

    if !can_access_dispatch_desk_actions(&session) {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message:
                "The authenticated session does not have dispatch desk follow-up access in the Rust slice."
                    .into(),
        }));
    }

    let note = payload.note.trim();
    if note.is_empty() {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "Enter a follow-up note before saving it to the Rust dispatch desk.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: format!(
                "Dispatch follow-up is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(scope) = find_load_id_and_status_for_leg(pool, leg_id)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "The selected leg was not found for this dispatch desk note.".into(),
        }));
    };

    let allowed =
        session.user.primary_role() == Some(UserRole::Admin)
            || session.session.permissions.iter().any(|permission| {
                permission == "manage_dispatch_desk" || permission == "manage_loads"
            });
    if !allowed {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: scope.load_id,
            message: "This session cannot add follow-up notes to the selected dispatch desk leg."
                .into(),
        }));
    }

    match append_dispatch_desk_follow_up(
        pool,
        leg_id,
        Some(session.user.id),
        &payload.desk_key,
        note,
        payload
            .visibility
            .as_deref()
            .map(str::trim)
            .filter(|value| *value == "customer_visible")
            .unwrap_or("internal"),
    )
    .await
    {
        Ok(Some(load_row)) => {
            let summary = format!(
                "{} added a {} desk follow-up note on leg {}.",
                session.user.name,
                payload.desk_key.trim(),
                leg_id
            );
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::AdminDashboardUpdated,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: None,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: summary.clone(),
                })
                .for_permission_keys([
                    "manage_dispatch_desk",
                    "manage_loads",
                    "access_admin_portal",
                ])
                .with_topics([RealtimeTopic::LoadBoard.as_key()]),
            );

            Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
                success: true,
                leg_id,
                load_id: load_row.load_id,
                message: summary,
            }))
        }
        Ok(None) => Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "The selected leg was not found for this dispatch desk note.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: format!("Dispatch follow-up save failed: {}", error),
        })),
    }
}

async fn resolve_dispatch_exception_handler(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ResolveDispatchExceptionRequest>,
) -> Json<ApiResponse<ResolveDispatchExceptionResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "Sign in before resolving dispatch exceptions.".into(),
        }));
    };

    if !can_access_dispatch_desk_actions(&session) {
        return Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "This session cannot resolve dispatch desk exceptions.".into(),
        }));
    }

    let note = payload.resolution_note.trim();
    if note.is_empty() {
        return Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "Enter a resolution note before closing the exception.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: format!(
                "Exception resolution is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    match resolve_dispatch_exception(
        pool,
        leg_id,
        Some(session.user.id),
        &payload.desk_key,
        payload.exception_type.as_deref(),
        note,
    )
    .await
    {
        Ok(Some(load_row)) => {
            let summary = format!(
                "{} resolved {} desk exception on leg {}.",
                session.user.name,
                payload.desk_key.trim(),
                leg_id
            );
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::AdminDashboardUpdated,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: None,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: summary.clone(),
                })
                .for_permission_keys([
                    "manage_dispatch_desk",
                    "manage_loads",
                    "access_admin_portal",
                ])
                .with_topics([RealtimeTopic::LoadBoard.as_key()]),
            );

            Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
                success: true,
                leg_id,
                load_id: load_row.load_id,
                message: summary,
            }))
        }
        Ok(None) => Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "The selected leg was not found for exception resolution.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ResolveDispatchExceptionResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: format!("Exception resolution failed: {}", error),
        })),
    }
}

async fn edit_load_builder(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<LoadBuilderScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        build_load_builder_screen(&state, viewer.as_ref(), Some(load_id)).await,
    ))
}

async fn load_profile(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<LoadProfileScreen>> {
    let viewer = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten();

    Json(ApiResponse::ok(
        build_load_profile_screen(&state, viewer.as_ref(), load_id).await,
    ))
}

async fn create_load(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateLoadRequest>,
) -> Json<ApiResponse<CreateLoadResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure("create_load", None, None, None, "unauthenticated request");
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Sign in before creating a load from the Rust builder.".into(),
        }));
    };

    if !can_manage_loads(&session) {
        log_dispatch_failure(
            "create_load",
            Some(session.user.id),
            None,
            None,
            "user lacks load creation access",
        );
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message:
                "The authenticated session does not have load creation access in the Rust slice."
                    .into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "create_load",
            Some(session.user.id),
            None,
            None,
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: format!(
                "Load creation is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let title = payload.title.trim().to_string();
    if title.is_empty() {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Enter a title before creating a load.".into(),
        }));
    }

    if payload.weight <= 0.0 {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Weight must be greater than zero.".into(),
        }));
    }

    if !matches!(payload.weight_unit.as_str(), "LBS" | "KG" | "MTON") {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Weight unit must be LBS, KG, or MTON.".into(),
        }));
    }

    if payload.legs.is_empty() {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Add at least one leg before saving the load.".into(),
        }));
    }

    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let mut leg_params = Vec::new();
    for (index, leg) in payload.legs.iter().enumerate() {
        let pickup_address = leg
            .pickup_location_address
            .as_deref()
            .map(str::trim)
            .unwrap_or_default();
        let delivery_address = leg
            .delivery_location_address
            .as_deref()
            .map(str::trim)
            .unwrap_or_default();

        let same_selected_location =
            leg.pickup_location_id.is_some() && leg.pickup_location_id == leg.delivery_location_id;
        let same_autocomplete_address = !pickup_address.is_empty()
            && !delivery_address.is_empty()
            && pickup_address.eq_ignore_ascii_case(delivery_address);

        if same_selected_location || same_autocomplete_address {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!(
                    "Leg {} must use different pickup and delivery locations.",
                    index + 1
                ),
            }));
        }

        if !matches!(leg.bid_status.as_str(), "Fixed" | "Open") {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!("Leg {} must use Fixed or Open bid status.", index + 1),
            }));
        }

        if leg.price < 0.0 {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!("Leg {} price must be zero or greater.", index + 1),
            }));
        }

        let pickup_date = match parse_date_for_storage(&leg.pickup_date) {
            Ok(value) => value,
            Err(message) => {
                return Json(ApiResponse::ok(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} pickup date error: {}", index + 1, message),
                }));
            }
        };

        let delivery_date = match parse_date_for_storage(&leg.delivery_date) {
            Ok(value) => value,
            Err(message) => {
                return Json(ApiResponse::ok(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} delivery date error: {}", index + 1, message),
                }));
            }
        };

        if delivery_date < pickup_date {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!(
                    "Leg {} delivery date must be on or after the pickup date.",
                    index + 1
                ),
            }));
        }

        let pickup_location_id = match resolve_leg_location_reference(
            pool,
            "pickup",
            leg.pickup_location_id,
            leg.pickup_location_address.as_deref(),
            leg.pickup_city.as_deref(),
            leg.pickup_country.as_deref(),
            leg.pickup_place_id.as_deref(),
            leg.pickup_latitude,
            leg.pickup_longitude,
            organization_id,
            "pickup",
        )
        .await
        {
            Ok(value) => value,
            Err(message) => {
                return Json(ApiResponse::ok(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} pickup error: {}", index + 1, message),
                }));
            }
        };

        let delivery_location_id = match resolve_leg_location_reference(
            pool,
            "delivery",
            leg.delivery_location_id,
            leg.delivery_location_address.as_deref(),
            leg.delivery_city.as_deref(),
            leg.delivery_country.as_deref(),
            leg.delivery_place_id.as_deref(),
            leg.delivery_latitude,
            leg.delivery_longitude,
            organization_id,
            "delivery",
        )
        .await
        {
            Ok(value) => value,
            Err(message) => {
                return Json(ApiResponse::ok(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} delivery error: {}", index + 1, message),
                }));
            }
        };

        leg_params.push(CreateLoadLegParams {
            pickup_location_id,
            delivery_location_id,
            pickup_date,
            delivery_date,
            bid_status: leg.bid_status.clone(),
            price: leg.price,
        });
    }

    let visibility = match normalize_visibility(payload.visibility.as_ref()) {
        Ok(value) => value,
        Err(message) => {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: 0,
                message,
            }));
        }
    };
    let appointment_window_start = match parse_optional_datetime_for_storage(
        "Appointment window start",
        payload.appointment_window_start.as_ref(),
    ) {
        Ok(value) => value,
        Err(message) => {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: 0,
                message,
            }));
        }
    };
    let appointment_window_end = match parse_optional_datetime_for_storage(
        "Appointment window end",
        payload.appointment_window_end.as_ref(),
    ) {
        Ok(value) => value,
        Err(message) => {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: 0,
                message,
            }));
        }
    };
    if let (Some(start), Some(end)) = (appointment_window_start, appointment_window_end)
        && end < start
    {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Appointment window end must be after the start.".into(),
        }));
    }
    let freight_mode = match validate_mode_specific_payload(&payload) {
        Ok(mode) => mode,
        Err(message) => {
            return Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: 0,
                message,
            }));
        }
    };

    let mut params = CreateLoadParams {
        title,
        owner_user_id: session.user.id,
        load_type_id: payload.load_type_id as i64,
        equipment_id: payload.equipment_id as i64,
        commodity_type_id: payload.commodity_type_id as i64,
        customer_contract_id: payload.customer_contract_id.map(|value| value as i64),
        customer_contract_lane_id: payload.customer_contract_lane_id.map(|value| value as i64),
        contract_rate: None,
        contract_rate_currency: None,
        contract_posting_behavior: None,
        contract_service_rules: None,
        freight_mode,
        visibility,
        service_level: normalize_optional_text(payload.service_level.as_ref()),
        customer_reference: normalize_optional_text(payload.customer_reference.as_ref()),
        po_number: normalize_optional_text(payload.po_number.as_ref()),
        pickup_appointment_ref: normalize_optional_text(payload.pickup_appointment_ref.as_ref()),
        delivery_appointment_ref: normalize_optional_text(
            payload.delivery_appointment_ref.as_ref(),
        ),
        facility_contact_name: normalize_optional_text(payload.facility_contact_name.as_ref()),
        facility_contact_phone: normalize_optional_text(payload.facility_contact_phone.as_ref()),
        facility_contact_email: normalize_optional_text(payload.facility_contact_email.as_ref()),
        appointment_window_start,
        appointment_window_end,
        accessorial_flags: clone_non_null_json(&payload.accessorial_flags),
        weight_unit: payload.weight_unit.clone(),
        weight: payload.weight,
        temperature_data: clone_non_null_json(&payload.temperature_data),
        container_data: clone_non_null_json(&payload.container_data),
        securement_data: clone_non_null_json(&payload.securement_data),
        special_instructions: payload
            .special_instructions
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        is_hazardous: payload.is_hazardous,
        is_temperature_controlled: payload.is_temperature_controlled,
    };

    if let Err(message) = apply_customer_contract_lane(pool, organization_id, &mut params).await {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message,
        }));
    }

    match create_load_with_legs(pool, &params, &leg_params, Some(session.user.id)).await {
        Ok(created) => {
            let _ = persist_load_localization_snapshot(
                pool,
                organization_id,
                created.load_id,
                payload.weight,
                &payload.weight_unit,
                params.contract_rate_currency.as_deref(),
                Some(session.user.id),
            )
            .await;
            let _ = record_mode_validation_event(
                pool,
                created.load_id,
                &params.freight_mode,
                "validated",
                &[],
                Some(session.user.id),
            )
            .await;
            log_dispatch_success(
                "create_load",
                Some(session.user.id),
                Some(created.load_id),
                None,
            );
            Json(ApiResponse::ok(CreateLoadResponse {
                success: true,
                load_id: Some(created.load_id),
                load_number: Some(created.load_number.clone()),
                leg_count: created.leg_count,
                message: format!(
                    "{} created load {} with {} leg(s) from the Rust builder. Continue in the Rust load profile for document and follow-up workflow.",
                    session.user.name, created.load_number, created.leg_count
                ),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "create_load",
                Some(session.user.id),
                None,
                None,
                &format!("load creation failed: {error}"),
            );
            Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: 0,
                message: format!("Load creation failed: {}", error),
            }))
        }
    }
}

async fn preview_bulk_load_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<BulkLoadImportPreviewRequest>,
) -> Json<ApiResponse<BulkLoadImportResponse>> {
    bulk_load_import_response(state, headers, payload.csv, payload.filename, None, false).await
}

async fn commit_bulk_load_import(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<BulkLoadImportCommitRequest>,
) -> Json<ApiResponse<BulkLoadImportResponse>> {
    bulk_load_import_response(
        state,
        headers,
        payload.csv,
        payload.filename,
        payload.idempotency_key,
        true,
    )
    .await
}

async fn bulk_load_import_response(
    state: AppState,
    headers: HeaderMap,
    csv: String,
    filename: Option<String>,
    idempotency_key: Option<String>,
    commit: bool,
) -> Json<ApiResponse<BulkLoadImportResponse>> {
    let action = if commit {
        "commit_bulk_load_import"
    } else {
        "preview_bulk_load_import"
    };
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure(action, None, None, None, "unauthenticated request");
        return Json(ApiResponse::ok(BulkLoadImportResponse {
            success: false,
            batch_id: None,
            total_rows: 0,
            valid_rows: 0,
            invalid_rows: 0,
            created_load_count: 0,
            error_csv: None,
            rows: vec![],
            message: "Sign in before importing load rows.".into(),
        }));
    };

    if !can_manage_loads(&session) {
        log_dispatch_failure(
            action,
            Some(session.user.id),
            None,
            None,
            "user lacks load import access",
        );
        return Json(ApiResponse::ok(BulkLoadImportResponse {
            success: false,
            batch_id: None,
            total_rows: 0,
            valid_rows: 0,
            invalid_rows: 0,
            created_load_count: 0,
            error_csv: None,
            rows: vec![],
            message: "The authenticated session cannot bulk import loads.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(BulkLoadImportResponse {
            success: false,
            batch_id: None,
            total_rows: 0,
            valid_rows: 0,
            invalid_rows: 0,
            created_load_count: 0,
            error_csv: None,
            rows: vec![],
            message: format!(
                "Bulk import is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let parsed_rows = match parse_bulk_load_csv(&csv) {
        Ok(rows) => rows,
        Err(message) => {
            return Json(ApiResponse::ok(BulkLoadImportResponse {
                success: false,
                batch_id: None,
                total_rows: 0,
                valid_rows: 0,
                invalid_rows: 0,
                created_load_count: 0,
                error_csv: Some("row_number,error\n1,\"CSV parsing failed\"\n".into()),
                rows: vec![BulkLoadImportRowResult {
                    row_number: 1,
                    valid: false,
                    errors: vec![message.clone()],
                    load_id: None,
                    load_number: None,
                    title: None,
                    pickup_label: None,
                    delivery_label: None,
                    price: None,
                }],
                message,
            }));
        }
    };

    let total_rows = parsed_rows.len() as u64;
    let invalid_rows = parsed_rows
        .iter()
        .filter(|row| row.payload.is_none())
        .count() as u64;
    let valid_rows = total_rows.saturating_sub(invalid_rows);
    let mut results = parsed_rows
        .iter()
        .map(|row| row.to_result())
        .collect::<Vec<_>>();

    let batch_id = match insert_load_import_batch(
        pool,
        organization_id,
        Some(session.user.id),
        if commit { "csv_commit" } else { "csv_preview" },
        idempotency_key.as_deref(),
        filename.as_deref(),
        total_rows,
        valid_rows,
        invalid_rows,
        0,
        if invalid_rows == 0 {
            if commit { "committing" } else { "previewed" }
        } else {
            "previewed"
        },
        None,
    )
    .await
    {
        Ok(Some(batch_id)) => Some(batch_id),
        Ok(None) => {
            return Json(ApiResponse::ok(BulkLoadImportResponse {
                success: false,
                batch_id: None,
                total_rows,
                valid_rows,
                invalid_rows,
                created_load_count: 0,
                error_csv: Some(load_import_error_csv(&results)),
                rows: results,
                message: "This bulk import idempotency key has already been used.".into(),
            }));
        }
        Err(error) => {
            warn!(%error, "failed to create bulk load import batch");
            return Json(ApiResponse::ok(BulkLoadImportResponse {
                success: false,
                batch_id: None,
                total_rows,
                valid_rows,
                invalid_rows,
                created_load_count: 0,
                error_csv: Some(load_import_error_csv(&results)),
                rows: results,
                message: format!("Bulk import ledger save failed: {}", error),
            }));
        }
    };

    if let Some(batch_id) = batch_id {
        for row in &parsed_rows {
            let _ = insert_load_import_row(pool, batch_id, row, None, None).await;
        }
    }

    let mut created_load_count = 0_u64;
    if commit && invalid_rows == 0 {
        for (index, row) in parsed_rows.iter().enumerate() {
            let Some(payload) = row.payload.as_ref() else {
                continue;
            };
            match build_load_mutation_inputs(pool, session.user.id, organization_id, payload).await
            {
                Ok((params, leg_params)) => {
                    match create_load_with_legs(pool, &params, &leg_params, Some(session.user.id))
                        .await
                    {
                        Ok(created) => {
                            created_load_count += 1;
                            results[index].load_id = Some(created.load_id);
                            results[index].load_number = Some(created.load_number.clone());
                            if let Some(batch_id) = batch_id {
                                let _ = mark_load_import_row_created(
                                    pool,
                                    batch_id,
                                    row.row_number,
                                    created.load_id,
                                )
                                .await;
                            }
                        }
                        Err(error) => {
                            results[index].valid = false;
                            results[index]
                                .errors
                                .push(format!("Load creation failed: {}", error));
                        }
                    }
                }
                Err(response) => {
                    results[index].valid = false;
                    results[index].errors.push(response.message);
                }
            }
        }
    }

    let final_invalid_rows = results.iter().filter(|row| !row.errors.is_empty()).count() as u64;
    let error_csv = (final_invalid_rows > 0).then(|| load_import_error_csv(&results));
    if let Some(batch_id) = batch_id {
        let _ = update_load_import_batch_outcome(
            pool,
            batch_id,
            created_load_count,
            final_invalid_rows,
            if commit && final_invalid_rows == 0 {
                "committed"
            } else if commit {
                "failed"
            } else {
                "previewed"
            },
            error_csv.as_deref(),
        )
        .await;
    }

    Json(ApiResponse::ok(BulkLoadImportResponse {
        success: if commit {
            final_invalid_rows == 0
        } else {
            true
        },
        batch_id,
        total_rows,
        valid_rows: total_rows.saturating_sub(final_invalid_rows),
        invalid_rows: final_invalid_rows,
        created_load_count,
        error_csv,
        rows: results,
        message: if commit {
            format!(
                "Bulk import committed {} load(s); {} row(s) need attention.",
                created_load_count, final_invalid_rows
            )
        } else {
            format!(
                "Bulk import preview found {} valid row(s) and {} row(s) needing fixes.",
                valid_rows, invalid_rows
            )
        },
    }))
}

async fn api_post_load(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ApiPostLoadRequest>,
) -> Json<ApiResponse<ApiPostLoadResponse>> {
    let idempotency_key = payload.idempotency_key.trim();
    if idempotency_key.len() < 8 {
        return Json(ApiResponse::ok(ApiPostLoadResponse {
            success: false,
            duplicate: false,
            load_id: None,
            load_number: None,
            message: "API posting requires an idempotency key of at least 8 characters.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ApiPostLoadResponse {
            success: false,
            duplicate: false,
            load_id: None,
            load_number: None,
            message: format!(
                "API posting is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let api_actor = match resolve_api_post_actor(&state, &headers).await {
        Ok(actor) => actor,
        Err(message) => {
            log_dispatch_failure("api_post_load", None, None, None, &message);
            return Json(ApiResponse::ok(ApiPostLoadResponse {
                success: false,
                duplicate: false,
                load_id: None,
                load_number: None,
                message,
            }));
        }
    };

    let organization_id = api_actor.organization_id;
    if let Ok(Some(existing)) =
        find_api_post_by_idempotency(pool, organization_id, idempotency_key).await
    {
        return Json(ApiResponse::ok(ApiPostLoadResponse {
            success: true,
            duplicate: true,
            load_id: Some(existing.0),
            load_number: existing.1,
            message: "Idempotent API post replayed the existing load.".into(),
        }));
    }

    let batch_id = match insert_load_import_batch(
        pool,
        organization_id,
        Some(api_actor.actor_user_id),
        "api_post",
        Some(idempotency_key),
        None,
        1,
        1,
        0,
        0,
        "committing",
        None,
    )
    .await
    {
        Ok(Some(batch_id)) => batch_id,
        Ok(None) => {
            if let Ok(Some(existing)) =
                find_api_post_by_idempotency(pool, organization_id, idempotency_key).await
            {
                return Json(ApiResponse::ok(ApiPostLoadResponse {
                    success: true,
                    duplicate: true,
                    load_id: Some(existing.0),
                    load_number: existing.1,
                    message: "Idempotent API post replayed the existing load.".into(),
                }));
            }
            return Json(ApiResponse::ok(ApiPostLoadResponse {
                success: false,
                duplicate: true,
                load_id: None,
                load_number: None,
                message: "This API idempotency key is already processing.".into(),
            }));
        }
        Err(error) => {
            return Json(ApiResponse::ok(ApiPostLoadResponse {
                success: false,
                duplicate: false,
                load_id: None,
                load_number: None,
                message: format!("API posting ledger save failed: {}", error),
            }));
        }
    };

    match build_load_mutation_inputs(
        pool,
        api_actor.actor_user_id,
        organization_id,
        &payload.load,
    )
    .await
    {
        Ok((params, leg_params)) => {
            match create_load_with_legs(pool, &params, &leg_params, Some(api_actor.actor_user_id))
                .await
            {
                Ok(created) => {
                    let row = CsvLoadImportRow::from_api_payload(&payload.load);
                    let _ = insert_load_import_row(
                        pool,
                        batch_id,
                        &row,
                        Some(created.load_id),
                        Some(idempotency_key),
                    )
                    .await;
                    let _ =
                        update_load_import_batch_outcome(pool, batch_id, 1, 0, "committed", None)
                            .await;
                    Json(ApiResponse::ok(ApiPostLoadResponse {
                        success: true,
                        duplicate: false,
                        load_id: Some(created.load_id),
                        load_number: Some(created.load_number),
                        message: format!(
                            "API posted load created with idempotent tracking via {}.",
                            api_actor.auth_label
                        ),
                    }))
                }
                Err(error) => {
                    let _ = update_load_import_batch_outcome(
                        pool,
                        batch_id,
                        0,
                        1,
                        "failed",
                        Some(&format!(
                            "row_number,error\n1,\"{}\"\n",
                            csv_cell(&error.to_string())
                        )),
                    )
                    .await;
                    Json(ApiResponse::ok(ApiPostLoadResponse {
                        success: false,
                        duplicate: false,
                        load_id: None,
                        load_number: None,
                        message: format!("API load posting failed: {}", error),
                    }))
                }
            }
        }
        Err(response) => {
            let _ = update_load_import_batch_outcome(
                pool,
                batch_id,
                0,
                1,
                "failed",
                Some(&format!(
                    "row_number,error\n1,\"{}\"\n",
                    csv_cell(&response.message)
                )),
            )
            .await;
            Json(ApiResponse::ok(ApiPostLoadResponse {
                success: false,
                duplicate: false,
                load_id: None,
                load_number: None,
                message: response.message,
            }))
        }
    }
}

#[derive(Debug, Clone)]
struct ApiPostActor {
    actor_user_id: i64,
    organization_id: i64,
    auth_label: String,
}

async fn resolve_api_post_actor(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<ApiPostActor, String> {
    if let Some(session) = auth_session::resolve_session_from_headers(state, headers)
        .await
        .map_err(|error| format!("Session lookup failed: {}", error))?
    {
        if !can_manage_loads(&session) {
            return Err("The authenticated session cannot post loads through the API.".into());
        }
        return Ok(ApiPostActor {
            actor_user_id: session.user.id,
            organization_id: auth_session::session_organization_id(&session).unwrap_or(1),
            auth_label: "session auth".into(),
        });
    }

    let partner = partner_auth::resolve_partner_client(
        state,
        headers,
        Method::POST,
        "/dispatch/loads/api-post",
        "loads:write",
    )
    .await
    .map_err(|failure| failure.message)?;

    Ok(ApiPostActor {
        actor_user_id: partner.client.actor_user_id,
        organization_id: partner.client.organization_id,
        auth_label: format!("partner API client {}", partner.client.client_name),
    })
}

async fn update_load(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<CreateLoadRequest>,
) -> Json<ApiResponse<CreateLoadResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure(
            "update_load",
            None,
            Some(load_id),
            None,
            "unauthenticated request",
        );
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            leg_count: 0,
            message: "Sign in before editing a load from the Rust builder.".into(),
        }));
    };

    if !can_manage_loads(&session) {
        log_dispatch_failure(
            "update_load",
            Some(session.user.id),
            Some(load_id),
            None,
            "user lacks load update access",
        );
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            leg_count: 0,
            message:
                "The authenticated session does not have load update access in the Rust slice."
                    .into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "update_load",
            Some(session.user.id),
            Some(load_id),
            None,
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            leg_count: 0,
            message: format!(
                "Load updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(existing_load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            leg_count: 0,
            message: format!(
                "Load #{} was not found in the Rust dispatch store.",
                load_id
            ),
        }));
    };

    if !can_manage_existing_load(&session, existing_load.user_id) {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: existing_load.load_number,
            leg_count: existing_load.leg_count.max(0) as u64,
            message: "The authenticated session cannot edit this load in the current Rust slice."
                .into(),
        }));
    }

    let existing_legs = list_load_legs_for_load(pool, load_id)
        .await
        .unwrap_or_default();
    let has_locked_legs = existing_legs.iter().any(|leg| {
        leg.booked_carrier_id.is_some()
            || leg.status_id >= 4
            || leg.accepted_offer_id.is_some()
            || leg.booked_amount.is_some()
    });
    if has_locked_legs {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: existing_load.load_number,
            leg_count: existing_load.leg_count.max(0) as u64,
            message: "This load already has booked or execution-stage legs, so the Rust builder keeps it locked. Continue from the load profile and execution flow instead.".into(),
        }));
    }

    let organization_id = auth_session::session_organization_id(&session).unwrap_or(1);
    let (params, leg_params) =
        match build_load_mutation_inputs(pool, session.user.id, organization_id, &payload).await {
            Ok(values) => values,
            Err(response) => return Json(ApiResponse::ok(response)),
        };

    match update_load_with_legs(pool, load_id, &params, &leg_params, Some(session.user.id)).await {
        Ok(Some(updated)) => {
            let _ = persist_load_localization_snapshot(
                pool,
                organization_id,
                updated.load_id,
                payload.weight,
                &payload.weight_unit,
                params.contract_rate_currency.as_deref(),
                Some(session.user.id),
            )
            .await;
            let _ = record_mode_validation_event(
                pool,
                updated.load_id,
                &params.freight_mode,
                "validated",
                &[],
                Some(session.user.id),
            )
            .await;
            log_dispatch_success(
                "update_load",
                Some(session.user.id),
                Some(updated.load_id),
                None,
            );
            Json(ApiResponse::ok(CreateLoadResponse {
                success: true,
                load_id: Some(updated.load_id),
                load_number: Some(updated.load_number.clone()),
                leg_count: updated.leg_count,
                message: format!(
                    "{} updated load {} from the Rust builder. Continue in the Rust load profile for documents and follow-up workflow.",
                    session.user.name, updated.load_number
                ),
            }))
        }
        Ok(None) => {
            log_dispatch_failure(
                "update_load",
                Some(session.user.id),
                Some(load_id),
                None,
                "load disappeared during update",
            );
            Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: Some(load_id),
                load_number: existing_load.load_number,
                leg_count: 0,
                message: format!(
                    "Load #{} was not found while applying the Rust builder update.",
                    load_id
                ),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "update_load",
                Some(session.user.id),
                Some(load_id),
                None,
                &format!("load update failed: {error}"),
            );
            Json(ApiResponse::ok(CreateLoadResponse {
                success: false,
                load_id: Some(load_id),
                load_number: existing_load.load_number,
                leg_count: 0,
                message: format!("Load update failed: {}", error),
            }))
        }
    }
}

async fn load_lifecycle_action(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<LoadLifecycleActionRequest>,
) -> Json<ApiResponse<LoadLifecycleActionResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            lifecycle_status: None,
            revision_number: None,
            redirect_path: None,
            message: "Sign in before changing a load lifecycle state.".into(),
        }));
    };

    if !can_manage_loads(&session) {
        return Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            lifecycle_status: None,
            revision_number: None,
            redirect_path: None,
            message: "The authenticated session cannot manage load lifecycle actions.".into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            lifecycle_status: None,
            revision_number: None,
            redirect_path: None,
            message: "Load lifecycle actions are unavailable while the database is disabled."
                .into(),
        }));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            lifecycle_status: None,
            revision_number: None,
            redirect_path: None,
            message: format!("Load #{} was not found.", load_id),
        }));
    };

    if !can_manage_existing_load(&session, load.user_id) {
        return Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: load.load_number,
            lifecycle_status: Some(load.lifecycle_status),
            revision_number: Some(load.revision_number),
            redirect_path: None,
            message: "The authenticated session cannot change this load lifecycle.".into(),
        }));
    }

    let action = payload.action.trim().to_ascii_lowercase();
    let legs = list_load_legs_for_load(pool, load_id)
        .await
        .unwrap_or_default();
    let locked = legs.iter().any(|leg| {
        leg.booked_carrier_id.is_some()
            || leg.status_id >= 4
            || leg.accepted_offer_id.is_some()
            || leg.booked_amount.is_some()
    });
    let reason = normalize_optional_text(payload.reason.as_ref());
    let template_name = normalize_optional_text(payload.template_name.as_ref());

    if action == "clone" {
        return match clone_load_as_draft(pool, load_id, Some(session.user.id)).await {
            Ok(Some(created)) => Json(ApiResponse::ok(LoadLifecycleActionResponse {
                success: true,
                load_id: Some(created.load_id),
                load_number: Some(created.load_number.clone()),
                lifecycle_status: Some("draft".into()),
                revision_number: Some(1),
                redirect_path: Some(format!("/loads/{}/edit", created.load_id)),
                message: format!("Cloned load {} into a private draft.", created.load_number),
            })),
            Ok(None) => Json(ApiResponse::ok(LoadLifecycleActionResponse {
                success: false,
                load_id: Some(load_id),
                load_number: load.load_number,
                lifecycle_status: Some(load.lifecycle_status),
                revision_number: Some(load.revision_number),
                redirect_path: None,
                message: "Load could not be cloned because it no longer exists.".into(),
            })),
            Err(error) => Json(ApiResponse::ok(LoadLifecycleActionResponse {
                success: false,
                load_id: Some(load_id),
                load_number: load.load_number,
                lifecycle_status: Some(load.lifecycle_status),
                revision_number: Some(load.revision_number),
                redirect_path: None,
                message: format!("Load clone failed: {}", error),
            })),
        };
    }

    let target_status = match action.as_str() {
        "publish" if matches!(load.lifecycle_status.as_str(), "draft" | "revised") => "published",
        "revise" if load.lifecycle_status == "published" && !locked => "revised",
        "cancel"
            if matches!(
                load.lifecycle_status.as_str(),
                "draft" | "published" | "revised"
            ) && !locked =>
        {
            "cancelled"
        }
        "archive"
            if matches!(load.lifecycle_status.as_str(), "cancelled" | "published") && !locked =>
        {
            "archived"
        }
        "template" if !template_name.clone().unwrap_or_default().trim().is_empty() => {
            load.lifecycle_status.as_str()
        }
        "revise" | "cancel" | "archive" if locked => {
            return Json(ApiResponse::ok(LoadLifecycleActionResponse {
                success: false,
                load_id: Some(load_id),
                load_number: load.load_number,
                lifecycle_status: Some(load.lifecycle_status),
                revision_number: Some(load.revision_number),
                redirect_path: None,
                message: "Booked or execution-stage loads cannot use this lifecycle action.".into(),
            }));
        }
        _ => {
            return Json(ApiResponse::ok(LoadLifecycleActionResponse {
                success: false,
                load_id: Some(load_id),
                load_number: load.load_number,
                lifecycle_status: Some(load.lifecycle_status),
                revision_number: Some(load.revision_number),
                redirect_path: None,
                message: "This lifecycle action is not allowed for the current load state.".into(),
            }));
        }
    };

    match update_load_lifecycle(
        pool,
        load_id,
        target_status,
        reason.as_deref(),
        template_name.as_deref(),
        Some(session.user.id),
    )
    .await
    {
        Ok(Some(updated)) => Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: true,
            load_id: Some(updated.id),
            load_number: updated.load_number.clone(),
            lifecycle_status: Some(updated.lifecycle_status.clone()),
            revision_number: Some(updated.revision_number),
            redirect_path: Some(format!("/loads/{}", updated.id)),
            message: format!(
                "Load {} lifecycle is now {}.",
                updated
                    .load_number
                    .unwrap_or_else(|| format!("#{}", updated.id)),
                profile_title_case(&updated.lifecycle_status)
            ),
        })),
        Ok(None) => Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            lifecycle_status: None,
            revision_number: None,
            redirect_path: None,
            message: "Load lifecycle action could not find the target load.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(LoadLifecycleActionResponse {
            success: false,
            load_id: Some(load_id),
            load_number: load.load_number,
            lifecycle_status: Some(load.lifecycle_status),
            revision_number: Some(load.revision_number),
            redirect_path: None,
            message: format!("Load lifecycle action failed: {}", error),
        })),
    }
}

async fn calculate_load_rate(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<RateCalculationRequest>,
) -> Json<ApiResponse<RateCalculationResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(empty_rate_response(
            load_id,
            "Sign in before calculating load rates.",
        )));
    };

    if !can_manage_loads(&session) && !can_access_admin(&session) {
        return Json(ApiResponse::ok(empty_rate_response(
            load_id,
            "The authenticated session cannot calculate load rates.",
        )));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(empty_rate_response(
            load_id,
            "Rate calculation is unavailable while the database is disabled.",
        )));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return Json(ApiResponse::ok(empty_rate_response(
            load_id,
            "The requested load was not found.",
        )));
    };

    if !can_manage_existing_load(&session, load.user_id) && !can_access_admin(&session) {
        return Json(ApiResponse::ok(empty_rate_response(
            load_id,
            "The authenticated session cannot rate this load.",
        )));
    }

    match calculate_and_persist_rate_quote(pool, &load, &payload, session.user.id).await {
        Ok(response) => Json(ApiResponse::ok(response)),
        Err(error) => Json(ApiResponse::ok(empty_rate_response(
            load_id,
            &format!("Rate calculation failed: {}", error),
        ))),
    }
}

async fn schedule_facility_appointment(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<FacilityAppointmentRequest>,
) -> Json<ApiResponse<FacilityAppointmentResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(empty_facility_appointment_response(
            load_id,
            None,
            "Sign in before scheduling facility appointments.",
        )));
    };

    if !can_manage_loads(&session) && !can_access_dispatch_desk_actions(&session) {
        return Json(ApiResponse::ok(empty_facility_appointment_response(
            load_id,
            Some(payload.leg_id as i64),
            "The authenticated session cannot schedule facility appointments.",
        )));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(empty_facility_appointment_response(
            load_id,
            Some(payload.leg_id as i64),
            "Facility scheduling is unavailable while the database is disabled.",
        )));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return Json(ApiResponse::ok(empty_facility_appointment_response(
            load_id,
            Some(payload.leg_id as i64),
            "The requested load was not found.",
        )));
    };

    if !can_manage_existing_load(&session, load.user_id)
        && !can_access_dispatch_desk_actions(&session)
    {
        return Json(ApiResponse::ok(empty_facility_appointment_response(
            load_id,
            Some(payload.leg_id as i64),
            "The authenticated session cannot schedule this load.",
        )));
    }

    match schedule_or_reschedule_facility_appointment(pool, &load, &payload, session.user.id).await
    {
        Ok(response) => {
            if response.success {
                state.publish_realtime(
                    RoutedRealtimeEvent::new(RealtimeEvent {
                        request_id: None,
                        kind: RealtimeEventKind::TmsOperationsUpdated,
                        leg_id: response.leg_id.map(|value| value.max(0) as u64),
                        conversation_id: None,
                        offer_id: None,
                        message_id: None,
                        actor_user_id: Some(session.user.id.max(0) as u64),
                        subject_user_id: load.user_id.map(|value| value.max(0) as u64),
                        presence_state: None,
                        last_read_message_id: None,
                        summary: response.message.clone(),
                    })
                    .for_permission_keys(["manage_loads", "manage_dispatch_desk"])
                    .with_topics([RealtimeTopic::LoadBoard.as_key()]),
                );
            }
            Json(ApiResponse::ok(response))
        }
        Err(error) => Json(ApiResponse::ok(empty_facility_appointment_response(
            load_id,
            Some(payload.leg_id as i64),
            &format!("Facility appointment save failed: {}", error),
        ))),
    }
}

#[derive(Debug)]
struct ParsedUploadedDocument {
    document_name: String,
    document_type: String,
    original_name: String,
    mime_type: Option<String>,
    bytes: Vec<u8>,
}

async fn upload_load_document_handler(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Json<ApiResponse<UpsertLoadDocumentResponse>> {
    if state.config.kill_switch_document_uploads {
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "Document uploads are temporarily disabled by an operational kill switch."
                .into(),
        }));
    }

    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure(
            "upload_load_document",
            None,
            Some(load_id),
            None,
            "unauthenticated request",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "Sign in before uploading load documents from the Rust profile.".into(),
        }));
    };

    let rate_decision = state
        .check_rate_limit(
            dispatch_document_policy("dispatch_document_upload"),
            format!("{}:{}", session.user.id, load_id),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: rate_limit_message("load document upload", rate_decision.retry_after_seconds),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "upload_load_document",
            None,
            Some(load_id),
            None,
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: format!(
                "Document uploads are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        log_dispatch_failure(
            "upload_load_document",
            Some(session.user.id),
            Some(load_id),
            None,
            "load not found",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: format!(
                "Load #{} was not found in the Rust dispatch store.",
                load_id
            ),
        }));
    };

    if !can_manage_load_documents(&session, load.user_id, load.organization_id) {
        log_dispatch_failure(
            "upload_load_document",
            Some(session.user.id),
            Some(load_id),
            None,
            "user cannot manage load documents",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "The authenticated session cannot upload documents for this load.".into(),
        }));
    }

    let upload = match parse_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => {
            log_dispatch_failure(
                "upload_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                &message,
            );
            return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message,
            }));
        }
    };

    let stored_file = match state
        .document_storage
        .save_load_document(load_id, &upload.original_name, &upload.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            log_dispatch_failure(
                "upload_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                &format!("document storage failed: {error}"),
            );
            return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message: format!("Document upload storage failed: {}", error),
            }));
        }
    };

    let params = UpsertLoadDocumentParams {
        document_name: upload.document_name,
        document_type: upload.document_type,
        file_path: stored_file.file_path,
        storage_provider: stored_file.storage_provider,
        original_name: Some(upload.original_name),
        mime_type: upload.mime_type,
        file_size: Some(upload.bytes.len() as i64),
    };

    match insert_load_document(pool, load_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => {
            log_dispatch_success(
                "upload_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: true,
                load_id,
                document_id: Some(document.id),
                message: format!(
                    "{} uploaded document {} to load {}. The binary file is now viewable by admin and the uploader profile in this Rust slice.",
                    session.user.name,
                    document.document_name,
                    load.load_number
                        .clone()
                        .unwrap_or_else(|| format!("#{}", load.id))
                ),
            }))
        }
        Ok(None) => {
            log_dispatch_failure(
                "upload_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                "load disappeared during uploaded document save",
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message: "The target load could not be found while saving the uploaded document."
                    .into(),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "upload_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                &format!("document create failed after upload: {error}"),
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message: format!("Document upload create failed: {}", error),
            }))
        }
    }
}

async fn download_load_document_file(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before viewing load document files from the Rust profile.",
        );
    };

    let rate_decision = state
        .check_rate_limit(
            dispatch_document_policy("dispatch_document_read"),
            format!("{}:{}", session.user.id, document_id),
        )
        .await;
    if !rate_decision.allowed {
        return text_response(
            StatusCode::TOO_MANY_REQUESTS,
            &rate_limit_message("load document read", rate_decision.retry_after_seconds),
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Document file access is unavailable because the database is disabled.",
        );
    };

    let Some(scope) = find_load_document_scope(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        return text_response(
            StatusCode::NOT_FOUND,
            "The requested document was not found.",
        );
    };

    if !can_view_load_document_file(&session, scope.uploaded_by_user_id) {
        return text_response(
            StatusCode::FORBIDDEN,
            "Only admin users and the profile that uploaded this document can view the file in the current Rust slice.",
        );
    }

    let Some(document) = find_load_document_by_id(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        return text_response(
            StatusCode::NOT_FOUND,
            "The requested document was not found.",
        );
    };

    let bytes = match state
        .document_storage
        .read_document(&document.storage_provider, &document.file_path)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Document file could not be opened: {}", error),
            );
        }
    };

    let mime_type = document
        .mime_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("application/octet-stream");
    let file_name = document
        .original_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| profile_file_label(&document.file_path));
    let content_disposition = format!(
        "inline; filename=\"{}\"",
        sanitize_header_file_name(&file_name)
    );

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    response
}
async fn create_load_document_handler(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpsertLoadDocumentRequest>,
) -> Json<ApiResponse<UpsertLoadDocumentResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure(
            "create_load_document",
            None,
            Some(load_id),
            None,
            "unauthenticated request",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "Sign in before adding load documents from the Rust profile.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "create_load_document",
            None,
            Some(load_id),
            None,
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: format!(
                "Document actions are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        log_dispatch_failure(
            "create_load_document",
            Some(session.user.id),
            Some(load_id),
            None,
            "load not found",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: format!(
                "Load #{} was not found in the Rust dispatch store.",
                load_id
            ),
        }));
    };

    if !can_manage_load_documents(&session, load.user_id, load.organization_id) {
        log_dispatch_failure(
            "create_load_document",
            Some(session.user.id),
            Some(load_id),
            None,
            "user cannot manage load documents",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "The authenticated session cannot add documents for this load.".into(),
        }));
    }

    let params = match validate_load_document_payload(&payload) {
        Ok(value) => value,
        Err(message) => {
            log_dispatch_failure(
                "create_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                &message,
            );
            return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message,
            }));
        }
    };

    match insert_load_document(pool, load_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => {
            log_dispatch_success(
                "create_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: true,
                load_id,
                document_id: Some(document.id),
                message: format!(
                    "{} added document {} to load {}. Binary upload transport will move to IBM object storage next.",
                    session.user.name,
                    document.document_name,
                    load.load_number
                        .clone()
                        .unwrap_or_else(|| format!("#{}", load.id))
                ),
            }))
        }
        Ok(None) => {
            log_dispatch_failure(
                "create_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                "load disappeared during document create",
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message: "The target load could not be found while saving the document.".into(),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "create_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
                &format!("document create failed: {error}"),
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message: format!("Document create failed: {}", error),
            }))
        }
    }
}

async fn generate_standard_freight_documents_handler(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<GenerateFreightDocumentsRequest>,
) -> Json<ApiResponse<GenerateFreightDocumentsResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
            success: false,
            load_id,
            generated: Vec::new(),
            message: "Sign in before generating standard freight documents.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
            success: false,
            load_id,
            generated: Vec::new(),
            message: "Freight document generation is unavailable because the database is disabled."
                .into(),
        }));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
            success: false,
            load_id,
            generated: Vec::new(),
            message: format!("Load #{} was not found.", load_id),
        }));
    };

    if !can_manage_load_documents(&session, load.user_id, load.organization_id) {
        return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
            success: false,
            load_id,
            generated: Vec::new(),
            message: "This session cannot generate documents for the requested load.".into(),
        }));
    }

    let selected = normalize_template_selection(payload.template_keys);
    let templates = list_active_freight_document_templates(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|template| selected.iter().any(|key| key == &template.template_key))
        .collect::<Vec<_>>();
    if templates.is_empty() {
        return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
            success: false,
            load_id,
            generated: Vec::new(),
            message: "No active freight document templates matched the request.".into(),
        }));
    }

    let Some(context) = load_freight_document_context(pool, load_id)
        .await
        .unwrap_or_default()
    else {
        return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
            success: false,
            load_id,
            generated: Vec::new(),
            message: "Load context could not be prepared for document generation.".into(),
        }));
    };

    let mut generated = Vec::new();
    for template in templates {
        let rendered = render_freight_document_template(&template.body_template, &context);
        let original_name = format!(
            "{}-{}-{}.txt",
            context
                .load_number
                .clone()
                .unwrap_or_else(|| format!("load-{}", load_id)),
            template.template_key,
            template.version
        );
        let saved = match state
            .document_storage
            .save_load_document(load_id, &original_name, rendered.as_bytes())
            .await
        {
            Ok(saved) => saved,
            Err(error) => {
                return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
                    success: false,
                    load_id,
                    generated,
                    message: format!("Generated document storage failed: {}", error),
                }));
            }
        };

        let params = UpsertLoadDocumentParams {
            document_name: template.title.clone(),
            document_type: template.document_type_key.clone(),
            file_path: saved.file_path,
            storage_provider: saved.storage_provider,
            original_name: Some(original_name),
            mime_type: Some("text/plain".into()),
            file_size: Some(rendered.len() as i64),
        };
        let Some(document) =
            (match insert_load_document(pool, load_id, &params, Some(session.user.id)).await {
                Ok(document) => document,
                Err(error) => {
                    return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
                        success: false,
                        load_id,
                        generated,
                        message: format!("Generated document persistence failed: {}", error),
                    }));
                }
            })
        else {
            return Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
                success: false,
                load_id,
                generated,
                message: "Generated document could not be linked to the load.".into(),
            }));
        };

        let _ = record_generated_freight_document(
            pool,
            load_id,
            document.id,
            &template.template_key,
            &template.version,
            Some(session.user.id),
        )
        .await;

        generated.push(GeneratedFreightDocumentItem {
            template_key: template.template_key,
            template_version: template.version,
            document_id: document.id,
            document_name: document.document_name,
            document_type: document.document_type,
        });
    }

    Json(ApiResponse::ok(GenerateFreightDocumentsResponse {
        success: !generated.is_empty(),
        load_id,
        message: format!(
            "Generated {} standard freight document(s) and linked them to load history.",
            generated.len()
        ),
        generated,
    }))
}

async fn update_load_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpsertLoadDocumentRequest>,
) -> Json<ApiResponse<UpsertLoadDocumentResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure(
            "update_load_document",
            None,
            None,
            None,
            "unauthenticated request",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id: Some(document_id),
            message: "Sign in before editing load documents from the Rust profile.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "update_load_document",
            None,
            None,
            None,
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id: Some(document_id),
            message: format!(
                "Document actions are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(scope) = find_load_document_scope(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        log_dispatch_failure(
            "update_load_document",
            Some(session.user.id),
            None,
            None,
            "document not found",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id: Some(document_id),
            message: format!(
                "Document #{} was not found in the Rust dispatch store.",
                document_id
            ),
        }));
    };

    if !can_manage_load_documents(&session, scope.load_owner_user_id, scope.organization_id) {
        log_dispatch_failure(
            "update_load_document",
            Some(session.user.id),
            Some(scope.load_id),
            None,
            "user cannot manage load documents",
        );
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id: Some(document_id),
            message: "The authenticated session cannot edit documents for this load.".into(),
        }));
    }

    let params = match validate_load_document_payload(&payload) {
        Ok(value) => value,
        Err(message) => {
            log_dispatch_failure(
                "update_load_document",
                Some(session.user.id),
                Some(scope.load_id),
                None,
                &message,
            );
            return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id: Some(document_id),
                message,
            }));
        }
    };

    match persist_load_document_updates(pool, document_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => {
            log_dispatch_success(
                "update_load_document",
                Some(session.user.id),
                Some(document.load_id),
                None,
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: true,
                load_id: document.load_id,
                document_id: Some(document.id),
                message: format!(
                    "{} updated document {} from the Rust load profile.",
                    session.user.name, document.document_name
                ),
            }))
        }
        Ok(None) => {
            log_dispatch_failure(
                "update_load_document",
                Some(session.user.id),
                Some(scope.load_id),
                None,
                "document disappeared during update",
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id: Some(document_id),
                message: "The requested document disappeared before the update completed.".into(),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "update_load_document",
                Some(session.user.id),
                Some(scope.load_id),
                None,
                &format!("document update failed: {error}"),
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id: Some(document_id),
                message: format!("Document update failed: {}", error),
            }))
        }
    }
}

async fn verify_load_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<VerifyLoadDocumentRequest>,
) -> Json<ApiResponse<VerifyLoadDocumentResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        log_dispatch_failure(
            "verify_load_document",
            None,
            None,
            None,
            "unauthenticated request",
        );
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            hash: None,
            message: "Sign in before verifying a document content hash from the Rust profile."
                .into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "verify_load_document",
            None,
            None,
            None,
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            hash: None,
            message: format!(
                "Content hash verification is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(scope) = find_load_document_scope(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        log_dispatch_failure(
            "verify_load_document",
            Some(session.user.id),
            None,
            None,
            "document not found",
        );
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            hash: None,
            message: format!(
                "Document #{} was not found in the Rust dispatch store.",
                document_id
            ),
        }));
    };

    if !can_manage_load_documents(&session, scope.load_owner_user_id, scope.organization_id) {
        log_dispatch_failure(
            "verify_load_document",
            Some(session.user.id),
            Some(scope.load_id),
            None,
            "user cannot manage load documents",
        );
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            hash: None,
            message:
                "The authenticated session cannot verify content hash state for this load document."
                    .into(),
        }));
    }

    if let Ok(Some(document)) = find_load_document_by_id(pool, document_id).await
        && let Some(existing_hash) = document.hash.clone()
    {
        log_dispatch_success(
            "verify_load_document",
            Some(session.user.id),
            Some(document.load_id),
            None,
        );
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: true,
            load_id: document.load_id,
            document_id: document.id,
            hash: Some(existing_hash),
            message: format!(
                "{} already has a stored SHA-256 content hash in the Rust document ledger.",
                document.document_name
            ),
        }));
    }

    let Some(document) = find_load_document_by_id(pool, document_id)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            hash: None,
            message: "The requested document disappeared before content hash verification.".into(),
        }));
    };
    let bytes = match state
        .document_storage
        .read_document(&document.storage_provider, &document.file_path)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => {
            return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id,
                hash: None,
                message: format!("Document content could not be read for hashing: {}", error),
            }));
        }
    };
    let content_sha256 = sha256_hex(&bytes);

    let note = payload
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match persist_load_document_blockchain_verification(
        pool,
        document_id,
        &content_sha256,
        Some(session.user.id),
        note,
    )
    .await
    {
        Ok(Some(document)) => {
            log_dispatch_success(
                "verify_load_document",
                Some(session.user.id),
                Some(document.load_id),
                None,
            );
            Json(ApiResponse::ok(VerifyLoadDocumentResponse {
                success: true,
                load_id: document.load_id,
                document_id: document.id,
                hash: document.hash,
                message: format!(
                    "{} verified a SHA-256 content hash for document {}.",
                    session.user.name, document.document_name
                ),
            }))
        }
        Ok(None) => {
            log_dispatch_failure(
                "verify_load_document",
                Some(session.user.id),
                Some(scope.load_id),
                None,
                "document disappeared during verification",
            );
            Json(ApiResponse::ok(VerifyLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id,
                hash: None,
                message:
                    "The requested document disappeared before content hash verification completed."
                        .into(),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "verify_load_document",
                Some(session.user.id),
                Some(scope.load_id),
                None,
                &format!("content hash verification failed: {error}"),
            );
            Json(ApiResponse::ok(VerifyLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id,
                hash: None,
                message: format!("Content hash verification failed: {}", error),
            }))
        }
    }
}
async fn book_leg(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<BookLoadLegRequest>,
) -> Json<ApiResponse<BookLoadLegResponse>> {
    if state.config.kill_switch_booking {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Paused".into(),
            message: "Carrier booking is temporarily disabled by an operational kill switch."
                .into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        log_dispatch_failure(
            "book_leg",
            None,
            None,
            Some(leg_id),
            "database connection is disabled",
        );
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Unavailable".into(),
            message: format!(
                "Booking action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Ok(Some(session)) = auth_session::resolve_session_from_headers(&state, &headers).await
    else {
        log_dispatch_failure(
            "book_leg",
            None,
            None,
            Some(leg_id),
            "unauthenticated request",
        );
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Unauthorized".into(),
            message: "Sign in as a carrier before booking a leg from the Rust load board.".into(),
        }));
    };

    if session.user.primary_role() != Some(UserRole::Carrier) {
        log_dispatch_failure(
            "book_leg",
            Some(session.user.id),
            None,
            Some(leg_id),
            "user is not a carrier",
        );
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Forbidden".into(),
            message:
                "Only authenticated carrier accounts can self-book a load leg in this Rust slice."
                    .into(),
        }));
    }

    let Ok(Some(existing_leg)) = find_load_leg_by_id(pool, leg_id).await else {
        log_dispatch_failure(
            "book_leg",
            Some(session.user.id),
            None,
            Some(leg_id),
            "leg not found",
        );
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Missing".into(),
            message: "The requested load leg was not found.".into(),
        }));
    };

    if existing_leg.booked_carrier_id == Some(session.user.id) {
        log_dispatch_success(
            "book_leg",
            Some(session.user.id),
            Some(existing_leg.load_id),
            Some(leg_id),
        );
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: true,
            leg_id,
            status_label: "Booked".into(),
            message: "This load leg is already booked by the authenticated carrier account.".into(),
        }));
    }

    if existing_leg.booked_carrier_id.is_some() || existing_leg.status_id >= 4 {
        log_dispatch_failure(
            "book_leg",
            Some(session.user.id),
            Some(existing_leg.load_id),
            Some(leg_id),
            "leg is no longer open for booking",
        );
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Locked".into(),
            message: "This load leg is no longer open for carrier self-booking.".into(),
        }));
    }

    match carrier_can_book_leg(pool, leg_id, session.user.id).await {
        Ok(true) => {}
        Ok(false) => {
            log_dispatch_failure(
                "book_leg",
                Some(session.user.id),
                Some(existing_leg.load_id),
                Some(leg_id),
                "carrier blocked by private network or visibility rule",
            );
            return Json(ApiResponse::ok(BookLoadLegResponse {
                success: false,
                leg_id,
                status_label: "Blocked".into(),
                message:
                    "This freight is restricted by the shipper private network or blocklist rules."
                        .into(),
            }));
        }
        Err(error) => {
            log_dispatch_failure(
                "book_leg",
                Some(session.user.id),
                Some(existing_leg.load_id),
                Some(leg_id),
                "private network lookup failed",
            );
            return Json(ApiResponse::ok(BookLoadLegResponse {
                success: false,
                leg_id,
                status_label: "Blocked".into(),
                message: format!("Private network check failed: {}", error),
            }));
        }
    }

    match carrier_restricted_packet_blockers(pool, leg_id, session.user.id).await {
        Ok(blockers) if blockers.is_empty() => {}
        Ok(blockers) => {
            log_dispatch_failure(
                "book_leg",
                Some(session.user.id),
                Some(existing_leg.load_id),
                Some(leg_id),
                "carrier packet is incomplete for restricted freight",
            );
            return Json(ApiResponse::ok(BookLoadLegResponse {
                success: false,
                leg_id,
                status_label: "Packet Incomplete".into(),
                message: format!(
                    "Carrier packet blocks restricted freight: {}.",
                    blockers.join("; ")
                ),
            }));
        }
        Err(error) => {
            log_dispatch_failure(
                "book_leg",
                Some(session.user.id),
                Some(existing_leg.load_id),
                Some(leg_id),
                "carrier packet lookup failed",
            );
            return Json(ApiResponse::ok(BookLoadLegResponse {
                success: false,
                leg_id,
                status_label: "Packet Review".into(),
                message: format!("Carrier packet review failed: {}", error),
            }));
        }
    }

    match book_load_leg(
        pool,
        leg_id,
        session.user.id,
        payload.booked_amount,
        Some(session.user.id),
        payload.idempotency_key.as_deref(),
    )
    .await
    {
        Ok(Some(updated_leg)) => {
            log_dispatch_success(
                "book_leg",
                Some(session.user.id),
                Some(updated_leg.load_id),
                Some(leg_id),
            );
            let mut target_user_ids = vec![session.user.id.max(0) as u64];
            if let Ok(Some(scope)) = find_load_leg_scope(pool, leg_id).await {
                if let Some(owner_id) = scope.load_owner_user_id
                    && owner_id > 0
                {
                    target_user_ids.push(owner_id as u64);
                }
                if let Some(booked_carrier_id) = scope.booked_carrier_id
                    && booked_carrier_id > 0
                {
                    target_user_ids.push(booked_carrier_id as u64);
                }
            }
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::LoadLegBooked,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: updated_leg
                        .accepted_offer_id
                        .map(|value| value.max(0) as u64),
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!("{} booked load leg #{}.", session.user.name, leg_id),
                })
                .for_user_ids(target_user_ids)
                .for_role_keys(["carrier"])
                .with_topics([RealtimeTopic::LoadBoard.as_key()]),
            );

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    request_id: None,
                    kind: RealtimeEventKind::PaymentsOperationsUpdated,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: updated_leg
                        .accepted_offer_id
                        .map(|value| value.max(0) as u64),
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(session.user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "Booking changed payment follow-up readiness for load leg #{}.",
                        leg_id
                    ),
                })
                .for_permission_keys(["manage_payments"])
                .with_topics([RealtimeTopic::AdminPayments.as_key()]),
            );

            Json(ApiResponse::ok(BookLoadLegResponse {
                success: true,
                leg_id,
                status_label: "Booked".into(),
                message: "Load leg booked from the authenticated Rust dispatch route; the board will refresh through scoped realtime updates.".into(),
            }))
        }
        Ok(None) => {
            log_dispatch_failure(
                "book_leg",
                Some(session.user.id),
                None,
                Some(leg_id),
                "leg disappeared during booking",
            );
            Json(ApiResponse::ok(BookLoadLegResponse {
                success: false,
                leg_id,
                status_label: "Missing".into(),
                message: "The requested load leg was not found.".into(),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "book_leg",
                Some(session.user.id),
                None,
                Some(leg_id),
                &format!("booking action failed: {error}"),
            );
            Json(ApiResponse::ok(BookLoadLegResponse {
                success: false,
                leg_id,
                status_label: "Error".into(),
                message: format!("Booking action failed: {}", error),
            }))
        }
    }
}

async fn build_load_builder_screen(
    state: &AppState,
    viewer: Option<&crate::auth_session::ResolvedSession>,
    load_id: Option<i64>,
) -> LoadBuilderScreen {
    let Some(viewer) = viewer else {
        return empty_load_builder_screen(
            state,
            load_id,
            vec![
                "Sign in before using the Rust load builder.".into(),
                "This route intentionally avoids Laravel fallback forms during staged cutover."
                    .into(),
            ],
        );
    };

    if !can_manage_loads(viewer) {
        return empty_load_builder_screen(
            state,
            load_id,
            vec![
                "The authenticated session does not have load builder access in the Rust slice."
                    .into(),
            ],
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return empty_load_builder_screen(
            state,
            load_id,
            vec![format!(
                "Load builder data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        );
    };

    let load_types = list_load_types(pool).await.unwrap_or_default();
    let equipments = list_equipments(pool).await.unwrap_or_default();
    let commodity_types = list_commodity_types(pool).await.unwrap_or_default();
    let mut locations = list_locations(pool).await.unwrap_or_default();
    locations.sort_by(|left, right| left.name.cmp(&right.name));

    let mut notes = vec![
        "Google-address autocomplete now drives Rust load creation and editing, with geolocation bias when the browser allows it and broader Google results when GPS is unavailable.".into(),
        "Booked or execution-stage loads stay locked out of builder edit mode so staged cutover cannot accidentally rewrite live operational legs.".into(),
    ];

    let draft = if let Some(load_id) = load_id {
        let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
            return empty_load_builder_screen(
                state,
                Some(load_id),
                vec![format!(
                    "Load #{} was not found in the Rust dispatch store.",
                    load_id
                )],
            );
        };

        if !can_manage_existing_load(viewer, load.user_id) {
            return empty_load_builder_screen(
                state,
                Some(load_id),
                vec![
                    "The authenticated session cannot edit this load in the current Rust slice."
                        .into(),
                ],
            );
        }

        let builder_legs = list_load_builder_legs_for_load(pool, load.id)
            .await
            .unwrap_or_default();
        let has_locked_legs = builder_legs
            .iter()
            .any(|leg| leg.booked_carrier_id.is_some() || leg.status_id >= 4);
        if has_locked_legs {
            return empty_load_builder_screen(
                state,
                Some(load.id),
                vec![
                    "This load already has booked or execution-stage legs, so the Rust builder keeps it locked. Continue from the load profile and execution workflow instead.".into(),
                ],
            );
        }

        notes.push(
            "This builder is preloaded from the saved Rust load so dispatch can reopen a draft-like load, adjust legs, and save without dropping into the legacy PHP form.".into(),
        );

        Some(LoadBuilderDraft {
            load_id: load.id.max(0) as u64,
            load_number: load.load_number.clone(),
            title: load.title,
            load_type_id: load.load_type_id.map(|value| value.max(0) as u64),
            equipment_id: load.equipment_id.map(|value| value.max(0) as u64),
            commodity_type_id: load.commodity_type_id.map(|value| value.max(0) as u64),
            customer_contract_id: load.customer_contract_id.map(|value| value.max(0) as u64),
            customer_contract_lane_id: load
                .customer_contract_lane_id
                .map(|value| value.max(0) as u64),
            freight_mode: Some(load.freight_mode),
            visibility: Some(load.visibility),
            service_level: load.service_level,
            customer_reference: load.customer_reference,
            po_number: load.po_number,
            pickup_appointment_ref: load.pickup_appointment_ref,
            delivery_appointment_ref: load.delivery_appointment_ref,
            facility_contact_name: load.facility_contact_name,
            facility_contact_phone: load.facility_contact_phone,
            facility_contact_email: load.facility_contact_email,
            appointment_window_start: format_builder_datetime(
                load.appointment_window_start.as_ref(),
            ),
            appointment_window_end: format_builder_datetime(load.appointment_window_end.as_ref()),
            accessorial_flags: load.accessorial_flags,
            weight_unit: load.weight_unit,
            weight: load.weight,
            temperature_data: load.temperature_data,
            container_data: load.container_data,
            securement_data: load.securement_data,
            special_instructions: load.special_instructions,
            is_hazardous: load.is_hazardous,
            is_temperature_controlled: load.is_temperature_controlled,
            legs: builder_legs
                .into_iter()
                .map(|leg| LoadBuilderLegDraft {
                    pickup_location_address: leg.pickup_location_name.unwrap_or_else(|| "".into()),
                    pickup_city: leg.pickup_city_name,
                    pickup_country: leg.pickup_country_name,
                    pickup_place_id: None,
                    pickup_latitude: None,
                    pickup_longitude: None,
                    delivery_location_address: leg
                        .delivery_location_name
                        .unwrap_or_else(|| "".into()),
                    delivery_city: leg.delivery_city_name,
                    delivery_country: leg.delivery_country_name,
                    delivery_place_id: None,
                    delivery_latitude: None,
                    delivery_longitude: None,
                    pickup_date: format_builder_date(leg.pickup_date.as_ref()),
                    delivery_date: format_builder_date(leg.delivery_date.as_ref()),
                    bid_status: leg.bid_status.unwrap_or_else(|| "Fixed".into()),
                    price: leg.price,
                })
                .collect(),
        })
    } else {
        notes.push(
            "This Rust builder ports core load creation with multi-leg posting and direct handoff into the Rust load profile workflow.".into(),
        );
        None
    };

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so builder requests remain proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    let is_edit_mode = draft.is_some();

    LoadBuilderScreen {
        title: if is_edit_mode {
            "Edit Load".into()
        } else {
            "Create Load".into()
        },
        subtitle: if is_edit_mode {
            "Rust builder for revising load details and multi-leg routes before dispatch execution begins.".into()
        } else {
            "First-pass Rust builder for core load details and multi-leg creation.".into()
        },
        mode: if is_edit_mode {
            "edit".into()
        } else {
            "create".into()
        },
        submit_label: if is_edit_mode {
            "Save load changes".into()
        } else {
            "Create load".into()
        },
        load_id: draft.as_ref().map(|value| value.load_id),
        draft,
        load_type_options: load_types
            .into_iter()
            .map(|row| LoadBuilderOption {
                id: row.id.max(0) as u64,
                label: row.name,
            })
            .collect(),
        equipment_options: equipments
            .into_iter()
            .map(|row| LoadBuilderOption {
                id: row.id.max(0) as u64,
                label: row.name,
            })
            .collect(),
        commodity_type_options: commodity_types
            .into_iter()
            .map(|row| LoadBuilderOption {
                id: row.id.max(0) as u64,
                label: row.name,
            })
            .collect(),
        location_options: locations
            .into_iter()
            .map(|row| LoadBuilderOption {
                id: row.id.max(0) as u64,
                label: row.name,
            })
            .collect(),
        weight_units: vec!["LBS".into(), "KG".into(), "MTON".into()],
        bid_status_options: vec!["Fixed".into(), "Open".into()],
        notes,
    }
}

fn empty_load_builder_screen(
    state: &AppState,
    load_id: Option<i64>,
    mut notes: Vec<String>,
) -> LoadBuilderScreen {
    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so builder requests remain proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    LoadBuilderScreen {
        title: if load_id.is_some() {
            "Edit Load".into()
        } else {
            "Create Load".into()
        },
        subtitle: "Secure Rust builder".into(),
        mode: if load_id.is_some() {
            "edit".into()
        } else {
            "create".into()
        },
        submit_label: if load_id.is_some() {
            "Save load changes".into()
        } else {
            "Create load".into()
        },
        load_id: load_id.map(|value| value.max(0) as u64),
        draft: None,
        load_type_options: Vec::new(),
        equipment_options: Vec::new(),
        commodity_type_options: Vec::new(),
        location_options: Vec::new(),
        weight_units: vec!["LBS".into(), "KG".into(), "MTON".into()],
        bid_status_options: vec!["Fixed".into(), "Open".into()],
        notes,
    }
}

fn can_manage_loads(viewer: &crate::auth_session::ResolvedSession) -> bool {
    viewer
        .session
        .permissions
        .iter()
        .any(|permission| permission == "manage_loads")
}

fn can_access_admin(viewer: &crate::auth_session::ResolvedSession) -> bool {
    viewer.user.primary_role() == Some(UserRole::Admin)
        || viewer
            .session
            .permissions
            .iter()
            .any(|permission| permission == "access_admin_portal")
}

fn can_access_dispatch_desk_actions(viewer: &crate::auth_session::ResolvedSession) -> bool {
    if viewer.user.primary_role() == Some(UserRole::Admin) {
        return true;
    }

    viewer.session.permissions.iter().any(|permission| {
        permission == "manage_dispatch_desk"
            || permission == "manage_loads"
            || permission == "access_admin_portal"
    })
}

fn can_manage_existing_load(
    viewer: &crate::auth_session::ResolvedSession,
    load_owner_user_id: Option<i64>,
) -> bool {
    if viewer.user.primary_role() == Some(UserRole::Admin) {
        return true;
    }

    if viewer.session.permissions.iter().any(|permission| {
        permission == "access_admin_portal" || permission == "manage_dispatch_desk"
    }) {
        return true;
    }

    load_owner_user_id == Some(viewer.user.id)
        && viewer
            .session
            .permissions
            .iter()
            .any(|permission| permission == "manage_loads")
}

fn build_load_lifecycle_actions(
    load: &db::dispatch::LoadRecord,
    legs: &[db::dispatch::LoadProfileLegRecord],
    can_manage: bool,
) -> Vec<LoadLifecycleAction> {
    let locked = legs.iter().any(|leg| {
        leg.booked_carrier_id.is_some() || leg.status_id >= 4 || leg.booked_amount.is_some()
    });
    let locked_reason =
        locked.then_some("Booked or execution-stage loads are locked for this action.".to_string());

    let action = |action: &str, label: &str, tone: &str, enabled: bool, reason: Option<String>| {
        LoadLifecycleAction {
            action: action.into(),
            label: label.into(),
            tone: tone.into(),
            enabled: can_manage && enabled,
            disabled_reason: if can_manage {
                (!enabled).then_some(reason).flatten()
            } else {
                Some("You do not have permission to manage this load lifecycle.".into())
            },
        }
    };

    vec![
        action(
            "publish",
            "Publish",
            "success",
            matches!(load.lifecycle_status.as_str(), "draft" | "revised"),
            Some("Only draft or revised loads can be published.".into()),
        ),
        action(
            "revise",
            "Revise",
            "primary",
            load.lifecycle_status == "published" && !locked,
            locked_reason
                .clone()
                .or_else(|| Some("Only published loads can be revised.".into())),
        ),
        action("clone", "Clone", "secondary", true, None),
        action("template", "Save as template", "info", true, None),
        action(
            "cancel",
            "Cancel",
            "warning",
            matches!(
                load.lifecycle_status.as_str(),
                "draft" | "published" | "revised"
            ) && !locked,
            locked_reason
                .clone()
                .or_else(|| Some("Only active loads can be cancelled.".into())),
        ),
        action(
            "archive",
            "Archive",
            "dark",
            matches!(load.lifecycle_status.as_str(), "cancelled" | "published") && !locked,
            locked_reason.or_else(|| Some("Cancel or complete the load before archiving.".into())),
        ),
    ]
}

fn format_builder_date(value: Option<&chrono::NaiveDateTime>) -> String {
    value
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn format_builder_datetime(value: Option<&chrono::NaiveDateTime>) -> Option<String> {
    value.map(|date| date.format("%Y-%m-%dT%H:%M").to_string())
}

fn normalize_optional_text(value: Option<&String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_date_for_storage(value: &str) -> Result<NaiveDateTime, String> {
    let date = NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| "use YYYY-MM-DD format".to_string())?;
    date.and_hms_opt(0, 0, 0)
        .ok_or_else(|| "date could not be normalized".to_string())
}

fn parse_optional_datetime_for_storage(
    label: &str,
    value: Option<&String>,
) -> Result<Option<NaiveDateTime>, String> {
    let Some(value) = value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    if let Ok(parsed) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Ok(Some(parsed));
    }

    if let Ok(parsed) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(Some(parsed));
    }

    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("{} must use YYYY-MM-DD or YYYY-MM-DDTHH:MM format.", label))?;
    date.and_hms_opt(0, 0, 0)
        .map(Some)
        .ok_or_else(|| format!("{} could not be normalized.", label))
}

fn normalize_freight_mode(value: Option<&String>) -> String {
    canonical_freight_mode(normalize_optional_text(value).as_deref())
}

fn canonical_freight_mode(value: Option<&str>) -> String {
    match value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("FTL")
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "full_truckload" | "truckload" | "ftl" => "FTL".into(),
        "less_than_truckload" | "ltl" => "LTL".into(),
        "dray" | "drayage" => "drayage".into(),
        "rail_intermodal" | "intermodal" => "intermodal".into(),
        "cross_border" | "customs" => "cross_border".into(),
        "forwarding" | "freight_forwarding" => "freight_forwarding".into(),
        "mixed" | "mixed_mode" => "mixed_mode".into(),
        _ => "unsupported".into(),
    }
}

fn validate_mode_specific_payload(payload: &CreateLoadRequest) -> Result<String, String> {
    let mode = normalize_freight_mode(payload.freight_mode.as_ref());
    match mode.as_str() {
        "FTL" => Ok(mode),
        "LTL" => {
            if json_has_any_key(
                payload.securement_data.as_ref().or(payload.container_data.as_ref()),
                &["freight_class", "nmfc", "pieces", "dimensions"],
            ) {
                Ok(mode)
            } else {
                Err("LTL requires freight_class, NMFC, pieces, or dimensions in securement/container mode details.".into())
            }
        }
        "drayage" => {
            if json_has_any_key(
                payload.container_data.as_ref(),
                &["container_number", "chassis_number", "terminal", "port", "free_time_date"],
            ) {
                Ok(mode)
            } else {
                Err("Drayage requires container_number, chassis_number, terminal, port, or free_time_date in container details.".into())
            }
        }
        "intermodal" => {
            if json_has_any_key(
                payload.container_data.as_ref(),
                &["container_number", "rail_ramp", "terminal", "free_time_date"],
            ) {
                Ok(mode)
            } else {
                Err("Intermodal requires container_number, rail_ramp, terminal, or free_time_date in container details.".into())
            }
        }
        "cross_border" => Err("Cross-border workflow is deferred until tax, FX, duties, customs, and localization controls are complete.".into()),
        "freight_forwarding" => Err("Freight forwarding workflow is deferred until legal operating model and forwarding controls are complete.".into()),
        "mixed_mode" => Err("Mixed-mode workflow is deferred until segment-level statuses, documents, and rating are implemented.".into()),
        _ => Err("Freight mode must be FTL, LTL, drayage, or intermodal for this release.".into()),
    }
}

fn json_has_any_key(value: Option<&Value>, keys: &[&str]) -> bool {
    match value {
        Some(Value::Object(map)) => keys.iter().any(|key| map.contains_key(*key)),
        Some(Value::String(text)) => {
            let normalized = text.to_ascii_lowercase();
            keys.iter().any(|key| normalized.contains(key))
        }
        _ => false,
    }
}

async fn record_mode_validation_event(
    pool: &db::DbPool,
    load_id: i64,
    mode_key: &str,
    validation_status: &str,
    validation_notes: &[String],
    actor_user_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE loads
         SET mode_validation_status = $2,
             mode_validation_notes = $3,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(load_id)
    .bind(validation_status)
    .bind(validation_notes)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO load_mode_validation_events (
            load_id, mode_key, validation_status, validation_notes, actor_user_id, created_at
         )
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(mode_key)
    .bind(validation_status)
    .bind(validation_notes)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct LocalizationSettings {
    locale: String,
    time_zone: String,
    distance_unit: String,
    weight_unit: String,
    dimension_unit: String,
    temperature_unit: String,
    currency: String,
}

async fn organization_localization_settings(
    pool: &db::DbPool,
    organization_id: i64,
) -> Result<LocalizationSettings, sqlx::Error> {
    sqlx::query_as::<_, (String, String, String, String, String, String, String)>(
        "SELECT locale, time_zone, distance_unit, weight_unit, dimension_unit, temperature_unit, currency
         FROM organization_localization_settings
         WHERE organization_id = $1
         LIMIT 1",
    )
    .bind(organization_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(
                locale,
                time_zone,
                distance_unit,
                weight_unit,
                dimension_unit,
                temperature_unit,
                currency,
            )| LocalizationSettings {
                locale,
                time_zone,
                distance_unit,
                weight_unit,
                dimension_unit,
                temperature_unit,
                currency,
            },
        )
        .unwrap_or_else(|| LocalizationSettings {
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            distance_unit: "mi".into(),
            weight_unit: "LBS".into(),
            dimension_unit: "in".into(),
            temperature_unit: "F".into(),
            currency: "USD".into(),
        })
    })
}

async fn persist_load_localization_snapshot(
    pool: &db::DbPool,
    organization_id: i64,
    load_id: i64,
    source_weight: f64,
    source_weight_unit: &str,
    rate_currency: Option<&str>,
    actor_user_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    let settings = organization_localization_settings(pool, organization_id).await?;
    let canonical_weight_lbs = convert_weight_to_lbs(source_weight, source_weight_unit);
    let display_weight = canonical_weight_lbs
        .map(|weight_lbs| convert_weight_from_lbs(weight_lbs, &settings.weight_unit));
    let mut validation_notes = Vec::new();
    if canonical_weight_lbs.is_none() {
        validation_notes.push(format!(
            "Unsupported source weight unit '{}'; canonical LBS could not be calculated.",
            source_weight_unit
        ));
    }

    sqlx::query(
        "INSERT INTO load_localization_snapshots (
            organization_id, load_id, locale, time_zone, canonical_distance_unit,
            display_distance_unit, canonical_weight_unit, display_weight_unit,
            canonical_weight_lbs, display_weight, dimension_unit, temperature_unit,
            currency, validation_notes, created_by_user_id, created_at
         )
         VALUES ($1, $2, $3, $4, 'mi', $5, 'LBS', $6, $7, $8, $9, $10, $11, $12, $13, CURRENT_TIMESTAMP)",
    )
    .bind(organization_id)
    .bind(load_id)
    .bind(&settings.locale)
    .bind(&settings.time_zone)
    .bind(&settings.distance_unit)
    .bind(&settings.weight_unit)
    .bind(canonical_weight_lbs)
    .bind(display_weight)
    .bind(&settings.dimension_unit)
    .bind(&settings.temperature_unit)
    .bind(rate_currency.unwrap_or(&settings.currency))
    .bind(&validation_notes)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn convert_weight_to_lbs(weight: f64, unit: &str) -> Option<f64> {
    match unit.trim().to_ascii_uppercase().as_str() {
        "LBS" | "LB" => Some(weight),
        "KG" => Some(weight * 2.204_622_621_8),
        "MTON" | "METRIC_TON" => Some(weight * 2_204.622_621_8),
        _ => None,
    }
}

fn convert_weight_from_lbs(weight_lbs: f64, unit: &str) -> f64 {
    match unit.trim().to_ascii_uppercase().as_str() {
        "KG" => weight_lbs / 2.204_622_621_8,
        "MTON" | "METRIC_TON" => weight_lbs / 2_204.622_621_8,
        _ => weight_lbs,
    }
}

fn normalize_visibility(value: Option<&String>) -> Result<String, String> {
    let visibility = normalize_optional_text(value)
        .unwrap_or_else(|| "public".into())
        .to_ascii_lowercase();
    match visibility.as_str() {
        "public" | "private" | "contract" | "internal" => Ok(visibility),
        _ => Err("Visibility must be public, private, contract, or internal.".into()),
    }
}

fn clone_non_null_json(value: &Option<Value>) -> Option<Value> {
    value.clone().filter(|value| !value.is_null())
}

fn can_manage_carrier_network(viewer: &auth_session::ResolvedSession) -> bool {
    matches!(
        viewer.user.primary_role(),
        Some(UserRole::Admin | UserRole::Shipper | UserRole::Broker | UserRole::FreightForwarder)
    )
}

async fn build_carrier_network_screen(
    state: &AppState,
    viewer: Option<&auth_session::ResolvedSession>,
) -> shared::CarrierNetworkScreen {
    let Some(viewer) = viewer else {
        return shared::CarrierNetworkScreen {
            can_manage: false,
            owner_label: "Signed out".into(),
            carrier_options: Vec::new(),
            rows: Vec::new(),
            notes: vec!["Sign in before managing private carrier networks.".into()],
        };
    };
    let can_manage = can_manage_carrier_network(viewer);
    let Some(pool) = state.pool.as_ref() else {
        return shared::CarrierNetworkScreen {
            can_manage,
            owner_label: viewer.user.name.clone(),
            carrier_options: Vec::new(),
            rows: Vec::new(),
            notes: vec![format!(
                "Carrier network data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        };
    };
    if !can_manage {
        return shared::CarrierNetworkScreen {
            can_manage: false,
            owner_label: viewer.user.name.clone(),
            carrier_options: Vec::new(),
            rows: Vec::new(),
            notes: vec![
                "Carrier accounts can view private freight assigned to them but cannot manage shipper private networks.".into(),
            ],
        };
    }

    let carrier_options = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT id, name, company_name
         FROM users
         WHERE role_id = $1
         ORDER BY name
         LIMIT 200",
    )
    .bind(UserRole::Carrier.legacy_id())
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(user_id, name, company)| shared::CarrierNetworkOption {
        user_id: user_id.max(0) as u64,
        label: company
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("{} ({})", name, value))
            .unwrap_or(name),
    })
    .collect::<Vec<_>>();

    let rows = sqlx::query_as::<
        _,
        (
            i64,
            i64,
            String,
            Option<String>,
            String,
            Option<String>,
            Option<String>,
            NaiveDate,
            Option<NaiveDate>,
        ),
    >(
        "SELECT network.id, network.carrier_user_id, carrier.name, carrier.company_name,
                network.relationship_status, network.carrier_group_key, network.notes,
                network.effective_from, network.effective_to
         FROM carrier_network_memberships network
         INNER JOIN users carrier ON carrier.id = network.carrier_user_id
         WHERE network.owner_user_id = $1
         ORDER BY
            CASE network.relationship_status
                WHEN 'blocked' THEN 0
                WHEN 'preferred' THEN 1
                WHEN 'approved' THEN 2
                ELSE 3
            END,
            carrier.name",
    )
    .bind(viewer.user.id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(
        |(
            id,
            carrier_user_id,
            carrier_name,
            company_name,
            relationship_status,
            carrier_group_key,
            notes,
            effective_from,
            effective_to,
        )| shared::CarrierNetworkRow {
            id: id.max(0) as u64,
            carrier_user_id: carrier_user_id.max(0) as u64,
            carrier_label: company_name
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!("{} ({})", carrier_name, value))
                .unwrap_or(carrier_name),
            relationship_status,
            carrier_group_key,
            notes,
            effective_from: effective_from.format("%Y-%m-%d").to_string(),
            effective_to: effective_to.map(|value| value.format("%Y-%m-%d").to_string()),
        },
    )
    .collect::<Vec<_>>();

    shared::CarrierNetworkScreen {
        can_manage,
        owner_label: viewer.user.name.clone(),
        carrier_options,
        rows,
        notes: vec![
            "Private and contract freight is visible only to approved, preferred, or backup carriers in this network.".into(),
            "Blocked carriers are excluded from private freight visibility and carrier self-booking.".into(),
        ],
    }
}

async fn build_carrier_match_screen(
    state: &AppState,
    viewer: Option<&auth_session::ResolvedSession>,
    leg_id: i64,
) -> CarrierMatchScreen {
    let Some(viewer) = viewer else {
        return empty_carrier_match_screen(leg_id, "Sign in before viewing carrier matches.");
    };
    let allowed = matches!(
        viewer.user.primary_role(),
        Some(UserRole::Admin | UserRole::Shipper | UserRole::Broker | UserRole::FreightForwarder)
    );
    if !allowed {
        return empty_carrier_match_screen(
            leg_id,
            "Only owner, broker, forwarder, and admin users can view carrier recommendations.",
        );
    }
    let Some(pool) = state.pool.as_ref() else {
        return empty_carrier_match_screen(
            leg_id,
            "Carrier matching is unavailable while the database is offline.",
        );
    };

    let leg = sqlx::query_as::<
        _,
        (
            i64,
            i64,
            Option<String>,
            String,
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<f64>,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT leg.id, load.id, load.load_number, load.title, load.user_id, load.visibility,
                equipment.name, commodity.name, load.service_level,
                pickup.name, leg.price::double precision, handoff.status, sync_issue.title
         FROM load_legs leg
         INNER JOIN loads load ON load.id = leg.load_id AND load.deleted_at IS NULL
         LEFT JOIN equipments equipment ON equipment.id = load.equipment_id
         LEFT JOIN commodity_types commodity ON commodity.id = load.commodity_type_id
         LEFT JOIN locations pickup ON pickup.id = leg.pickup_location_id
         LEFT JOIN stloads_handoffs handoff ON handoff.id = (
            SELECT handoff_inner.id FROM stloads_handoffs handoff_inner
            WHERE handoff_inner.load_id = load.id ORDER BY handoff_inner.id DESC LIMIT 1
         )
         LEFT JOIN stloads_sync_errors sync_issue ON sync_issue.id = (
            SELECT sync_issue_inner.id FROM stloads_sync_errors sync_issue_inner
            WHERE sync_issue_inner.handoff_id = handoff.id AND sync_issue_inner.resolved = FALSE
            ORDER BY sync_issue_inner.id DESC LIMIT 1
         )
         WHERE leg.id = $1 AND leg.deleted_at IS NULL",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some((
        leg_id,
        load_id,
        load_number,
        load_title,
        owner_user_id,
        visibility,
        equipment_name,
        commodity_name,
        service_level,
        pickup_name,
        price,
        handoff_status,
        sync_issue,
    )) = leg
    else {
        return empty_carrier_match_screen(leg_id, "Load leg was not found.");
    };

    if viewer.user.primary_role() != Some(UserRole::Admin) && viewer.user.id != owner_user_id {
        return empty_carrier_match_screen(
            leg_id,
            "This leg is outside the signed-in user's scope.",
        );
    }

    let candidates = sqlx::query_as::<
        _,
        (
            i64,
            String,
            Option<String>,
            Option<String>,
            Option<Vec<String>>,
            Option<Vec<String>>,
            Option<Vec<String>>,
            Option<Vec<String>>,
            Option<Vec<String>>,
            Option<String>,
            Option<i32>,
            Option<f64>,
            i64,
        ),
    >(
        "SELECT carrier.id,
                carrier.name,
                carrier.company_name,
                network.relationship_status,
                capacity.equipment_types,
                capacity.operating_regions,
                capacity.preferred_commodities,
                capacity.service_levels,
                capacity.certifications,
                capacity.availability_status,
                capacity.available_power_units,
                capacity.insurance_limit_usd,
                COUNT(document.id)::bigint AS document_count
         FROM users carrier
         LEFT JOIN carrier_network_memberships network
            ON network.carrier_user_id = carrier.id
           AND network.owner_user_id = $1
           AND network.effective_from <= CURRENT_DATE
           AND (network.effective_to IS NULL OR network.effective_to >= CURRENT_DATE)
         LEFT JOIN carrier_capacity_profiles capacity ON capacity.carrier_user_id = carrier.id
         LEFT JOIN kyc_documents document ON document.user_id = carrier.id
         WHERE carrier.role_id = $2
         GROUP BY carrier.id, carrier.name, carrier.company_name, network.relationship_status,
                  capacity.equipment_types, capacity.operating_regions, capacity.preferred_commodities,
                  capacity.service_levels, capacity.certifications, capacity.availability_status,
                  capacity.available_power_units, capacity.insurance_limit_usd
         ORDER BY carrier.name
         LIMIT 100",
    )
    .bind(owner_user_id)
    .bind(UserRole::Carrier.legacy_id())
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut rows = candidates
        .into_iter()
        .map(
            |(
                carrier_id,
                carrier_name,
                company_name,
                relationship_status,
                equipment_types,
                operating_regions,
                preferred_commodities,
                service_levels,
                certifications,
                availability_status,
                available_power_units,
                insurance_limit_usd,
                document_count,
            )| {
                score_carrier_match(
                    carrier_id,
                    carrier_name,
                    company_name,
                    relationship_status,
                    &visibility,
                    equipment_name.as_deref(),
                    commodity_name.as_deref(),
                    service_level.as_deref(),
                    pickup_name.as_deref(),
                    price,
                    handoff_status.as_deref(),
                    sync_issue.as_deref(),
                    equipment_types.unwrap_or_default(),
                    operating_regions.unwrap_or_default(),
                    preferred_commodities.unwrap_or_default(),
                    service_levels.unwrap_or_default(),
                    certifications.unwrap_or_default(),
                    availability_status.unwrap_or_else(|| "unavailable".into()),
                    available_power_units.unwrap_or(0),
                    insurance_limit_usd.unwrap_or(0.0),
                    document_count,
                )
            },
        )
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .eligible
            .cmp(&left.eligible)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| left.carrier_label.cmp(&right.carrier_label))
    });

    CarrierMatchScreen {
        leg_id: leg_id.max(0) as u64,
        load_id: load_id.max(0) as u64,
        load_label: load_number.unwrap_or_else(|| load_title.clone()),
        rows,
        notes: vec![
            "Scores combine lane fit, equipment, service, commodity, relationship, compliance documents, price readiness, tracking health, availability, and insurance evidence.".into(),
            "Blocked rows are retained with reasons so operators can explain why a carrier is not eligible.".into(),
        ],
    }
}

fn empty_carrier_match_screen(leg_id: i64, note: &str) -> CarrierMatchScreen {
    CarrierMatchScreen {
        leg_id: leg_id.max(0) as u64,
        load_id: 0,
        load_label: "Unavailable".into(),
        rows: Vec::new(),
        notes: vec![note.into()],
    }
}

#[allow(clippy::too_many_arguments)]
fn score_carrier_match(
    carrier_user_id: i64,
    carrier_name: String,
    company_name: Option<String>,
    relationship_status: Option<String>,
    load_visibility: &str,
    equipment_name: Option<&str>,
    commodity_name: Option<&str>,
    service_level: Option<&str>,
    pickup_name: Option<&str>,
    price: Option<f64>,
    handoff_status: Option<&str>,
    sync_issue: Option<&str>,
    equipment_types: Vec<String>,
    operating_regions: Vec<String>,
    preferred_commodities: Vec<String>,
    service_levels: Vec<String>,
    certifications: Vec<String>,
    availability_status: String,
    available_power_units: i32,
    insurance_limit_usd: f64,
    document_count: i64,
) -> CarrierMatchRow {
    let mut score = 30_i32;
    let mut explanation = Vec::new();
    let mut blocked_reasons = Vec::new();
    let relationship = relationship_status.as_deref();

    if relationship == Some("blocked") {
        blocked_reasons.push("Carrier is blocked by the customer private network.".into());
    } else if matches!(load_visibility, "private" | "contract" | "internal")
        && !matches!(relationship, Some("approved" | "preferred" | "backup"))
    {
        blocked_reasons.push("Carrier is not approved for this private freight.".into());
    }

    match relationship {
        Some("preferred") => {
            score += 18;
            explanation.push("Preferred carrier relationship.".into());
        }
        Some("approved") => {
            score += 12;
            explanation.push("Approved carrier relationship.".into());
        }
        Some("backup") => {
            score += 8;
            explanation.push("Backup carrier relationship.".into());
        }
        _ => {}
    }

    match availability_status.as_str() {
        "available" => {
            score += 12;
            explanation.push("Carrier is marked available.".into());
        }
        "limited" | "seasonal" => {
            score += 4;
            explanation.push("Carrier capacity is limited or seasonal.".into());
        }
        _ => blocked_reasons.push("Carrier is unavailable or paused.".into()),
    }

    if available_power_units > 0 {
        score += 8;
        explanation.push(format!(
            "{} available power unit(s).",
            available_power_units
        ));
    } else {
        blocked_reasons.push("No available power units are recorded.".into());
    }
    if insurance_limit_usd > 0.0 {
        score += 8;
        explanation.push(format!(
            "Insurance limit recorded at USD {:.0}.",
            insurance_limit_usd
        ));
    } else {
        blocked_reasons.push("Insurance limit is missing.".into());
    }
    if document_count >= 5 {
        score += 8;
        explanation.push("Carrier packet checklist has core documents.".into());
    } else {
        let note = "Carrier packet is missing one or more core documents: W-9, COI, authority, operating agreement, or banking/payout setup.";
        if matches!(load_visibility, "private" | "contract" | "internal") {
            blocked_reasons.push(note.into());
        } else {
            explanation.push(note.into());
        }
    }
    if capacity_value_matches(equipment_name, &equipment_types) {
        score += 12;
        explanation.push("Equipment matches carrier capacity.".into());
    }
    if capacity_value_matches(commodity_name, &preferred_commodities) {
        score += 6;
        explanation.push("Commodity matches carrier preference.".into());
    }
    if capacity_value_matches(service_level, &service_levels) {
        score += 6;
        explanation.push("Service level matches carrier profile.".into());
    }
    if capacity_value_matches(pickup_name, &operating_regions) {
        score += 8;
        explanation.push("Pickup geography matches operating region.".into());
    }
    if price.is_some() {
        score += 5;
        explanation.push("Rate is available for price comparison.".into());
    }
    if sync_issue.is_none() {
        score += 5;
        explanation.push("No active tracking/TMS sync issue.".into());
    } else {
        explanation.push("Active tracking/TMS issue may reduce confidence.".into());
    }
    if handoff_status == Some("published") {
        score += 3;
        explanation.push("Freight is published for marketplace activity.".into());
    }
    if certifications.iter().any(|value| value.contains("hazmat")) {
        explanation.push("Hazmat certification is recorded.".into());
    }

    let eligible = blocked_reasons.is_empty();
    CarrierMatchRow {
        carrier_user_id: carrier_user_id.max(0) as u64,
        carrier_label: company_name
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!("{} ({})", carrier_name, value))
            .unwrap_or(carrier_name),
        score: if eligible {
            score.clamp(0, 98) as u8
        } else {
            score.clamp(0, 35) as u8
        },
        eligible,
        relationship_status,
        explanation,
        blocked_reasons,
    }
}

fn capacity_value_matches(value: Option<&str>, candidates: &[String]) -> bool {
    let Some(value) = value else {
        return false;
    };
    let value = value.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    candidates.iter().any(|candidate| {
        let candidate = candidate
            .trim()
            .to_ascii_lowercase()
            .replace([' ', '-'], "_");
        !candidate.is_empty() && (value.contains(&candidate) || candidate.contains(&value))
    })
}

async fn carrier_can_book_leg(
    pool: &db::DbPool,
    leg_id: i64,
    carrier_user_id: i64,
) -> Result<bool, sqlx::Error> {
    let allowed = sqlx::query_scalar::<_, bool>(
        "SELECT
            CASE
                WHEN leg.booked_carrier_id = $2 THEN TRUE
                WHEN EXISTS (
                    SELECT 1
                    FROM carrier_network_memberships blocked_network
                    WHERE blocked_network.owner_user_id = load.user_id
                      AND blocked_network.carrier_user_id = $2
                      AND blocked_network.relationship_status = 'blocked'
                      AND blocked_network.effective_from <= CURRENT_DATE
                      AND (blocked_network.effective_to IS NULL OR blocked_network.effective_to >= CURRENT_DATE)
                ) THEN FALSE
                WHEN NOT EXISTS (
                    SELECT 1
                    FROM carrier_authority_verifications verification
                    WHERE verification.carrier_user_id = $2
                      AND verification.authority_status = 'active'
                      AND verification.insurance_status = 'verified'
                      AND verification.insurance_expires_at IS NOT NULL
                      AND verification.insurance_expires_at >= CURRENT_DATE
                ) THEN FALSE
                WHEN EXISTS (
                    SELECT 1
                    FROM driver_equipment_safety_compliance safety
                    WHERE safety.carrier_user_id = $2
                      AND (
                          safety.restricted_freight_blocking = TRUE
                          OR safety.driver_compliance_status IN ('expired', 'blocked')
                          OR safety.equipment_compliance_status IN ('expired', 'blocked')
                          OR (safety.cdl_expires_at IS NOT NULL AND safety.cdl_expires_at < CURRENT_DATE)
                          OR (safety.medical_card_expires_at IS NOT NULL AND safety.medical_card_expires_at < CURRENT_DATE)
                          OR (safety.inspection_expires_at IS NOT NULL AND safety.inspection_expires_at < CURRENT_DATE)
                          OR safety.maintenance_status IN ('overdue', 'blocked')
                          OR safety.equipment_insurance_status IN ('expired', 'rejected', 'missing')
                      )
                ) THEN FALSE
                WHEN EXISTS (
                    SELECT 1
                    FROM sanctions_tax_profiles profile
                    WHERE profile.user_id = $2
                      AND (
                          profile.sanctions_status IN ('possible_match', 'blocked')
                          OR profile.beneficial_owner_status = 'blocked'
                      )
                ) THEN FALSE
                WHEN EXISTS (
                    SELECT 1
                    FROM risk_review_items risk
                    WHERE risk.hold_booking = TRUE
                      AND risk.status IN ('open', 'in_review', 'blocked')
                      AND (risk.subject_user_id = $2 OR risk.leg_id = $1)
                ) THEN FALSE
                WHEN load.visibility = 'public' THEN TRUE
                WHEN EXISTS (
                    SELECT 1
                    FROM carrier_network_memberships allowed_network
                    WHERE allowed_network.owner_user_id = load.user_id
                      AND allowed_network.carrier_user_id = $2
                      AND allowed_network.relationship_status IN ('approved', 'preferred', 'backup')
                      AND allowed_network.effective_from <= CURRENT_DATE
                      AND (allowed_network.effective_to IS NULL OR allowed_network.effective_to >= CURRENT_DATE)
                ) THEN TRUE
                ELSE FALSE
            END
         FROM load_legs leg
         INNER JOIN loads load ON load.id = leg.load_id AND load.deleted_at IS NULL
         WHERE leg.id = $1 AND leg.deleted_at IS NULL",
    )
    .bind(leg_id)
    .bind(carrier_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(allowed.unwrap_or(false))
}

async fn carrier_restricted_packet_blockers(
    pool: &db::DbPool,
    leg_id: i64,
    carrier_user_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let Some(visibility) = sqlx::query_scalar::<_, String>(
        "SELECT load.visibility
         FROM load_legs leg
         INNER JOIN loads load ON load.id = leg.load_id AND load.deleted_at IS NULL
         WHERE leg.id = $1 AND leg.deleted_at IS NULL
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(vec!["Load leg is unavailable for packet review".into()]);
    };

    if !matches!(visibility.as_str(), "private" | "contract" | "internal") {
        return Ok(Vec::new());
    }

    let document_types = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(string_agg(lower(document_type), ','), '')
         FROM kyc_documents
         WHERE user_id = $1",
    )
    .bind(carrier_user_id)
    .fetch_one(pool)
    .await?;
    let insurance_limit = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT insurance_limit_usd::double precision
         FROM carrier_capacity_profiles
         WHERE carrier_user_id = $1
         LIMIT 1",
    )
    .bind(carrier_user_id)
    .fetch_optional(pool)
    .await?
    .flatten()
    .unwrap_or(0.0);

    let normalized_docs = document_types.replace([' ', '-'], "_");
    let mut blockers = Vec::new();
    for (needle, label) in [
        ("w9", "W-9"),
        ("coi", "certificate of insurance"),
        ("authority", "operating authority"),
        ("operating_agreement", "operating agreement"),
        ("banking", "banking/payout setup"),
    ] {
        if !normalized_docs.contains(needle) {
            blockers.push(format!("missing {}", label));
        }
    }
    if insurance_limit <= 0.0 {
        blockers.push("missing insurance limit on carrier capacity profile".into());
    }

    Ok(blockers)
}

async fn apply_customer_contract_lane(
    pool: &db::DbPool,
    organization_id: i64,
    params: &mut CreateLoadParams,
) -> Result<(), String> {
    let Some(lane_id) = params.customer_contract_lane_id else {
        return Ok(());
    };

    let lane = find_active_customer_contract_lane(pool, organization_id, lane_id)
        .await
        .map_err(|error| format!("Contract lane lookup failed: {}", error))?
        .ok_or_else(|| {
            "Selected contract lane is inactive, expired, or unavailable.".to_string()
        })?;

    if lane.contract_id != params.customer_contract_id.unwrap_or(lane.contract_id) {
        return Err("Selected contract lane does not belong to the selected contract.".into());
    }

    params.customer_contract_id = Some(lane.contract_id);
    params.customer_contract_lane_id = Some(lane.id);
    params.contract_rate = Some(lane.contracted_rate);
    params.contract_rate_currency = Some(lane.rate_currency.clone());
    params.contract_posting_behavior = Some(lane.posting_behavior.clone());
    params.contract_service_rules = lane.service_rules.clone();
    params.visibility = lane.posting_behavior;
    params.freight_mode = lane.freight_mode;
    params.service_level = lane.service_level.or_else(|| params.service_level.clone());
    params.accessorial_flags = lane
        .accessorial_rules
        .or_else(|| params.accessorial_flags.clone());

    Ok(())
}

// Location resolution is called from load creation/import paths where every
// field maps directly to user input or facility fallback evidence.
#[allow(clippy::too_many_arguments)]
async fn resolve_leg_location_reference(
    pool: &db::DbPool,
    label: &str,
    location_id: Option<u64>,
    address: Option<&str>,
    city: Option<&str>,
    country: Option<&str>,
    place_id: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    organization_id: i64,
    facility_type: &str,
) -> Result<i64, String> {
    let normalized_address = address.map(str::trim).unwrap_or_default();
    if !normalized_address.is_empty() {
        validate_coordinates(latitude, longitude)?;
        let country_id = match country.map(str::trim).filter(|value| !value.is_empty()) {
            Some(country_name) => Some(
                ensure_country_by_name(pool, country_name)
                    .await
                    .map_err(|error| format!("country lookup failed: {}", error))?
                    .id,
            ),
            None => None,
        };

        let city_id = match (
            city.map(str::trim).filter(|value| !value.is_empty()),
            country_id,
        ) {
            (Some(city_name), Some(country_id)) => Some(
                ensure_city_by_name(pool, country_id, city_name)
                    .await
                    .map_err(|error| format!("city lookup failed: {}", error))?
                    .id,
            ),
            _ => None,
        };

        let location = upsert_location(pool, None, normalized_address, city_id, country_id)
            .await
            .map_err(|error| format!("location save failed: {}", error))?;
        persist_location_geocode_and_facility(
            pool,
            location.id,
            organization_id,
            normalized_address,
            place_id,
            latitude,
            longitude,
            facility_type,
        )
        .await
        .map_err(|error| format!("facility normalization failed: {}", error))?;

        return Ok(location.id);
    }

    if let Some(location_id) = location_id {
        return Ok(location_id as i64);
    }

    Err(format!(
        "select or autocomplete a valid {} location before saving the load.",
        label
    ))
}

async fn build_load_mutation_inputs(
    pool: &db::DbPool,
    owner_user_id: i64,
    organization_id: i64,
    payload: &CreateLoadRequest,
) -> Result<(CreateLoadParams, Vec<CreateLoadLegParams>), CreateLoadResponse> {
    let title = payload.title.trim().to_string();
    if title.is_empty() {
        return Err(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Enter a load title before saving.".into(),
        });
    }

    if payload.weight <= 0.0 {
        return Err(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Weight must be greater than zero.".into(),
        });
    }

    if !matches!(payload.weight_unit.as_str(), "LBS" | "KG" | "MTON") {
        return Err(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Weight unit must be LBS, KG, or MTON.".into(),
        });
    }

    if payload.legs.is_empty() {
        return Err(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Add at least one leg before saving the load.".into(),
        });
    }

    let mut leg_params = Vec::new();
    for (index, leg) in payload.legs.iter().enumerate() {
        let pickup_address = leg
            .pickup_location_address
            .as_deref()
            .map(str::trim)
            .unwrap_or_default();
        let delivery_address = leg
            .delivery_location_address
            .as_deref()
            .map(str::trim)
            .unwrap_or_default();

        let same_selected_location =
            leg.pickup_location_id.is_some() && leg.pickup_location_id == leg.delivery_location_id;
        let same_autocomplete_address = !pickup_address.is_empty()
            && !delivery_address.is_empty()
            && pickup_address.eq_ignore_ascii_case(delivery_address);

        if same_selected_location || same_autocomplete_address {
            return Err(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!(
                    "Leg {} must use different pickup and delivery locations.",
                    index + 1
                ),
            });
        }

        if !matches!(leg.bid_status.as_str(), "Fixed" | "Open") {
            return Err(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!("Leg {} must use Fixed or Open bid status.", index + 1),
            });
        }

        if leg.price < 0.0 {
            return Err(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!("Leg {} price must be zero or greater.", index + 1),
            });
        }

        let pickup_date = match parse_date_for_storage(&leg.pickup_date) {
            Ok(value) => value,
            Err(message) => {
                return Err(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} pickup date error: {}", index + 1, message),
                });
            }
        };

        let delivery_date = match parse_date_for_storage(&leg.delivery_date) {
            Ok(value) => value,
            Err(message) => {
                return Err(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} delivery date error: {}", index + 1, message),
                });
            }
        };

        if delivery_date < pickup_date {
            return Err(CreateLoadResponse {
                success: false,
                load_id: None,
                load_number: None,
                leg_count: index as u64,
                message: format!(
                    "Leg {} delivery date must be on or after the pickup date.",
                    index + 1
                ),
            });
        }

        let pickup_location_id = match resolve_leg_location_reference(
            pool,
            "pickup",
            leg.pickup_location_id,
            leg.pickup_location_address.as_deref(),
            leg.pickup_city.as_deref(),
            leg.pickup_country.as_deref(),
            leg.pickup_place_id.as_deref(),
            leg.pickup_latitude,
            leg.pickup_longitude,
            organization_id,
            "pickup",
        )
        .await
        {
            Ok(value) => value,
            Err(message) => {
                return Err(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} pickup error: {}", index + 1, message),
                });
            }
        };

        let delivery_location_id = match resolve_leg_location_reference(
            pool,
            "delivery",
            leg.delivery_location_id,
            leg.delivery_location_address.as_deref(),
            leg.delivery_city.as_deref(),
            leg.delivery_country.as_deref(),
            leg.delivery_place_id.as_deref(),
            leg.delivery_latitude,
            leg.delivery_longitude,
            organization_id,
            "delivery",
        )
        .await
        {
            Ok(value) => value,
            Err(message) => {
                return Err(CreateLoadResponse {
                    success: false,
                    load_id: None,
                    load_number: None,
                    leg_count: index as u64,
                    message: format!("Leg {} delivery error: {}", index + 1, message),
                });
            }
        };

        leg_params.push(CreateLoadLegParams {
            pickup_location_id,
            delivery_location_id,
            pickup_date,
            delivery_date,
            bid_status: leg.bid_status.clone(),
            price: leg.price,
        });
    }

    let visibility = normalize_visibility(payload.visibility.as_ref()).map_err(|message| {
        CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message,
        }
    })?;
    let appointment_window_start = parse_optional_datetime_for_storage(
        "Appointment window start",
        payload.appointment_window_start.as_ref(),
    )
    .map_err(|message| CreateLoadResponse {
        success: false,
        load_id: None,
        load_number: None,
        leg_count: 0,
        message,
    })?;
    let appointment_window_end = parse_optional_datetime_for_storage(
        "Appointment window end",
        payload.appointment_window_end.as_ref(),
    )
    .map_err(|message| CreateLoadResponse {
        success: false,
        load_id: None,
        load_number: None,
        leg_count: 0,
        message,
    })?;
    if let (Some(start), Some(end)) = (appointment_window_start, appointment_window_end)
        && end < start
    {
        return Err(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Appointment window end must be after the start.".into(),
        });
    }
    let freight_mode =
        validate_mode_specific_payload(payload).map_err(|message| CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message,
        })?;

    let mut params = CreateLoadParams {
        title,
        owner_user_id,
        load_type_id: payload.load_type_id as i64,
        equipment_id: payload.equipment_id as i64,
        commodity_type_id: payload.commodity_type_id as i64,
        customer_contract_id: payload.customer_contract_id.map(|value| value as i64),
        customer_contract_lane_id: payload.customer_contract_lane_id.map(|value| value as i64),
        contract_rate: None,
        contract_rate_currency: None,
        contract_posting_behavior: None,
        contract_service_rules: None,
        freight_mode,
        visibility,
        service_level: normalize_optional_text(payload.service_level.as_ref()),
        customer_reference: normalize_optional_text(payload.customer_reference.as_ref()),
        po_number: normalize_optional_text(payload.po_number.as_ref()),
        pickup_appointment_ref: normalize_optional_text(payload.pickup_appointment_ref.as_ref()),
        delivery_appointment_ref: normalize_optional_text(
            payload.delivery_appointment_ref.as_ref(),
        ),
        facility_contact_name: normalize_optional_text(payload.facility_contact_name.as_ref()),
        facility_contact_phone: normalize_optional_text(payload.facility_contact_phone.as_ref()),
        facility_contact_email: normalize_optional_text(payload.facility_contact_email.as_ref()),
        appointment_window_start,
        appointment_window_end,
        accessorial_flags: clone_non_null_json(&payload.accessorial_flags),
        weight_unit: payload.weight_unit.clone(),
        weight: payload.weight,
        temperature_data: clone_non_null_json(&payload.temperature_data),
        container_data: clone_non_null_json(&payload.container_data),
        securement_data: clone_non_null_json(&payload.securement_data),
        special_instructions: payload
            .special_instructions
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        is_hazardous: payload.is_hazardous,
        is_temperature_controlled: payload.is_temperature_controlled,
    };

    if let Err(message) = apply_customer_contract_lane(pool, organization_id, &mut params).await {
        return Err(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message,
        });
    }

    Ok((params, leg_params))
}

#[derive(Debug, Clone)]
struct CsvLoadImportRow {
    row_number: u64,
    raw_payload: Value,
    payload: Option<CreateLoadRequest>,
    errors: Vec<String>,
}

impl CsvLoadImportRow {
    fn to_result(&self) -> BulkLoadImportRowResult {
        let leg = self
            .payload
            .as_ref()
            .and_then(|payload| payload.legs.first());
        BulkLoadImportRowResult {
            row_number: self.row_number,
            valid: self.errors.is_empty(),
            errors: self.errors.clone(),
            load_id: None,
            load_number: None,
            title: self.payload.as_ref().map(|payload| payload.title.clone()),
            pickup_label: leg.and_then(|leg| leg.pickup_location_address.clone()),
            delivery_label: leg.and_then(|leg| leg.delivery_location_address.clone()),
            price: leg.map(|leg| leg.price),
        }
    }

    fn from_api_payload(payload: &CreateLoadRequest) -> Self {
        Self {
            row_number: 1,
            raw_payload: json!({ "source": "api_post", "title": payload.title }),
            payload: Some(payload.clone()),
            errors: vec![],
        }
    }
}

fn parse_bulk_load_csv(csv: &str) -> Result<Vec<CsvLoadImportRow>, String> {
    let records = parse_csv_records(csv)?;
    if records.is_empty() {
        return Err("CSV import requires a header row.".into());
    }

    let headers = records[0]
        .iter()
        .map(|value| normalize_csv_header(value))
        .collect::<Vec<_>>();
    if headers.iter().all(|header| header.is_empty()) {
        return Err("CSV import requires named headers.".into());
    }

    let mut rows = Vec::new();
    for (offset, values) in records.into_iter().skip(1).enumerate() {
        if values.iter().all(|value| value.trim().is_empty()) {
            continue;
        }

        let mut fields = HashMap::new();
        for (index, header) in headers.iter().enumerate() {
            if !header.is_empty() {
                fields.insert(
                    header.clone(),
                    values.get(index).cloned().unwrap_or_default(),
                );
            }
        }
        rows.push(csv_fields_to_load_request(offset as u64 + 2, fields));
    }

    if rows.is_empty() {
        return Err("CSV import requires at least one data row.".into());
    }

    Ok(rows)
}

fn csv_fields_to_load_request(
    row_number: u64,
    fields: HashMap<String, String>,
) -> CsvLoadImportRow {
    let mut errors = Vec::new();
    let required_text = |key: &str, label: &str, errors: &mut Vec<String>| -> String {
        let value = fields
            .get(key)
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if value.is_empty() {
            errors.push(format!("{} is required", label));
        }
        value
    };
    let required_u64 = |key: &str, label: &str, errors: &mut Vec<String>| -> u64 {
        let value = required_text(key, label, errors);
        value.parse::<u64>().unwrap_or_else(|_| {
            if !value.is_empty() {
                errors.push(format!("{} must be a positive integer", label));
            }
            0
        })
    };
    let required_f64 = |key: &str, label: &str, errors: &mut Vec<String>| -> f64 {
        let value = required_text(key, label, errors);
        value.parse::<f64>().unwrap_or_else(|_| {
            if !value.is_empty() {
                errors.push(format!("{} must be a number", label));
            }
            0.0
        })
    };
    let optional_u64 = |key: &str, errors: &mut Vec<String>| -> Option<u64> {
        fields
            .get(key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| {
                value.parse::<u64>().map_err(|_| {
                    errors.push(format!("{} must be a positive integer", key));
                })
            })
            .and_then(Result::ok)
    };

    let title = required_text("title", "title", &mut errors);
    let load_type_id = required_u64("load_type_id", "load_type_id", &mut errors);
    let equipment_id = required_u64("equipment_id", "equipment_id", &mut errors);
    let commodity_type_id = required_u64("commodity_type_id", "commodity_type_id", &mut errors);
    let pickup_address = required_text("pickup_address", "pickup_address", &mut errors);
    let delivery_address = required_text("delivery_address", "delivery_address", &mut errors);
    let pickup_date = required_text("pickup_date", "pickup_date", &mut errors);
    let delivery_date = required_text("delivery_date", "delivery_date", &mut errors);
    let price = required_f64("price", "price", &mut errors);
    let weight = required_f64("weight", "weight", &mut errors);

    if !pickup_date.is_empty() && parse_date_for_storage(&pickup_date).is_err() {
        errors.push("pickup_date must be YYYY-MM-DD or YYYY-MM-DD HH:MM:SS".into());
    }
    if !delivery_date.is_empty() && parse_date_for_storage(&delivery_date).is_err() {
        errors.push("delivery_date must be YYYY-MM-DD or YYYY-MM-DD HH:MM:SS".into());
    }
    if !pickup_date.is_empty()
        && !delivery_date.is_empty()
        && parse_date_for_storage(&pickup_date).ok() > parse_date_for_storage(&delivery_date).ok()
    {
        errors.push("delivery_date must be on or after pickup_date".into());
    }
    if price < 0.0 {
        errors.push("price must be zero or greater".into());
    }
    if weight <= 0.0 {
        errors.push("weight must be greater than zero".into());
    }

    let weight_unit = csv_value(&fields, "weight_unit")
        .unwrap_or_else(|| "LBS".into())
        .to_ascii_uppercase();
    if !matches!(weight_unit.as_str(), "LBS" | "KG" | "MTON") {
        errors.push("weight_unit must be LBS, KG, or MTON".into());
    }

    let bid_status = csv_value(&fields, "bid_status").unwrap_or_else(|| "Fixed".into());
    if !matches!(bid_status.as_str(), "Fixed" | "Open") {
        errors.push("bid_status must be Fixed or Open".into());
    }

    let raw_payload = json!(fields);
    let customer_contract_id = optional_u64("customer_contract_id", &mut errors);
    let customer_contract_lane_id = optional_u64("customer_contract_lane_id", &mut errors);
    let payload = errors.is_empty().then(|| CreateLoadRequest {
        title,
        load_type_id,
        equipment_id,
        commodity_type_id,
        customer_contract_id,
        customer_contract_lane_id,
        freight_mode: csv_value(&fields, "freight_mode"),
        visibility: csv_value(&fields, "visibility"),
        service_level: csv_value(&fields, "service_level"),
        customer_reference: csv_value(&fields, "customer_reference"),
        po_number: csv_value(&fields, "po_number"),
        pickup_appointment_ref: csv_value(&fields, "pickup_appointment_ref"),
        delivery_appointment_ref: csv_value(&fields, "delivery_appointment_ref"),
        facility_contact_name: csv_value(&fields, "facility_contact_name"),
        facility_contact_phone: csv_value(&fields, "facility_contact_phone"),
        facility_contact_email: csv_value(&fields, "facility_contact_email"),
        appointment_window_start: csv_value(&fields, "appointment_window_start"),
        appointment_window_end: csv_value(&fields, "appointment_window_end"),
        accessorial_flags: None,
        weight_unit,
        weight,
        temperature_data: None,
        container_data: None,
        securement_data: None,
        special_instructions: csv_value(&fields, "special_instructions"),
        is_hazardous: csv_bool(&fields, "is_hazardous"),
        is_temperature_controlled: csv_bool(&fields, "is_temperature_controlled"),
        legs: vec![CreateLoadLegRequest {
            pickup_location_id: None,
            pickup_location_address: Some(pickup_address),
            pickup_city: csv_value(&fields, "pickup_city"),
            pickup_country: csv_value(&fields, "pickup_country"),
            pickup_place_id: None,
            pickup_latitude: None,
            pickup_longitude: None,
            delivery_location_id: None,
            delivery_location_address: Some(delivery_address),
            delivery_city: csv_value(&fields, "delivery_city"),
            delivery_country: csv_value(&fields, "delivery_country"),
            delivery_place_id: None,
            delivery_latitude: None,
            delivery_longitude: None,
            pickup_date,
            delivery_date,
            bid_status,
            price,
        }],
    });

    CsvLoadImportRow {
        row_number,
        raw_payload,
        payload,
        errors,
    }
}

fn csv_value(fields: &HashMap<String, String>, key: &str) -> Option<String> {
    fields
        .get(key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn csv_bool(fields: &HashMap<String, String>, key: &str) -> bool {
    fields
        .get(key)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "y"
            )
        })
        .unwrap_or(false)
}

fn normalize_csv_header(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn parse_csv_records(csv: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut chars = csv.chars().peekable();
    let mut quoted = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                row.push(cell.trim().to_string());
                cell.clear();
            }
            '\n' if !quoted => {
                row.push(cell.trim_end_matches('\r').trim().to_string());
                cell.clear();
                records.push(row);
                row = Vec::new();
            }
            _ => cell.push(ch),
        }
    }

    if quoted {
        return Err("CSV contains an unclosed quoted cell.".into());
    }

    if !cell.is_empty() || !row.is_empty() {
        row.push(cell.trim_end_matches('\r').trim().to_string());
        records.push(row);
    }

    Ok(records
        .into_iter()
        .filter(|row| row.iter().any(|cell| !cell.is_empty()))
        .collect())
}

// Import-batch evidence preserves source, counts, and export paths together so
// operators can audit failed customer uploads without chasing side channels.
#[allow(clippy::too_many_arguments)]
async fn insert_load_import_batch(
    pool: &db::DbPool,
    organization_id: i64,
    requested_by_user_id: Option<i64>,
    source_type: &str,
    idempotency_key: Option<&str>,
    original_filename: Option<&str>,
    total_rows: u64,
    valid_rows: u64,
    invalid_rows: u64,
    created_load_count: u64,
    status: &str,
    error_export_csv: Option<&str>,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO load_import_batches (
            organization_id, requested_by_user_id, source_type, idempotency_key,
            original_filename, total_rows, valid_rows, invalid_rows, created_load_count,
            status, error_export_csv, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, source_type, idempotency_key)
         WHERE idempotency_key IS NOT NULL
         DO NOTHING
         RETURNING id",
    )
    .bind(organization_id)
    .bind(requested_by_user_id)
    .bind(source_type)
    .bind(idempotency_key)
    .bind(original_filename)
    .bind(total_rows as i32)
    .bind(valid_rows as i32)
    .bind(invalid_rows as i32)
    .bind(created_load_count as i32)
    .bind(status)
    .bind(error_export_csv)
    .fetch_optional(pool)
    .await
}

async fn insert_load_import_row(
    pool: &db::DbPool,
    batch_id: i64,
    row: &CsvLoadImportRow,
    load_id: Option<i64>,
    idempotency_key: Option<&str>,
) -> Result<(), sqlx::Error> {
    let normalized_payload = row.payload.as_ref().map(|payload| json!(payload));
    sqlx::query(
        "INSERT INTO load_import_rows (
            batch_id, row_number, raw_payload, normalized_payload, validation_status,
            error_messages, load_id, idempotency_key, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CURRENT_TIMESTAMP)",
    )
    .bind(batch_id)
    .bind(row.row_number as i32)
    .bind(&row.raw_payload)
    .bind(&normalized_payload)
    .bind(if load_id.is_some() {
        "created"
    } else if row.errors.is_empty() {
        "valid"
    } else {
        "invalid"
    })
    .bind(&row.errors)
    .bind(load_id)
    .bind(idempotency_key)
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_load_import_row_created(
    pool: &db::DbPool,
    batch_id: i64,
    row_number: u64,
    load_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE load_import_rows
         SET validation_status = 'created', load_id = $3
         WHERE batch_id = $1 AND row_number = $2",
    )
    .bind(batch_id)
    .bind(row_number as i32)
    .bind(load_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_load_import_batch_outcome(
    pool: &db::DbPool,
    batch_id: i64,
    created_load_count: u64,
    invalid_rows: u64,
    status: &str,
    error_export_csv: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE load_import_batches
         SET created_load_count = $2,
             invalid_rows = $3,
             valid_rows = GREATEST(total_rows - $3, 0),
             status = $4,
             error_export_csv = $5,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(batch_id)
    .bind(created_load_count as i32)
    .bind(invalid_rows as i32)
    .bind(status)
    .bind(error_export_csv)
    .execute(pool)
    .await?;
    Ok(())
}

async fn find_api_post_by_idempotency(
    pool: &db::DbPool,
    organization_id: i64,
    idempotency_key: &str,
) -> Result<Option<(i64, Option<String>)>, sqlx::Error> {
    sqlx::query_as::<_, (i64, Option<String>)>(
        "SELECT rows.load_id, loads.load_number
         FROM load_import_batches batches
         INNER JOIN load_import_rows rows ON rows.batch_id = batches.id
         LEFT JOIN loads ON loads.id = rows.load_id
         WHERE batches.organization_id = $1
           AND batches.source_type = 'api_post'
           AND batches.idempotency_key = $2
           AND batches.status = 'committed'
           AND rows.load_id IS NOT NULL
         ORDER BY rows.id DESC
         LIMIT 1",
    )
    .bind(organization_id)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
}

fn load_import_error_csv(rows: &[BulkLoadImportRowResult]) -> String {
    let mut csv = String::from("row_number,title,error\n");
    for row in rows.iter().filter(|row| !row.errors.is_empty()) {
        csv.push_str(&format!(
            "{},{},{}\n",
            row.row_number,
            csv_cell(row.title.as_deref().unwrap_or_default()),
            csv_cell(&row.errors.join("; "))
        ));
    }
    csv
}

fn csv_cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

async fn calculate_and_persist_rate_quote(
    pool: &db::DbPool,
    load: &db::dispatch::LoadRecord,
    payload: &RateCalculationRequest,
    actor_user_id: i64,
) -> Result<RateCalculationResponse, sqlx::Error> {
    let legs = list_load_legs_for_load(pool, load.id).await?;
    let leg_count = legs.len().max(1) as f64;
    let leg_price_total = legs.iter().filter_map(|leg| leg.price).sum::<f64>();
    let base_rate = load.contract_rate.unwrap_or(leg_price_total).max(0.0);
    let currency = load
        .contract_rate_currency
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "USD".into());

    let mileage_rule = sqlx::query_as::<_, (String, f64, f64)>(
        "SELECT source_key, default_miles::double precision, rate_per_mile::double precision
         FROM mileage_calculation_rules
         WHERE organization_id = $1
           AND is_active = TRUE
           AND effective_from <= CURRENT_DATE
           AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
         ORDER BY effective_from DESC, id DESC
         LIMIT 1",
    )
    .bind(load.organization_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_else(|| ("estimated_default".into(), 500.0, 0.0));

    let contract_miles = load
        .contract_service_rules
        .as_ref()
        .and_then(|rules| rules.get("mileage_miles"))
        .and_then(Value::as_f64);
    let mileage_miles = payload
        .mileage_miles_override
        .or(contract_miles)
        .unwrap_or(mileage_rule.1 * leg_count)
        .max(0.0);
    let mileage_source = payload
        .mileage_source_override
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| mileage_rule.0.clone());
    let mileage_charge = (mileage_miles * mileage_rule.2).max(0.0);

    let fuel_rule = sqlx::query_as::<_, (String, f64)>(
        "SELECT surcharge_type, surcharge_value::double precision
         FROM fuel_surcharge_rules
         WHERE organization_id = $1
           AND (customer_contract_id IS NULL OR customer_contract_id = $2)
           AND is_active = TRUE
           AND effective_from <= CURRENT_DATE
           AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
         ORDER BY CASE WHEN customer_contract_id = $2 THEN 0 ELSE 1 END, effective_from DESC, id DESC
         LIMIT 1",
    )
    .bind(load.organization_id)
    .bind(load.customer_contract_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_else(|| ("percentage".into(), 0.0));
    let fuel_surcharge = match fuel_rule.0.as_str() {
        "per_mile" => mileage_miles * fuel_rule.1,
        "flat" => fuel_rule.1,
        _ => base_rate * fuel_rule.1 / 100.0,
    }
    .max(0.0);

    let catalog = sqlx::query_as::<_, (String, String, f64)>(
        "SELECT code, label, default_amount::double precision
         FROM accessorial_catalog
         WHERE organization_id = $1 AND is_active = TRUE
         ORDER BY label",
    )
    .bind(load.organization_id)
    .fetch_all(pool)
    .await?;
    let accessorials = build_accessorial_lines(load.accessorial_flags.as_ref(), catalog);
    let accessorial_total = accessorials.iter().map(|line| line.amount).sum::<f64>();

    let calculated_total = base_rate + mileage_charge + fuel_surcharge + accessorial_total;
    let (total_customer_rate, manual_override_reason) =
        match payload.manual_override_total.filter(|value| *value >= 0.0) {
            Some(total) if (total - calculated_total).abs() > 0.01 => {
                let reason = payload
                    .manual_override_reason
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("Manual rating override")
                    .to_string();
                (total, Some(reason))
            }
            _ => (calculated_total, None),
        };
    let carrier_rate = (total_customer_rate * 0.88).max(0.0);
    let margin = total_customer_rate - carrier_rate;

    let mut explanation = vec![
        format!(
            "Base rate from contract or leg price total: {:.2}.",
            base_rate
        ),
        format!(
            "Mileage source {} produced {:.2} mile(s) at {:.4}/mile.",
            mileage_source, mileage_miles, mileage_rule.2
        ),
        format!(
            "Fuel rule {} applied value {:.4} for surcharge {:.2}.",
            fuel_rule.0, fuel_rule.1, fuel_surcharge
        ),
        format!(
            "{} accessorial line(s) total {:.2}.",
            accessorials.len(),
            accessorial_total
        ),
    ];
    if let Some(reason) = manual_override_reason.as_ref() {
        explanation.push(format!(
            "Manual override changed customer rate from {:.2} to {:.2}: {}.",
            calculated_total, total_customer_rate, reason
        ));
    }

    let quote_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO load_rating_quotes (
            organization_id, load_id, leg_count, base_rate, mileage_miles, mileage_source,
            mileage_charge, fuel_surcharge, accessorial_total, total_customer_rate,
            carrier_rate, margin, currency, explanation, accessorial_breakdown,
            manual_override_total, manual_override_reason, calculated_by_user_id, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                 $11, $12, $13, $14, $15, $16, $17, $18, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(load.organization_id)
    .bind(load.id)
    .bind(legs.len() as i32)
    .bind(base_rate)
    .bind(mileage_miles)
    .bind(&mileage_source)
    .bind(mileage_charge)
    .bind(fuel_surcharge)
    .bind(accessorial_total)
    .bind(total_customer_rate)
    .bind(carrier_rate)
    .bind(margin)
    .bind(&currency)
    .bind(json!(explanation))
    .bind(json!(accessorials))
    .bind(manual_override_reason.as_ref().map(|_| total_customer_rate))
    .bind(&manual_override_reason)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;

    let audit_event_id = if let Some(reason) = manual_override_reason.as_ref() {
        Some(
            sqlx::query_scalar::<_, i64>(
                "INSERT INTO load_rate_override_audit (
                    organization_id, load_id, quote_id, previous_total, new_total, reason,
                    actor_user_id, created_at
                 )
                 VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP)
                 RETURNING id",
            )
            .bind(load.organization_id)
            .bind(load.id)
            .bind(quote_id)
            .bind(calculated_total)
            .bind(total_customer_rate)
            .bind(reason)
            .bind(actor_user_id)
            .fetch_one(pool)
            .await?,
        )
    } else {
        None
    };

    Ok(RateCalculationResponse {
        success: true,
        quote_id: Some(quote_id),
        load_id: load.id,
        leg_count: legs.len() as u64,
        base_rate,
        mileage_miles,
        mileage_source,
        mileage_charge,
        fuel_surcharge,
        accessorial_total,
        total_customer_rate,
        carrier_rate,
        margin,
        currency,
        accessorials,
        explanation,
        audit_event_id,
        message: "Rate calculated and recorded with explainable pricing inputs.".into(),
    })
}

fn build_accessorial_lines(
    accessorial_flags: Option<&Value>,
    catalog: Vec<(String, String, f64)>,
) -> Vec<RateAccessorialLine> {
    let Some(Value::Object(flags)) = accessorial_flags else {
        return Vec::new();
    };

    catalog
        .into_iter()
        .filter_map(|(code, label, default_amount)| {
            let flag = flags.get(&code)?;
            let amount = match flag {
                Value::Bool(true) => default_amount,
                Value::Number(number) => number.as_f64().unwrap_or(default_amount),
                Value::String(value) => value.parse::<f64>().unwrap_or(default_amount),
                Value::Object(object) => object
                    .get("amount")
                    .and_then(Value::as_f64)
                    .unwrap_or(default_amount),
                _ => 0.0,
            };
            (amount > 0.0).then_some(RateAccessorialLine {
                code,
                label,
                amount,
            })
        })
        .collect()
}

fn empty_rate_response(load_id: i64, message: &str) -> RateCalculationResponse {
    RateCalculationResponse {
        success: false,
        quote_id: None,
        load_id,
        leg_count: 0,
        base_rate: 0.0,
        mileage_miles: 0.0,
        mileage_source: "unavailable".into(),
        mileage_charge: 0.0,
        fuel_surcharge: 0.0,
        accessorial_total: 0.0,
        total_customer_rate: 0.0,
        carrier_rate: 0.0,
        margin: 0.0,
        currency: "USD".into(),
        accessorials: vec![],
        explanation: vec![],
        audit_event_id: None,
        message: message.into(),
    }
}

fn validate_coordinates(latitude: Option<f64>, longitude: Option<f64>) -> Result<(), String> {
    if let Some(latitude) = latitude
        && !(-90.0..=90.0).contains(&latitude)
    {
        return Err("Latitude must be between -90 and 90.".into());
    }
    if let Some(longitude) = longitude
        && !(-180.0..=180.0).contains(&longitude)
    {
        return Err("Longitude must be between -180 and 180.".into());
    }
    if latitude.is_some() != longitude.is_some() {
        return Err("Latitude and longitude must be provided together.".into());
    }
    Ok(())
}

fn normalize_time_zone(value: Option<&String>) -> String {
    value
        .map(|value| value.trim())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 80
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '+'))
        })
        .unwrap_or("UTC")
        .to_string()
}

// Geocode persistence receives one normalized facility payload from location
// resolution; keeping the fields explicit avoids hiding user-entered evidence.
#[allow(clippy::too_many_arguments)]
async fn persist_location_geocode_and_facility(
    pool: &db::DbPool,
    location_id: i64,
    organization_id: i64,
    normalized_address: &str,
    place_id: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    facility_type: &str,
) -> Result<i64, sqlx::Error> {
    let validation_status = if latitude.is_some() && longitude.is_some() {
        "geocoded"
    } else {
        "manual"
    };
    sqlx::query(
        "UPDATE locations
         SET normalized_address = $2,
             google_place_id = COALESCE($3, google_place_id),
             latitude = COALESCE($4, latitude),
             longitude = COALESCE($5, longitude),
             validation_status = $6,
             facility_type = $7,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(location_id)
    .bind(normalized_address)
    .bind(place_id.map(str::trim).filter(|value| !value.is_empty()))
    .bind(latitude)
    .bind(longitude)
    .bind(validation_status)
    .bind(facility_type)
    .execute(pool)
    .await?;

    sqlx::query_scalar::<_, i64>(
        "INSERT INTO facilities (
            organization_id, location_id, facility_name, facility_type, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (organization_id, location_id)
         DO UPDATE SET
            facility_name = EXCLUDED.facility_name,
            facility_type = EXCLUDED.facility_type,
            updated_at = CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(organization_id)
    .bind(location_id)
    .bind(normalized_address)
    .bind(facility_type)
    .fetch_one(pool)
    .await
}

async fn schedule_or_reschedule_facility_appointment(
    pool: &db::DbPool,
    load: &db::dispatch::LoadRecord,
    payload: &FacilityAppointmentRequest,
    actor_user_id: i64,
) -> Result<FacilityAppointmentResponse, sqlx::Error> {
    let leg_id = payload.leg_id as i64;
    let Some(leg) = find_load_leg_by_id(pool, leg_id).await? else {
        return Ok(empty_facility_appointment_response(
            load.id,
            Some(leg_id),
            "The selected load leg was not found.",
        ));
    };
    if leg.load_id != load.id {
        return Ok(empty_facility_appointment_response(
            load.id,
            Some(leg_id),
            "The selected load leg does not belong to this load.",
        ));
    }

    let stop_type = payload.stop_type.trim().to_ascii_lowercase();
    let location_id = match stop_type.as_str() {
        "pickup" => leg.pickup_location_id,
        "delivery" | "dropoff" => leg.delivery_location_id,
        _ => {
            return Ok(empty_facility_appointment_response(
                load.id,
                Some(leg_id),
                "Appointment stop_type must be pickup or delivery.",
            ));
        }
    };
    let appointment_start = match parse_optional_datetime_for_storage(
        "Appointment start",
        Some(&payload.appointment_start),
    ) {
        Ok(Some(value)) => value,
        Ok(None) => {
            return Ok(empty_facility_appointment_response(
                load.id,
                Some(leg_id),
                "Appointment start is required.",
            ));
        }
        Err(message) => {
            return Ok(empty_facility_appointment_response(
                load.id,
                Some(leg_id),
                &format!("Appointment start error: {}", message),
            ));
        }
    };
    let appointment_end = match parse_optional_datetime_for_storage(
        "Appointment end",
        payload.appointment_end.as_ref(),
    ) {
        Ok(value) => value,
        Err(message) => {
            return Ok(empty_facility_appointment_response(
                load.id,
                Some(leg_id),
                &format!("Appointment end error: {}", message),
            ));
        }
    };
    if appointment_end.is_some_and(|end| end < appointment_start) {
        return Ok(empty_facility_appointment_response(
            load.id,
            Some(leg_id),
            "Appointment end must be after the start.",
        ));
    }
    let appointment_time_zone = normalize_time_zone(payload.appointment_time_zone.as_ref());

    let facility_id = ensure_facility_for_location(
        pool,
        load.organization_id,
        location_id,
        &format!(
            "{} facility for load #{}",
            profile_title_case(&stop_type),
            load.id
        ),
        &stop_type,
    )
    .await?;
    let previous = sqlx::query_as::<_, (i64, Option<NaiveDateTime>, Option<NaiveDateTime>)>(
        "SELECT id, appointment_start, appointment_end
         FROM facility_appointments
         WHERE load_id = $1 AND leg_id = $2 AND stop_type = $3
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(load.id)
    .bind(leg_id)
    .bind(&stop_type)
    .fetch_optional(pool)
    .await?;

    let (appointment_id, event_type, previous_start, previous_end) = if let Some(previous) =
        previous
    {
        let appointment_id = sqlx::query_scalar::<_, i64>(
            "UPDATE facility_appointments
             SET facility_id = $2,
                 appointment_ref = $3,
                 appointment_start = $4,
                 appointment_end = $5,
                 status = 'rescheduled',
                 dock_name = $6,
                 contact_name = $7,
                 contact_phone = $8,
                 contact_email = $9,
                 notes = $10,
                 scheduled_by_user_id = $11,
                 time_zone = $12,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id",
        )
        .bind(previous.0)
        .bind(facility_id)
        .bind(normalize_optional_text(payload.appointment_ref.as_ref()))
        .bind(appointment_start)
        .bind(appointment_end)
        .bind(normalize_optional_text(payload.dock_name.as_ref()))
        .bind(normalize_optional_text(payload.contact_name.as_ref()))
        .bind(normalize_optional_text(payload.contact_phone.as_ref()))
        .bind(normalize_optional_text(payload.contact_email.as_ref()))
        .bind(normalize_optional_text(payload.notes.as_ref()))
        .bind(actor_user_id)
        .bind(&appointment_time_zone)
        .fetch_one(pool)
        .await?;
        (appointment_id, "rescheduled", previous.1, previous.2)
    } else {
        let appointment_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO facility_appointments (
                organization_id, load_id, leg_id, facility_id, stop_type, appointment_ref,
                appointment_start, appointment_end, status, dock_name, contact_name,
                contact_phone, contact_email, notes, scheduled_by_user_id, time_zone,
                created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'scheduled', $9, $10, $11, $12, $13, $14, $15, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .bind(load.organization_id)
        .bind(load.id)
        .bind(leg_id)
        .bind(facility_id)
        .bind(&stop_type)
        .bind(normalize_optional_text(payload.appointment_ref.as_ref()))
        .bind(appointment_start)
        .bind(appointment_end)
        .bind(normalize_optional_text(payload.dock_name.as_ref()))
        .bind(normalize_optional_text(payload.contact_name.as_ref()))
        .bind(normalize_optional_text(payload.contact_phone.as_ref()))
        .bind(normalize_optional_text(payload.contact_email.as_ref()))
        .bind(normalize_optional_text(payload.notes.as_ref()))
        .bind(actor_user_id)
        .bind(&appointment_time_zone)
        .fetch_one(pool)
        .await?;
        (appointment_id, "scheduled", None, None)
    };

    let event_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO facility_appointment_events (
            appointment_id, event_type, previous_start, previous_end, new_start, new_end,
            reason, actor_user_id, time_zone, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(appointment_id)
    .bind(event_type)
    .bind(previous_start)
    .bind(previous_end)
    .bind(appointment_start)
    .bind(appointment_end)
    .bind(normalize_optional_text(payload.reason.as_ref()))
    .bind(actor_user_id)
    .bind(&appointment_time_zone)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load.id)
    .bind(actor_user_id)
    .bind(load.status)
    .bind(format!(
        "Facility appointment {} for leg #{} {} stop at {}.",
        event_type,
        leg_id,
        stop_type,
        format_profile_datetime(&appointment_start)
    ))
    .execute(pool)
    .await?;

    Ok(FacilityAppointmentResponse {
        success: true,
        appointment_id: Some(appointment_id),
        event_id: Some(event_id),
        load_id: load.id,
        leg_id: Some(leg_id),
        status: Some(event_type.into()),
        message: format!(
            "Facility appointment {} for {} leg #{}.",
            event_type, stop_type, leg_id
        ),
    })
}

async fn ensure_facility_for_location(
    pool: &db::DbPool,
    organization_id: i64,
    location_id: i64,
    fallback_name: &str,
    facility_type: &str,
) -> Result<i64, sqlx::Error> {
    let name = sqlx::query_scalar::<_, String>("SELECT name FROM locations WHERE id = $1")
        .bind(location_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or_else(|| fallback_name.into());
    persist_location_geocode_and_facility(
        pool,
        location_id,
        organization_id,
        &name,
        None,
        None,
        None,
        facility_type,
    )
    .await
}

fn empty_facility_appointment_response(
    load_id: i64,
    leg_id: Option<i64>,
    message: &str,
) -> FacilityAppointmentResponse {
    FacilityAppointmentResponse {
        success: false,
        appointment_id: None,
        event_id: None,
        load_id,
        leg_id,
        status: None,
        message: message.into(),
    }
}

#[derive(Debug)]
struct LatestRatingSummary {
    mileage_miles: f64,
    mileage_source: String,
    fuel_surcharge: f64,
    accessorial_total: f64,
    total_customer_rate: f64,
    carrier_rate: f64,
    margin: f64,
    currency: String,
}

async fn latest_load_rating_summary(
    pool: &db::DbPool,
    load_id: i64,
) -> Result<Option<LatestRatingSummary>, sqlx::Error> {
    sqlx::query_as::<_, (f64, String, f64, f64, f64, f64, f64, String)>(
        "SELECT
            mileage_miles::double precision,
            mileage_source,
            fuel_surcharge::double precision,
            accessorial_total::double precision,
            total_customer_rate::double precision,
            carrier_rate::double precision,
            margin::double precision,
            currency
         FROM load_rating_quotes
         WHERE load_id = $1
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(
                mileage_miles,
                mileage_source,
                fuel_surcharge,
                accessorial_total,
                total_customer_rate,
                carrier_rate,
                margin,
                currency,
            )| LatestRatingSummary {
                mileage_miles,
                mileage_source,
                fuel_surcharge,
                accessorial_total,
                total_customer_rate,
                carrier_rate,
                margin,
                currency,
            },
        )
    })
}

#[derive(Debug)]
struct LatestLocalizationSummary {
    locale: String,
    time_zone: String,
    display_distance_unit: String,
    display_weight_unit: String,
    canonical_weight_lbs: Option<f64>,
    display_weight: Option<f64>,
    dimension_unit: String,
    temperature_unit: String,
    currency: String,
}

async fn latest_load_localization_summary(
    pool: &db::DbPool,
    load_id: i64,
) -> Result<Option<LatestLocalizationSummary>, sqlx::Error> {
    sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            Option<f64>,
            Option<f64>,
            String,
            String,
            String,
        ),
    >(
        "SELECT
            locale,
            time_zone,
            display_distance_unit,
            display_weight_unit,
            canonical_weight_lbs::double precision,
            display_weight::double precision,
            dimension_unit,
            temperature_unit,
            currency
         FROM load_localization_snapshots
         WHERE load_id = $1
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(load_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(
            |(
                locale,
                time_zone,
                display_distance_unit,
                display_weight_unit,
                canonical_weight_lbs,
                display_weight,
                dimension_unit,
                temperature_unit,
                currency,
            )| LatestLocalizationSummary {
                locale,
                time_zone,
                display_distance_unit,
                display_weight_unit,
                canonical_weight_lbs,
                display_weight,
                dimension_unit,
                temperature_unit,
                currency,
            },
        )
    })
}

fn validate_load_document_payload(
    payload: &UpsertLoadDocumentRequest,
) -> Result<UpsertLoadDocumentParams, String> {
    let document_name = payload.document_name.trim().to_string();
    if document_name.is_empty() {
        return Err("Enter a document name before saving the load profile document row.".into());
    }

    let document_type = payload
        .document_type
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_");
    if document_type.is_empty() {
        return Err("Choose a document type before saving the load profile document row.".into());
    }

    let file_path = payload.file_path.trim().to_string();
    if file_path.is_empty() {
        return Err("Enter a storage path or URL before saving the document row.".into());
    }

    if payload.file_size.unwrap_or(0) < 0 {
        return Err("File size cannot be negative.".into());
    }

    Ok(UpsertLoadDocumentParams {
        document_name,
        document_type,
        file_path: file_path.clone(),
        storage_provider: normalize_storage_provider(&file_path),
        original_name: payload
            .original_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        mime_type: payload
            .mime_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        file_size: payload.file_size,
    })
}

fn normalize_storage_provider(file_path: &str) -> String {
    let normalized = file_path.trim().to_ascii_lowercase();
    if normalized.starts_with("ibm-cos://") || normalized.starts_with("s3://") {
        "ibm_cos".into()
    } else if normalized.starts_with("http://") || normalized.starts_with("https://") {
        "external_url".into()
    } else {
        "local".into()
    }
}

async fn parse_document_upload(mut multipart: Multipart) -> Result<ParsedUploadedDocument, String> {
    let mut document_name = None::<String>;
    let mut document_type = None::<String>;
    let mut original_name = None::<String>;
    let mut mime_type = None::<String>;
    let mut bytes = None::<Vec<u8>>;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| format!("Document upload parsing failed: {}", error))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        match field_name.as_str() {
            "document_name" => {
                let value = field
                    .text()
                    .await
                    .map_err(|error| format!("Document name parsing failed: {}", error))?;
                document_name = Some(value.trim().to_string());
            }
            "document_type" => {
                let value = field
                    .text()
                    .await
                    .map_err(|error| format!("Document type parsing failed: {}", error))?;
                document_type = Some(value.trim().to_ascii_lowercase().replace([' ', '-'], "_"));
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
        .ok_or_else(|| "Enter a document name before uploading a file.".to_string())?;
    let document_type = document_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Choose a document type before uploading a file.".to_string())?;
    let bytes = bytes.ok_or_else(|| "Choose a file before uploading a document.".to_string())?;
    let original_name = original_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "document.bin".into());
    let verdict = validate_uploaded_document(&original_name, mime_type.as_deref(), &bytes)?;

    Ok(ParsedUploadedDocument {
        document_name,
        document_type,
        original_name,
        mime_type: verdict.normalized_mime_type,
        bytes,
    })
}

fn text_response(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

fn can_view_load_document_file(
    viewer: &crate::auth_session::ResolvedSession,
    uploaded_by_user_id: Option<i64>,
) -> bool {
    if viewer.user.primary_role() == Some(UserRole::Admin) {
        return true;
    }

    uploaded_by_user_id == Some(viewer.user.id)
}

fn document_download_path(document_id: u64) -> String {
    format!("/dispatch/documents/{}/file", document_id)
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

async fn build_load_profile_screen(
    state: &AppState,
    viewer: Option<&crate::auth_session::ResolvedSession>,
    load_id: i64,
) -> LoadProfileScreen {
    let Some(viewer) = viewer else {
        return empty_load_profile_screen(
            state,
            load_id,
            vec![
                "Sign in before opening a Rust load profile.".into(),
                "This route is auth-scoped and intentionally avoids falling back to Laravel load views during staged cutover.".into(),
            ],
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return empty_load_profile_screen(
            state,
            load_id,
            vec![format!(
                "Load profile data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        );
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return empty_load_profile_screen(
            state,
            load_id,
            vec![format!(
                "Load #{} was not found in the Rust dispatch store.",
                load_id
            )],
        );
    };

    if !can_view_load_profile(viewer, &load) {
        return empty_load_profile_screen(
            state,
            load_id,
            vec![
                "The authenticated session cannot open this load profile in the current Rust slice.".into(),
                "Owner and admin visibility is enabled first so document and history parity can land safely.".into(),
            ],
        );
    }

    let can_manage_documents =
        can_manage_load_documents(viewer, load.user_id, load.organization_id);

    let load_types = list_load_types(pool).await.unwrap_or_default();
    let equipments = list_equipments(pool).await.unwrap_or_default();
    let commodity_types = list_commodity_types(pool).await.unwrap_or_default();
    let load_type_map = load_types
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect::<HashMap<_, _>>();
    let equipment_map = equipments
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect::<HashMap<_, _>>();
    let commodity_type_map = commodity_types
        .into_iter()
        .map(|row| (row.id, row.name))
        .collect::<HashMap<_, _>>();

    let legs = list_load_profile_legs_for_load(pool, load.id)
        .await
        .unwrap_or_default();
    let documents = list_load_documents_for_load(pool, load.id)
        .await
        .unwrap_or_default();
    let history = list_load_history_for_load(pool, load.id)
        .await
        .unwrap_or_default();
    let handoff = find_latest_handoff_for_load(pool, load.id)
        .await
        .unwrap_or_default();
    let latest_rating = latest_load_rating_summary(pool, load.id)
        .await
        .unwrap_or_default();
    let latest_localization = latest_load_localization_summary(pool, load.id)
        .await
        .unwrap_or_default();
    let can_manage_lifecycle = can_manage_existing_load(viewer, load.user_id);
    let lifecycle_actions = build_load_lifecycle_actions(&load, &legs, can_manage_lifecycle);

    let mut info_fields = vec![
        LoadProfileField {
            label: "Title".into(),
            value: load.title.clone(),
        },
        LoadProfileField {
            label: "Load Number".into(),
            value: load
                .load_number
                .clone()
                .unwrap_or_else(|| format!("Load #{}", load.id)),
        },
        LoadProfileField {
            label: "Lifecycle".into(),
            value: format!(
                "{} v{}",
                profile_title_case(&load.lifecycle_status),
                load.revision_number
            ),
        },
        LoadProfileField {
            label: "Contract Lane".into(),
            value: match (
                load.customer_contract_id,
                load.customer_contract_lane_id,
                load.contract_rate,
                load.contract_rate_currency.clone(),
            ) {
                (Some(contract_id), Some(lane_id), Some(rate), Some(currency)) => {
                    format!(
                        "Contract #{} lane #{} at {} {:.2}",
                        contract_id, lane_id, currency, rate
                    )
                }
                (Some(contract_id), Some(lane_id), _, _) => {
                    format!("Contract #{} lane #{}", contract_id, lane_id)
                }
                _ => "Spot or manually priced freight".into(),
            },
        },
        LoadProfileField {
            label: "Load Type".into(),
            value: load
                .load_type_id
                .and_then(|value| load_type_map.get(&value).cloned())
                .unwrap_or_else(|| "Not assigned".into()),
        },
        LoadProfileField {
            label: "Equipment".into(),
            value: load
                .equipment_id
                .and_then(|value| equipment_map.get(&value).cloned())
                .unwrap_or_else(|| "Not assigned".into()),
        },
        LoadProfileField {
            label: "Commodity Type".into(),
            value: load
                .commodity_type_id
                .and_then(|value| commodity_type_map.get(&value).cloned())
                .unwrap_or_else(|| "Not assigned".into()),
        },
        LoadProfileField {
            label: "Freight Mode".into(),
            value: load.freight_mode.clone(),
        },
        LoadProfileField {
            label: "Visibility".into(),
            value: profile_title_case(&load.visibility),
        },
        LoadProfileField {
            label: "Service Level".into(),
            value: format_profile_optional_text(load.service_level.as_ref(), "Not recorded"),
        },
        LoadProfileField {
            label: "Customer Reference".into(),
            value: format_profile_optional_text(load.customer_reference.as_ref(), "Not recorded"),
        },
        LoadProfileField {
            label: "PO Number".into(),
            value: format_profile_optional_text(load.po_number.as_ref(), "Not recorded"),
        },
        LoadProfileField {
            label: "Pickup Appointment".into(),
            value: format_profile_optional_text(
                load.pickup_appointment_ref.as_ref(),
                "Not recorded",
            ),
        },
        LoadProfileField {
            label: "Delivery Appointment".into(),
            value: format_profile_optional_text(
                load.delivery_appointment_ref.as_ref(),
                "Not recorded",
            ),
        },
        LoadProfileField {
            label: "Facility Contact".into(),
            value: {
                let label = [
                    load.facility_contact_name.as_deref(),
                    load.facility_contact_phone.as_deref(),
                    load.facility_contact_email.as_deref(),
                ]
                .into_iter()
                .flatten()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ");
                if label.is_empty() {
                    "Not recorded".into()
                } else {
                    label
                }
            },
        },
        LoadProfileField {
            label: "Appointment Window".into(),
            value: match (
                load.appointment_window_start.as_ref(),
                load.appointment_window_end.as_ref(),
            ) {
                (Some(start), Some(end)) => {
                    format!(
                        "{} to {}",
                        format_profile_datetime(start),
                        format_profile_datetime(end)
                    )
                }
                (Some(start), None) => format!("Starts {}", format_profile_datetime(start)),
                (None, Some(end)) => format!("Ends {}", format_profile_datetime(end)),
                _ => "Not recorded".into(),
            },
        },
        LoadProfileField {
            label: "Accessorials".into(),
            value: format_profile_json(load.accessorial_flags.as_ref()),
        },
        LoadProfileField {
            label: "Weight".into(),
            value: match (load.weight, load.weight_unit.clone()) {
                (Some(weight), Some(unit)) => format!("{:.2} {}", weight, unit),
                (Some(weight), None) => format!("{:.2}", weight),
                _ => "Not set".into(),
            },
        },
        LoadProfileField {
            label: "Hazardous".into(),
            value: if load.is_hazardous { "Yes" } else { "No" }.into(),
        },
        LoadProfileField {
            label: "Temperature Controlled".into(),
            value: if load.is_temperature_controlled {
                "Yes"
            } else {
                "No"
            }
            .into(),
        },
        LoadProfileField {
            label: "Temperature Details".into(),
            value: format_profile_json(load.temperature_data.as_ref()),
        },
        LoadProfileField {
            label: "Container Details".into(),
            value: format_profile_json(load.container_data.as_ref()),
        },
        LoadProfileField {
            label: "Securement Details".into(),
            value: format_profile_json(load.securement_data.as_ref()),
        },
        LoadProfileField {
            label: "Leg Count".into(),
            value: load.leg_count.to_string(),
        },
        LoadProfileField {
            label: "Special Instructions".into(),
            value: load
                .special_instructions
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "No special instructions recorded.".into()),
        },
    ];

    if let Some(rating) = latest_rating {
        info_fields.push(LoadProfileField {
            label: "Rating Summary".into(),
            value: format!(
                "{} {:.2} customer | {:.2} carrier | {:.2} margin",
                rating.currency, rating.total_customer_rate, rating.carrier_rate, rating.margin
            ),
        });
        info_fields.push(LoadProfileField {
            label: "Mileage And Fuel".into(),
            value: format!(
                "{:.2} miles via {} | fuel {:.2} | accessorials {:.2}",
                rating.mileage_miles,
                profile_title_case(&rating.mileage_source),
                rating.fuel_surcharge,
                rating.accessorial_total
            ),
        });
    }
    if let Some(localization) = latest_localization {
        info_fields.push(LoadProfileField {
            label: "Localization".into(),
            value: format!(
                "{} | {} | distance {} | weight {} | dimensions {} | temp {} | currency {}",
                localization.locale,
                localization.time_zone,
                localization.display_distance_unit,
                localization.display_weight_unit,
                localization.dimension_unit,
                localization.temperature_unit,
                localization.currency
            ),
        });
        info_fields.push(LoadProfileField {
            label: "Canonical Weight".into(),
            value: match (
                localization.canonical_weight_lbs,
                localization.display_weight,
            ) {
                (Some(canonical), Some(display)) => format!(
                    "{:.2} LBS canonical | {:.2} {} display",
                    canonical, display, localization.display_weight_unit
                ),
                _ => "Not available".into(),
            },
        });
    }

    let leg_rows =
        legs.into_iter()
            .map(|leg| {
                let payment_label = profile_payment_label(leg.escrow_status.as_deref());
                let (finance_action_key, finance_action_label, finance_action_enabled) =
                    profile_finance_action(
                        leg.status_id,
                        leg.escrow_status.as_deref(),
                        leg.booked_carrier_name.is_some(),
                    );
                let payments_action = finance_action_key
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "release".into());

                LoadProfileLegRow {
                leg_id: leg.id.max(0) as u64,
                status_code: leg.status_id,
                leg_code: leg.leg_code.unwrap_or_else(|| format!("LEG-{}", leg.leg_no)),
                route_label: format!(
                    "{} -> {}",
                    leg.pickup_location_name
                        .unwrap_or_else(|| "Unknown pickup".into()),
                    leg.delivery_location_name
                        .unwrap_or_else(|| "Unknown delivery".into())
                ),
                pickup_date_label: format_profile_date(leg.pickup_date.as_ref()),
                delivery_date_label: format_profile_date(leg.delivery_date.as_ref()),
                status_label: profile_load_leg_status_label(leg.status_id),
                status_tone: profile_load_leg_status_tone(leg.status_id).into(),
                bid_status_label: leg
                    .bid_status
                    .as_deref()
                    .map(profile_title_case)
                    .unwrap_or_else(|| "Open".into()),
                amount_label: format_profile_currency(leg.booked_amount.or(leg.price)),
                carrier_label: leg.booked_carrier_name,
                payment_label,
                finance_action_key,
                finance_action_label,
                finance_action_enabled,
                payments_href: (leg.status_id == 10 || leg.escrow_id.is_some()).then(|| {
                    format!(
                        "/admin/payments?leg_id={}&action={}&source=admin-load-profile&load_id={}",
                        leg.id.max(0) as u64,
                        payments_action,
                        load.id.max(0) as u64
                    )
                }),
            }
            })
            .collect::<Vec<_>>();

    let document_rows = documents
        .into_iter()
        .map(|document| {
            let has_hash = document.hash.as_deref().is_some();
            let blockchain_label = if has_hash {
                Some(match document.mock_blockchain_tx.as_deref() {
                    Some(tx_id) if !tx_id.trim().is_empty() => {
                        format!("Timestamped hash: {}", tx_id)
                    }
                    _ => "SHA-256 hash stored".into(),
                })
            } else if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("Pending content hash verification".into())
            } else {
                None
            };
            let blockchain_tone = if has_hash {
                Some("success".into())
            } else if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("warning".into())
            } else {
                None
            };
            let blockchain_hash_preview = document.hash.as_ref().map(|hash| {
                if hash.len() > 16 {
                    format!("{}...", &hash[..16])
                } else {
                    hash.clone()
                }
            });
            let can_view_file = can_view_load_document_file(viewer, document.uploaded_by_user_id);
            let uploaded_by_label = document.uploaded_by_user_id.map(|user_id| {
                if user_id == viewer.user.id {
                    "Uploaded by you".to_string()
                } else {
                    format!("Uploaded by user #{}", user_id)
                }
            });

            LoadDocumentRow {
                id: document.id.max(0) as u64,
                document_name: document.document_name,
                document_type_key: document.document_type.clone(),
                document_type_label: profile_title_case(&document.document_type),
                file_label: document
                    .original_name
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| profile_file_label(&document.file_path)),
                source_path: document.file_path.clone(),
                download_path: can_view_file
                    .then(|| document_download_path(document.id.max(0) as u64)),
                original_name: document.original_name,
                mime_type: document.mime_type,
                file_size_bytes: document
                    .file_size
                    .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
                uploaded_by_label,
                can_view_file,
                blockchain_label,
                blockchain_tone,
                blockchain_hash_preview,
                can_edit: can_manage_documents,
                can_verify_blockchain: can_manage_documents && !has_hash,
                uploaded_at_label: format_profile_datetime(&document.created_at),
                current_version: document.current_version.max(1) as u32,
                version_count: document.version_count.max(1) as u64,
                version_history_label: document_version_label(
                    document.current_version,
                    document.version_count,
                ),
            }
        })
        .collect::<Vec<_>>();

    let history_rows = history
        .into_iter()
        .map(|entry| LoadHistoryRow {
            id: entry.id.max(0) as u64,
            status_label: profile_load_leg_status_label(entry.status),
            status_tone: profile_load_leg_status_tone(entry.status).into(),
            remarks_label: entry
                .remarks
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "No remarks recorded.".into()),
            actor_label: entry
                .admin_id
                .map(|value| format!("User #{}", value))
                .unwrap_or_else(|| "System".into()),
            created_at_label: format_profile_datetime(&entry.created_at),
        })
        .collect::<Vec<_>>();

    let stloads_handoff = handoff.map(|handoff| LoadHandoffSummary {
        handoff_id: handoff.id.max(0) as u64,
        status_label: profile_title_case(&handoff.status),
        status_tone: profile_handoff_status_tone(&handoff.status).into(),
        tms_load_id: handoff.tms_load_id,
        board_rate_label: format!(
            "{} {}",
            handoff.rate_currency,
            format_profile_currency(handoff.board_rate)
        ),
        tms_status_label: handoff.tms_status.as_deref().map(profile_title_case),
        tms_status_at_label: handoff.tms_status_at.as_ref().map(format_profile_datetime),
        published_at_label: handoff.published_at.as_ref().map(format_profile_datetime),
        pushed_by_label: handoff.pushed_by.filter(|value| !value.trim().is_empty()),
    });

    let mut notes = vec![
        "The Rust load profile now supports binary document upload, restricted file viewing, metadata edits, and SHA-256 content hash controls alongside load details, history, and STLOADS context.".into(),
        "Uploaded files are only viewable by admin users and the profile that uploaded the file in the current Rust slice.".into(),
        format!(
            "Storage backend: {}. The abstraction is ready for IBM object storage, while this staged slice can still run on local-backed storage during development.",
            state.document_storage.backend()
        ),
    ];

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so load profile links remain proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    LoadProfileScreen {
        title: "Load Profile".into(),
        subtitle: "Rust detail view for created loads, leg lifecycle, documents, and STLOADS board context.".into(),
        load_id: load.id.max(0) as u64,
        load_number: load.load_number,
        can_manage_documents,
        can_manage_lifecycle,
        lifecycle_status: load.lifecycle_status,
        revision_number: load.revision_number,
        lifecycle_actions,
        info_fields,
        legs: leg_rows,
        required_documents: load_required_document_checklist(&document_rows),
        documents: document_rows,
        history: history_rows,
        stloads_handoff,
        notes,
    }
}

fn empty_load_profile_screen(
    state: &AppState,
    load_id: i64,
    mut notes: Vec<String>,
) -> LoadProfileScreen {
    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so load profile links remain proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    LoadProfileScreen {
        title: "Load Profile".into(),
        subtitle: "Secure Rust load detail view".into(),
        load_id: load_id.max(0) as u64,
        load_number: None,
        can_manage_documents: false,
        can_manage_lifecycle: false,
        lifecycle_status: "missing".into(),
        revision_number: 0,
        lifecycle_actions: Vec::new(),
        info_fields: Vec::new(),
        legs: Vec::new(),
        required_documents: Vec::new(),
        documents: Vec::new(),
        history: Vec::new(),
        stloads_handoff: None,
        notes,
    }
}

fn can_manage_load_documents(
    viewer: &crate::auth_session::ResolvedSession,
    load_owner_user_id: Option<i64>,
    organization_id: i64,
) -> bool {
    if !crate::auth_session::session_matches_organization(viewer, organization_id) {
        return false;
    }

    if viewer.user.primary_role() == Some(UserRole::Admin) {
        return true;
    }

    if viewer.session.permissions.iter().any(|permission| {
        permission == "access_admin_portal" || permission == "manage_dispatch_desk"
    }) {
        return true;
    }

    load_owner_user_id == Some(viewer.user.id)
        && viewer
            .session
            .permissions
            .iter()
            .any(|permission| permission == "manage_loads")
}

fn can_view_load_profile(
    viewer: &crate::auth_session::ResolvedSession,
    load: &db::dispatch::LoadRecord,
) -> bool {
    can_manage_load_documents(viewer, load.user_id, load.organization_id)
}

fn profile_load_leg_status_label(status_id: i16) -> String {
    match domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(domain::dispatch::LegacyLoadLegStatusCode::Draft) => "Draft".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::New) => "New".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::Reviewed) => "Reviewed".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::OfferReady) => "Offer Ready".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::Booked) => "Booked".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::PickupStarted) => "Pickup Started".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::AtPickup) => "At Pickup".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::InTransit) => "In Transit".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded) => "Escrow Funded".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::AtDelivery) => "At Delivery".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered) => "Delivered".into(),
        Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut) => "Paid Out".into(),
        None => format!("Status {}", status_id),
    }
}

fn profile_load_leg_status_tone(status_id: i16) -> &'static str {
    match domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(domain::dispatch::LegacyLoadLegStatusCode::Draft) => "secondary",
        Some(domain::dispatch::LegacyLoadLegStatusCode::New) => "warning",
        Some(domain::dispatch::LegacyLoadLegStatusCode::Reviewed) => "info",
        Some(domain::dispatch::LegacyLoadLegStatusCode::OfferReady) => "primary",
        Some(domain::dispatch::LegacyLoadLegStatusCode::Booked) => "primary",
        Some(domain::dispatch::LegacyLoadLegStatusCode::PickupStarted)
        | Some(domain::dispatch::LegacyLoadLegStatusCode::AtPickup)
        | Some(domain::dispatch::LegacyLoadLegStatusCode::InTransit)
        | Some(domain::dispatch::LegacyLoadLegStatusCode::AtDelivery) => "info",
        Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded)
        | Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered) => "success",
        Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut) => "dark",
        None => "secondary",
    }
}

fn profile_payment_label(escrow_status: Option<&str>) -> Option<String> {
    escrow_status.map(|status| match status {
        "released" | "paid_out" => "Released".to_string(),
        "funded" => "Funded".to_string(),
        "pending" | "hold" => "Pending".to_string(),
        "unfunded" => "Unfunded".to_string(),
        other => profile_title_case(other),
    })
}

fn profile_finance_action(
    status_id: i16,
    escrow_status: Option<&str>,
    has_carrier: bool,
) -> (Option<String>, Option<String>, bool) {
    if !has_carrier {
        return (None, Some("Assign a carrier first".into()), false);
    }

    match escrow_status {
        Some("released" | "paid_out") => (None, Some("Funds already released".into()), false),
        Some("hold") => (Some("release".into()), Some("Release funds".into()), true),
        Some("funded") => (Some("release".into()), Some("Release funds".into()), true),
        Some("pending") => (Some("hold".into()), Some("Place on hold".into()), true),
        Some("unfunded") | None => {
            if matches!(
                domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id),
                Some(domain::dispatch::LegacyLoadLegStatusCode::Booked)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::PickupStarted)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::AtPickup)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::InTransit)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::AtDelivery)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered)
                    | Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut)
            ) {
                (Some("fund".into()), Some("Fund escrow".into()), true)
            } else {
                (None, Some("Finance opens after booking".into()), false)
            }
        }
        Some(other) => (None, Some(profile_title_case(other)), false),
    }
}

fn profile_handoff_status_tone(status: &str) -> &'static str {
    match status {
        "queued" => "warning",
        "push_in_progress" => "info",
        "published" => "success",
        "push_failed" => "danger",
        "requeue_required" => "primary",
        "withdrawn" => "secondary",
        "closed" => "dark",
        _ => "secondary",
    }
}

fn format_profile_date(value: Option<&chrono::NaiveDateTime>) -> String {
    value
        .map(|date| date.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| "TBD".into())
}

fn format_profile_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
}

fn format_profile_optional_text(value: Option<&String>, fallback: &str) -> String {
    value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn format_profile_json(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => text.trim().to_string(),
        Some(Value::Array(items)) if !items.is_empty() => items
            .iter()
            .map(|item| match item {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", "),
        Some(Value::Object(map)) if !map.is_empty() => map
            .iter()
            .map(|(key, value)| {
                let value = match value {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                };
                format!("{}: {}", profile_title_case(key), value)
            })
            .collect::<Vec<_>>()
            .join("; "),
        Some(other) if !other.is_null() => other.to_string(),
        _ => "Not recorded".into(),
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

fn normalize_template_selection(template_keys: Vec<String>) -> Vec<String> {
    let selected = template_keys
        .into_iter()
        .map(|key| key.trim().to_ascii_lowercase())
        .filter(|key| !key.is_empty())
        .collect::<Vec<_>>();
    if selected.is_empty() {
        vec![
            "rate_confirmation".into(),
            "bill_of_lading".into(),
            "carrier_packet".into(),
            "shipper_document_package".into(),
        ]
    } else {
        selected
    }
}

fn render_freight_document_template(
    template: &str,
    context: &db::dispatch::FreightDocumentGenerationContext,
) -> String {
    let load_number = context
        .load_number
        .clone()
        .unwrap_or_else(|| format!("Load #{}", context.load_id));
    let route_summary = format!(
        "{} to {}",
        context
            .pickup_location_name
            .clone()
            .unwrap_or_else(|| "Pickup TBD".into()),
        context
            .delivery_location_name
            .clone()
            .unwrap_or_else(|| "Delivery TBD".into())
    );
    let rate_summary = context
        .booked_amount
        .map(|value| format!("${:.2}", value))
        .unwrap_or_else(|| "Rate pending".into());
    let commodity_summary = context.load_title.clone();
    let weight_summary = context
        .weight
        .map(|value| {
            format!(
                "{:.0} {}",
                value,
                context.weight_unit.clone().unwrap_or_else(|| "lbs".into())
            )
        })
        .unwrap_or_else(|| "Weight TBD".into());
    let carrier_summary = context
        .carrier_name
        .clone()
        .unwrap_or_else(|| "Carrier pending".into());
    template
        .replace("{{load_number}}", &load_number)
        .replace("{{load_title}}", &context.load_title)
        .replace("{{route_summary}}", &route_summary)
        .replace("{{rate_summary}}", &rate_summary)
        .replace("{{commodity_summary}}", &commodity_summary)
        .replace("{{weight_summary}}", &weight_summary)
        .replace("{{carrier_summary}}", &carrier_summary)
        .replace(
            "{{special_instructions}}",
            context
                .special_instructions
                .as_deref()
                .unwrap_or("None recorded"),
        )
        .replace(
            "{{generated_at}}",
            &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        )
}

fn load_required_document_checklist(
    documents: &[LoadDocumentRow],
) -> Vec<RequiredDocumentChecklistItem> {
    [(
        "rate_confirmation",
        "Rate confirmation",
        "Load profile",
        "booking",
    )]
    .into_iter()
    .map(|(key, label, scope, lifecycle_state)| {
        let is_satisfied = documents.iter().any(|document| {
            let haystack = format!(
                "{} {} {} {}",
                document.document_name,
                document.document_type_key,
                document.document_type_label,
                document.original_name.clone().unwrap_or_default()
            )
            .to_ascii_lowercase()
            .replace(['-', ' '], "_");
            haystack.contains(key)
        });
        RequiredDocumentChecklistItem {
            key: key.into(),
            label: label.into(),
            requirement_scope: scope.into(),
            lifecycle_state: lifecycle_state.into(),
            is_required: true,
            is_satisfied,
            status_label: if is_satisfied { "Ready" } else { "Missing" }.into(),
            status_tone: if is_satisfied { "success" } else { "warning" }.into(),
            blocking_message: (!is_satisfied)
                .then(|| format!("{} is required before booking/closeout readiness.", label)),
        }
    })
    .collect()
}

fn format_profile_currency(value: Option<f64>) -> String {
    value
        .map(|amount| format!("{:.2}", amount))
        .unwrap_or_else(|| "Not set".into())
}

fn profile_title_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn profile_file_label(file_path: &str) -> String {
    file_path
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(file_path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_load_csv_parser_handles_quotes_and_required_validation() {
        let csv = "title,load_type_id,equipment_id,commodity_type_id,weight,weight_unit,pickup_address,pickup_city,pickup_country,delivery_address,delivery_city,delivery_country,pickup_date,delivery_date,price\n\"Dallas, TX to Joliet\",1,2,3,42000,LBS,\"123 Pickup St\",Dallas,USA,\"500 Delivery Ave\",Joliet,USA,2026-06-01,2026-06-03,2500\nBroken row,1,2,3,0,LBS,,,,,,USA,2026-06-04,2026-06-03,-1";

        let rows = parse_bulk_load_csv(csv).expect("csv parses");

        assert_eq!(rows.len(), 2);
        assert!(rows[0].errors.is_empty());
        assert_eq!(
            rows[0]
                .payload
                .as_ref()
                .map(|payload| payload.title.as_str()),
            Some("Dallas, TX to Joliet")
        );
        assert!(rows[1].errors.iter().any(|error| error.contains("weight")));
        assert!(
            rows[1]
                .errors
                .iter()
                .any(|error| error.contains("delivery_date"))
        );
        assert!(rows[1].errors.iter().any(|error| error.contains("price")));
    }

    #[test]
    fn rating_accessorial_builder_uses_flags_and_amount_overrides() {
        let flags = json!({
            "detention": true,
            "lumper": 185.50,
            "tolls": { "amount": 92.25 },
            "storage": false
        });
        let lines = build_accessorial_lines(
            Some(&flags),
            vec![
                ("detention".into(), "Detention".into(), 75.0),
                ("lumper".into(), "Lumper".into(), 150.0),
                ("storage".into(), "Storage".into(), 100.0),
                ("tolls".into(), "Tolls".into(), 50.0),
            ],
        );

        assert_eq!(lines.len(), 3);
        assert!(
            lines
                .iter()
                .any(|line| line.code == "detention" && line.amount == 75.0)
        );
        assert!(
            lines
                .iter()
                .any(|line| line.code == "lumper" && line.amount == 185.50)
        );
        assert!(
            lines
                .iter()
                .any(|line| line.code == "tolls" && line.amount == 92.25)
        );
    }

    #[test]
    fn carrier_match_scoring_explains_recommended_and_blocked_carriers() {
        let preferred = score_carrier_match(
            10,
            "Atlas Freight".into(),
            Some("Atlas LLC".into()),
            Some("preferred".into()),
            "private",
            Some("Dry Van"),
            Some("Paper Goods"),
            Some("standard"),
            Some("Dallas DC"),
            Some(2400.0),
            Some("published"),
            None,
            vec!["dry_van".into()],
            vec!["dallas".into()],
            vec!["paper_goods".into()],
            vec!["standard".into()],
            vec!["hazmat".into()],
            "available".into(),
            2,
            1_000_000.0,
            5,
        );
        assert!(preferred.eligible);
        assert!(preferred.score > 80);
        assert!(
            preferred
                .explanation
                .iter()
                .any(|line| line.contains("Preferred"))
        );

        let blocked = score_carrier_match(
            11,
            "Blocked Carrier".into(),
            None,
            Some("blocked".into()),
            "private",
            Some("Dry Van"),
            Some("Paper Goods"),
            Some("standard"),
            Some("Dallas DC"),
            Some(2400.0),
            None,
            Some("Tracking stale"),
            vec!["dry_van".into()],
            vec!["dallas".into()],
            vec!["paper_goods".into()],
            vec!["standard".into()],
            Vec::new(),
            "paused".into(),
            0,
            0.0,
            0,
        );
        assert!(!blocked.eligible);
        assert!(blocked.score <= 35);
        assert!(
            blocked
                .blocked_reasons
                .iter()
                .any(|line| line.contains("blocked"))
        );
    }

    #[test]
    fn coordinate_validation_requires_valid_pairs() {
        assert!(validate_coordinates(Some(32.7767), Some(-96.7970)).is_ok());
        assert!(validate_coordinates(Some(91.0), Some(-96.7970)).is_err());
        assert!(validate_coordinates(Some(32.7767), None).is_err());
        assert!(validate_coordinates(None, Some(-96.7970)).is_err());
    }

    #[test]
    fn localization_helpers_normalize_units_and_time_zones() {
        let kg_as_lbs = convert_weight_to_lbs(100.0, "KG").expect("kg converts");
        assert!((kg_as_lbs - 220.462_262_18).abs() < 0.000_1);
        assert_eq!(convert_weight_to_lbs(42000.0, "LBS"), Some(42000.0));
        assert_eq!(convert_weight_to_lbs(1.0, "stones"), None);
        assert!((convert_weight_from_lbs(220.462_262_18, "KG") - 100.0).abs() < 0.000_1);

        assert_eq!(
            normalize_time_zone(Some(&"America/Chicago".to_string())),
            "America/Chicago"
        );
        assert_eq!(normalize_time_zone(Some(&"  UTC  ".to_string())), "UTC");
        assert_eq!(
            normalize_time_zone(Some(&"America Chicago".to_string())),
            "UTC"
        );
        assert_eq!(normalize_time_zone(None), "UTC");
    }

    #[test]
    fn mode_validation_blocks_deferred_modes_and_requires_mode_payload() {
        let mut payload = CreateLoadRequest {
            title: "Mode test".into(),
            load_type_id: 1,
            equipment_id: 1,
            commodity_type_id: 1,
            customer_contract_id: None,
            customer_contract_lane_id: None,
            freight_mode: Some("cross-border".into()),
            visibility: Some("public".into()),
            service_level: None,
            customer_reference: None,
            po_number: None,
            pickup_appointment_ref: None,
            delivery_appointment_ref: None,
            facility_contact_name: None,
            facility_contact_phone: None,
            facility_contact_email: None,
            appointment_window_start: None,
            appointment_window_end: None,
            accessorial_flags: None,
            weight_unit: "LBS".into(),
            weight: 1000.0,
            temperature_data: None,
            container_data: None,
            securement_data: None,
            special_instructions: None,
            is_hazardous: false,
            is_temperature_controlled: false,
            legs: vec![],
        };

        assert!(validate_mode_specific_payload(&payload).is_err());

        payload.freight_mode = Some("drayage".into());
        assert!(validate_mode_specific_payload(&payload).is_err());

        payload.container_data = Some(json!({ "container_number": "MSCU1234567" }));
        assert_eq!(
            validate_mode_specific_payload(&payload).as_deref(),
            Ok("drayage")
        );
    }
}
