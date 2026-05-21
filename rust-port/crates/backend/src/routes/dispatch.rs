use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{NaiveDate, NaiveDateTime};
use db::{
    dispatch::{
        CreateLoadLegParams, CreateLoadParams, DocumentReviewDecision, LoadBoardSearchFilters,
        UpsertLoadDocumentParams, append_dispatch_desk_follow_up, append_load_profile_submission,
        book_load_leg, create_load_document as insert_load_document, create_load_with_legs,
        find_load_by_id, find_load_document_by_id, find_load_document_scope,
        find_load_id_and_status_for_leg, find_load_leg_by_id, find_load_leg_scope,
        list_load_board_saved_searches, list_load_builder_legs_for_load,
        list_load_documents_for_load, list_load_history_for_load, list_load_legs_for_load,
        list_load_profile_legs_for_load, review_load_document,
        update_load_document as persist_load_document_updates, update_load_with_legs,
        upsert_load_board_alert_rule, upsert_load_board_saved_search,
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
use sha2::{Digest, Sha256};
use shared::{
    ApiResponse, BookLoadLegRequest, BookLoadLegResponse, CreateLoadRequest, CreateLoadResponse,
    DispatchDeskFollowUpRequest, DispatchDeskFollowUpResponse, DispatchDeskScreen,
    LoadBoardFilterState, LoadBoardSavedSearchItem, LoadBoardScreen, LoadBuilderDraft,
    LoadBuilderLegDraft, LoadBuilderOption, LoadBuilderScreen, LoadDocumentRow, LoadHandoffSummary,
    LoadHistoryRow, LoadProfileField, LoadProfileLegRow, LoadProfileScreen, RealtimeEvent,
    RealtimeEventKind, RealtimeTopic, ReviewLoadDocumentRequest, ReviewLoadDocumentResponse,
    SaveLoadBoardSearchRequest, SaveLoadBoardSearchResponse, SubmitLoadSummaryRequest,
    SubmitLoadSummaryResponse, UpsertLoadBoardAlertRequest, UpsertLoadBoardAlertResponse,
    UpsertLoadDocumentRequest, UpsertLoadDocumentResponse, VerifyLoadDocumentRequest,
    VerifyLoadDocumentResponse,
};
use std::collections::HashMap;
use tracing::{info, warn};
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
    load_type: Option<String>,
    equipment: Option<String>,
    mode: Option<String>,
    status: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    min_rate: Option<String>,
    max_rate: Option<String>,
    min_rpm: Option<String>,
    max_rpm: Option<String>,
    min_weight: Option<String>,
    max_weight: Option<String>,
    hazmat: Option<bool>,
    temperature_controlled: Option<bool>,
    service_level: Option<String>,
    visibility: Option<String>,
    sort: Option<String>,
    page: Option<i64>,
    per_page: Option<i64>,
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

fn load_board_filters_from_query(query: &LoadBoardQuery) -> LoadBoardSearchFilters {
    LoadBoardSearchFilters {
        origin: clean_string(query.origin.as_deref()),
        destination: clean_string(query.destination.as_deref()),
        load_type: clean_string(query.load_type.as_deref()),
        equipment: clean_string(query.equipment.as_deref()),
        mode: clean_string(query.mode.as_deref()),
        status: clean_string(query.status.as_deref()),
        date_from: parse_date(query.date_from.as_deref()),
        date_to: parse_date(query.date_to.as_deref()),
        min_rate: parse_f64(query.min_rate.as_deref()),
        max_rate: parse_f64(query.max_rate.as_deref()),
        min_rpm: parse_f64(query.min_rpm.as_deref()),
        max_rpm: parse_f64(query.max_rpm.as_deref()),
        min_weight: parse_f64(query.min_weight.as_deref()),
        max_weight: parse_f64(query.max_weight.as_deref()),
        hazmat: query.hazmat,
        temperature_controlled: query.temperature_controlled,
        service_level: clean_string(query.service_level.as_deref()),
        visibility: clean_string(query.visibility.as_deref()),
        sort: clean_string(query.sort.as_deref()),
        page: query.page.unwrap_or(1).max(1),
        per_page: query.per_page.unwrap_or(20).clamp(1, 100),
    }
}

fn load_board_filters_from_state(state: &LoadBoardFilterState) -> LoadBoardSearchFilters {
    LoadBoardSearchFilters {
        origin: clean_string(state.origin.as_deref()),
        destination: clean_string(state.destination.as_deref()),
        load_type: clean_string(state.load_type.as_deref()),
        equipment: clean_string(state.equipment.as_deref()),
        mode: clean_string(state.mode.as_deref()),
        status: clean_string(state.status.as_deref()),
        date_from: parse_date(state.date_from.as_deref()),
        date_to: parse_date(state.date_to.as_deref()),
        min_rate: parse_f64(state.min_rate.as_deref()),
        max_rate: parse_f64(state.max_rate.as_deref()),
        min_rpm: parse_f64(state.min_rpm.as_deref()),
        max_rpm: parse_f64(state.max_rpm.as_deref()),
        min_weight: parse_f64(state.min_weight.as_deref()),
        max_weight: parse_f64(state.max_weight.as_deref()),
        hazmat: state.hazmat,
        temperature_controlled: state.temperature_controlled,
        service_level: clean_string(state.service_level.as_deref()),
        visibility: clean_string(state.visibility.as_deref()),
        sort: clean_string(state.sort.as_deref()),
        page: parse_i64(state.page.as_deref()).unwrap_or(1).max(1),
        per_page: parse_i64(state.per_page.as_deref())
            .unwrap_or(20)
            .clamp(1, 100),
    }
}

fn clean_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_f64(value: Option<&str>) -> Option<f64> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<f64>().ok())
}

fn parse_i64(value: Option<&str>) -> Option<i64> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<i64>().ok())
}

fn parse_date(value: Option<&str>) -> Option<NaiveDate> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
}

fn session_tenant_id(session: &auth_session::ResolvedSession) -> String {
    session
        .session
        .tenant_scope
        .as_ref()
        .map(|scope| scope.tenant_id.clone())
        .unwrap_or_else(|| "legacy".into())
}

fn format_datetime(value: &NaiveDateTime) -> String {
    value.format("%b %-d, %Y %H:%M").to_string()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/legacy-statuses", get(legacy_statuses))
        .route("/load-board", get(load_board))
        .route("/load-board/saved-searches", get(load_board_saved_searches))
        .route("/load-board/saved-searches", post(save_load_board_search))
        .route(
            "/load-board/saved-searches/{saved_search_id}/alert",
            post(upsert_load_board_alert),
        )
        .route("/desk/{desk_key}", get(dispatch_desk))
        .route(
            "/desk/legs/{leg_id}/follow-up",
            post(add_dispatch_desk_follow_up),
        )
        .route("/load-builder", get(load_builder))
        .route("/loads", post(create_load))
        .route("/loads/{load_id}/builder", get(edit_load_builder))
        .route("/loads/{load_id}/update", post(update_load))
        .route("/loads/{load_id}", get(load_profile))
        .route("/loads/{load_id}/submit", post(submit_load_summary))
        .route(
            "/loads/{load_id}/documents",
            post(create_load_document_handler),
        )
        .route(
            "/loads/{load_id}/documents/upload",
            post(upload_load_document_handler).layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
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
        .route(
            "/documents/{document_id}/review",
            post(review_load_document_handler),
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
        screen_data::load_board_screen(
            &state,
            viewer.as_ref(),
            query.tab.clone(),
            load_board_filters_from_query(&query),
        )
        .await,
    ))
}

async fn load_board_saved_searches(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<Vec<LoadBoardSavedSearchItem>>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(Vec::new()));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(Vec::new()));
    };
    let tenant_id = session_tenant_id(&session);
    let rows = list_load_board_saved_searches(pool, &tenant_id, session.user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| LoadBoardSavedSearchItem {
            id: row.id.max(0) as u64,
            name: row.name,
            alert_enabled: row.alert_enabled,
            updated_at_label: format_datetime(&row.updated_at),
        })
        .collect::<Vec<_>>();
    Json(ApiResponse::ok(rows))
}

async fn save_load_board_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SaveLoadBoardSearchRequest>,
) -> Json<ApiResponse<SaveLoadBoardSearchResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(SaveLoadBoardSearchResponse {
            success: false,
            saved_search_id: None,
            message: "Sign in before saving a load-board search.".into(),
        }));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(SaveLoadBoardSearchResponse {
            success: false,
            saved_search_id: None,
            message: "Saved searches are unavailable because the database is not connected.".into(),
        }));
    };
    let name = payload.name.trim();
    if name.is_empty() {
        return Json(ApiResponse::ok(SaveLoadBoardSearchResponse {
            success: false,
            saved_search_id: None,
            message: "Name this search before saving it.".into(),
        }));
    }
    let tenant_id = session_tenant_id(&session);
    match upsert_load_board_saved_search(
        pool,
        &tenant_id,
        session.user.id,
        name,
        &load_board_filters_from_state(&payload.filters),
        payload.alert_enabled,
    )
    .await
    {
        Ok(saved) => Json(ApiResponse::ok(SaveLoadBoardSearchResponse {
            success: true,
            saved_search_id: Some(saved.id.max(0) as u64),
            message: "Saved search is ready for this tenant.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(SaveLoadBoardSearchResponse {
            success: false,
            saved_search_id: None,
            message: format!("Saved search failed: {}", error),
        })),
    }
}

async fn upsert_load_board_alert(
    State(state): State<AppState>,
    Path(saved_search_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpsertLoadBoardAlertRequest>,
) -> Json<ApiResponse<UpsertLoadBoardAlertResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(UpsertLoadBoardAlertResponse {
            success: false,
            alert_rule_id: None,
            message: "Sign in before changing load-board alerts.".into(),
        }));
    };
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpsertLoadBoardAlertResponse {
            success: false,
            alert_rule_id: None,
            message: "Alert rules are unavailable because the database is not connected.".into(),
        }));
    };
    let tenant_id = session_tenant_id(&session);
    match upsert_load_board_alert_rule(
        pool,
        &tenant_id,
        saved_search_id,
        session.user.id,
        &payload.channel,
        payload.active,
    )
    .await
    {
        Ok(alert) => Json(ApiResponse::ok(UpsertLoadBoardAlertResponse {
            success: true,
            alert_rule_id: Some(alert.id.max(0) as u64),
            message: "Alert rule updated for this saved search.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(UpsertLoadBoardAlertResponse {
            success: false,
            alert_rule_id: None,
            message: format!("Alert rule update failed: {}", error),
        })),
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
            message: "Sign in before adding a dispatch desk follow-up.".into(),
        }));
    };

    if !can_access_dispatch_desk_actions(&session) {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "This account does not have dispatch desk follow-up access.".into(),
        }));
    }

    let note = payload.note.trim();
    if note.is_empty() {
        return Json(ApiResponse::ok(DispatchDeskFollowUpResponse {
            success: false,
            leg_id,
            load_id: 0,
            message: "Enter a follow-up note before saving it to the dispatch desk.".into(),
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
                    kind: RealtimeEventKind::LoadBoardListingUpdated,
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
                .for_tenant(session_tenant_id(&session))
                .for_resource("load_leg", leg_id.max(0) as u64)
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

async fn submit_load_summary(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<SubmitLoadSummaryRequest>,
) -> Json<ApiResponse<SubmitLoadSummaryResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(SubmitLoadSummaryResponse {
            success: false,
            load_id,
            status_label: "Unauthenticated".into(),
            message: "Sign in before submitting the load summary.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(SubmitLoadSummaryResponse {
            success: false,
            load_id,
            status_label: "Unavailable".into(),
            message: format!(
                "Load submission is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
        return Json(ApiResponse::ok(SubmitLoadSummaryResponse {
            success: false,
            load_id,
            status_label: "Not Found".into(),
            message: format!("Load #{} was not found.", load_id),
        }));
    };

    if !can_manage_existing_load(&session, load.user_id) {
        return Json(ApiResponse::ok(SubmitLoadSummaryResponse {
            success: false,
            load_id,
            status_label: "Forbidden".into(),
            message: "This account cannot submit the selected load.".into(),
        }));
    }

    let load_label = load
        .load_number
        .clone()
        .unwrap_or_else(|| format!("#{}", load.id));
    let note = payload
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "{} submitted marketplace load {} from the detail summary.",
                session.user.name, load_label
            )
        });

    match append_load_profile_submission(pool, load.id, Some(session.user.id), Some(&note)).await {
        Ok(Some(submitted_load)) => {
            let status_label = profile_load_leg_status_label(submitted_load.status);
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::LoadBoardListingUpdated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: submitted_load
                        .user_id
                        .and_then(|value| if value > 0 { Some(value as u64) } else { None }),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "{} submitted marketplace load {}.",
                        session.user.name, load_label
                    ),
                })
                .for_permission_keys(["manage_loads", "access_admin_portal"])
                .for_tenant(session_tenant_id(&session)),
            );

            Json(ApiResponse::ok(SubmitLoadSummaryResponse {
                success: true,
                load_id: submitted_load.id,
                status_label,
                message: format!("Load {} summary submitted.", load_label),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(SubmitLoadSummaryResponse {
            success: false,
            load_id,
            status_label: "Not Found".into(),
            message: format!("Load #{} was not found.", load_id),
        })),
        Err(error) => Json(ApiResponse::ok(SubmitLoadSummaryResponse {
            success: false,
            load_id,
            status_label: "Submit Failed".into(),
            message: format!("Load summary submit failed: {}", error),
        })),
    }
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
            message: "Sign in before posting a marketplace load.".into(),
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
            message: "This account does not have marketplace posting access.".into(),
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
                "Marketplace posting is unavailable because the database is {} on {}.",
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
            message: "Enter a title before posting the load.".into(),
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
            message: "Add at least one leg before posting the load.".into(),
        }));
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

    let params = CreateLoadParams {
        title,
        owner_user_id: session.user.id,
        load_type_id: payload.load_type_id as i64,
        equipment_id: payload.equipment_id as i64,
        commodity_type_id: payload.commodity_type_id as i64,
        weight_unit: payload.weight_unit.clone(),
        weight: payload.weight,
        special_instructions: payload
            .special_instructions
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        is_hazardous: payload.is_hazardous,
        is_temperature_controlled: payload.is_temperature_controlled,
    };

    match create_load_with_legs(pool, &params, &leg_params, Some(session.user.id)).await {
        Ok(created) => {
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
                    "{} posted marketplace load {} with {} leg(s). Continue to the profile for documents and follow-up.",
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
            message: "Sign in before editing a marketplace load.".into(),
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
            message: "This account does not have marketplace load update access.".into(),
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
            message: format!("Load #{} was not found.", load_id),
        }));
    };

    if !can_manage_existing_load(&session, existing_load.user_id) {
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: existing_load.load_number,
            leg_count: existing_load.leg_count.max(0) as u64,
            message: "This account cannot edit the selected marketplace load.".into(),
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
            message: "This load already has booked or active legs. Continue from the load profile."
                .into(),
        }));
    }

    let (params, leg_params) =
        match build_load_mutation_inputs(pool, session.user.id, &payload).await {
            Ok(values) => values,
            Err(response) => return Json(ApiResponse::ok(response)),
        };

    match update_load_with_legs(pool, load_id, &params, &leg_params, Some(session.user.id)).await {
        Ok(Some(updated)) => {
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
                    "{} updated marketplace load {}. Continue to the profile for documents and follow-up.",
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
                message: format!("Load #{} was not found while saving changes.", load_id),
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
            message: "Sign in before uploading load documents.".into(),
        }));
    };

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
            message: format!("Load #{} was not found.", load_id),
        }));
    };

    if !can_manage_load_documents(&session, load.user_id) {
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
        file_hash: Some(sha256_hex(&upload.bytes)),
    };

    match insert_load_document(pool, load_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => {
            log_dispatch_success(
                "upload_load_document",
                Some(session.user.id),
                Some(load_id),
                None,
            );
            let load_owner_user_id = load.user_id.and_then(|user_id| {
                if user_id > 0 {
                    Some(user_id as u64)
                } else {
                    None
                }
            });
            let mut target_user_ids = vec![session.user.id.max(0) as u64];
            if let Some(load_owner_user_id) = load_owner_user_id {
                target_user_ids.push(load_owner_user_id);
            }
            target_user_ids.sort_unstable();
            target_user_ids.dedup();
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::LoadDocumentUpdated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: load_owner_user_id,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "{} uploaded document {} to load {}.",
                        session.user.name,
                        document.document_name,
                        load.load_number
                            .clone()
                            .unwrap_or_else(|| format!("#{}", load.id))
                    ),
                })
                .for_user_ids(target_user_ids)
                .for_permission_keys(["manage_loads", "access_admin_portal"])
                .for_tenant(session_tenant_id(&session))
                .for_resource("load", load_id.max(0) as u64)
                .with_topics([RealtimeTopic::LoadBoard.as_key()]),
            );
            Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: true,
                load_id,
                document_id: Some(document.id),
                message: format!(
                    "{} uploaded document {} to load {}. The file is now available to admins and the uploader.",
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
            "Sign in before viewing load document files.",
        );
    };

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
            "Only admins and the uploader can view this file.",
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
            message: "Sign in before adding load documents.".into(),
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
            message: format!("Load #{} was not found.", load_id),
        }));
    };

    if !can_manage_load_documents(&session, load.user_id) {
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
            message: "Sign in before editing load documents.".into(),
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
            message: format!("Document #{} was not found.", document_id),
        }));
    };

    if !can_manage_load_documents(&session, scope.load_owner_user_id) {
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
                    "{} updated document {} from the load profile.",
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
            message: "Sign in before anchoring document proof.".into(),
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
                "Blockchain verification is unavailable because the database is {} on {}.",
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
            message: format!("Document #{} was not found.", document_id),
        }));
    };

    if !can_manage_load_documents(&session, scope.load_owner_user_id) {
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
                "The authenticated session cannot verify blockchain state for this load document."
                    .into(),
        }));
    }

    if let Ok(Some(document)) = find_load_document_by_id(pool, document_id).await {
        if let Some(existing_hash) = document.hash.clone() {
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
                    "{} is already anchored with a stored hash.",
                    document.document_name
                ),
            }));
        }
    }

    let note = payload
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match persist_load_document_blockchain_verification(
        pool,
        document_id,
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
                    "{} anchored document {} with a blockchain proof.",
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
                    "The requested document disappeared before blockchain verification completed."
                        .into(),
            }))
        }
        Err(error) => {
            log_dispatch_failure(
                "verify_load_document",
                Some(session.user.id),
                Some(scope.load_id),
                None,
                &format!("blockchain verification failed: {error}"),
            );
            Json(ApiResponse::ok(VerifyLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id,
                hash: None,
                message: format!("Blockchain verification failed: {}", error),
            }))
        }
    }
}

async fn review_load_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ReviewLoadDocumentRequest>,
) -> Json<ApiResponse<ReviewLoadDocumentResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            review_status: "unauthorized".into(),
            malware_scan_status: "unknown".into(),
            payment_ready_blocked: true,
            message: "Sign in before reviewing load documents.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            review_status: "unavailable".into(),
            malware_scan_status: "unknown".into(),
            payment_ready_blocked: true,
            message: format!(
                "Document review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(scope) = find_load_document_scope(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        return Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            review_status: "missing".into(),
            malware_scan_status: "unknown".into(),
            payment_ready_blocked: true,
            message: format!("Document #{} was not found.", document_id),
        }));
    };

    if !can_manage_load_documents(&session, scope.load_owner_user_id) {
        return Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            review_status: "forbidden".into(),
            malware_scan_status: "unknown".into(),
            payment_ready_blocked: true,
            message: "The authenticated session cannot review this load document.".into(),
        }));
    }

    let decision = match payload.decision.trim().to_ascii_lowercase().as_str() {
        "approve" | "approved" => DocumentReviewDecision::Approve,
        "reject" | "rejected" => DocumentReviewDecision::Reject,
        "request_revision" | "revision" | "revision_requested" => {
            DocumentReviewDecision::RequestRevision
        }
        _ => {
            return Json(ApiResponse::ok(ReviewLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id,
                review_status: "invalid".into(),
                malware_scan_status: "unknown".into(),
                payment_ready_blocked: true,
                message: "Document review decision must be approve, reject, or request_revision."
                    .into(),
            }));
        }
    };

    let note = payload
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match review_load_document(pool, document_id, decision, note, Some(session.user.id)).await {
        Ok(Some(document)) => Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: true,
            load_id: document.load_id,
            document_id: document.id,
            review_status: document.review_status.clone(),
            malware_scan_status: document.malware_scan_status.clone(),
            payment_ready_blocked: document.payment_ready_blocked,
            message: format!(
                "{} marked {} as {}.",
                session.user.name,
                document.document_name,
                profile_title_case(&document.review_status)
            ),
        })),
        Ok(None) => Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            review_status: "missing".into(),
            malware_scan_status: "unknown".into(),
            payment_ready_blocked: true,
            message: "The requested document disappeared before review completed.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ReviewLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            review_status: "failed".into(),
            malware_scan_status: "unknown".into(),
            payment_ready_blocked: true,
            message: format!("Document review failed: {}", error),
        })),
    }
}
async fn book_leg(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<BookLoadLegRequest>,
) -> Json<ApiResponse<BookLoadLegResponse>> {
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
            message: "Sign in as a carrier before booking marketplace freight.".into(),
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
            message: "Only authenticated carrier accounts can self-book marketplace freight."
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

    match book_load_leg(
        pool,
        leg_id,
        session.user.id,
        payload.booked_amount,
        Some(session.user.id),
    )
    .await
    {
        Ok(Some(updated_leg)) => {
            let tenant_id = session_tenant_id(&session);
            log_dispatch_success(
                "book_leg",
                Some(session.user.id),
                Some(updated_leg.load_id),
                Some(leg_id),
            );
            let mut target_user_ids = vec![session.user.id.max(0) as u64];
            if let Ok(Some(scope)) = find_load_leg_scope(pool, leg_id).await {
                if let Some(owner_id) = scope.load_owner_user_id {
                    if owner_id > 0 {
                        target_user_ids.push(owner_id as u64);
                    }
                }
                if let Some(booked_carrier_id) = scope.booked_carrier_id {
                    if booked_carrier_id > 0 {
                        target_user_ids.push(booked_carrier_id as u64);
                    }
                }
            }
            target_user_ids.sort_unstable();
            target_user_ids.dedup();

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::BookingAwarded,
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
                .for_tenant(tenant_id.clone())
                .for_resource("load_leg", leg_id.max(0) as u64)
                .with_topics([RealtimeTopic::LoadBoard.as_key()]),
            );

            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::PaymentUpdated,
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
                .for_tenant(tenant_id)
                .for_resource("load_leg", leg_id.max(0) as u64)
                .with_topics([RealtimeTopic::AdminPayments.as_key()]),
            );

            Json(ApiResponse::ok(BookLoadLegResponse {
                success: true,
                leg_id,
                status_label: "Booked".into(),
                message: "Load leg booked. The board will refresh automatically.".into(),
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
            vec!["Sign in to post or edit marketplace loads.".into()],
        );
    };

    if !can_manage_loads(viewer) {
        return empty_load_builder_screen(
            state,
            load_id,
            vec!["This account does not have marketplace posting access.".into()],
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return empty_load_builder_screen(
            state,
            load_id,
            vec![format!(
                "Marketplace load data is unavailable because the database is {} on {}.",
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
        "Google address search is available for pickup and delivery locations.".into(),
        "Booked or active loads are locked from posting edits.".into(),
    ];

    let draft = if let Some(load_id) = load_id {
        let Some(load) = find_load_by_id(pool, load_id).await.unwrap_or_default() else {
            return empty_load_builder_screen(
                state,
                Some(load_id),
                vec![format!("Load #{} was not found.", load_id)],
            );
        };

        if !can_manage_existing_load(viewer, load.user_id) {
            return empty_load_builder_screen(
                state,
                Some(load_id),
                vec!["This account cannot edit the selected marketplace load.".into()],
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
                    "This load already has booked or active legs. Continue from the load profile."
                        .into(),
                ],
            );
        }

        notes.push("This posting is ready for marketplace load edits.".into());

        Some(LoadBuilderDraft {
            load_id: load.id.max(0) as u64,
            load_number: load.load_number.clone(),
            title: load.title,
            load_type_id: load.load_type_id.map(|value| value.max(0) as u64),
            equipment_id: load.equipment_id.map(|value| value.max(0) as u64),
            commodity_type_id: load.commodity_type_id.map(|value| value.max(0) as u64),
            weight_unit: load.weight_unit,
            weight: load.weight,
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
            "This marketplace builder creates STLoads postings for carrier bidding, booking, documents, and payment workflow.".into(),
        );
        None
    };

    let is_edit_mode = draft.is_some();

    LoadBuilderScreen {
        title: if is_edit_mode {
            "Edit Marketplace Load".into()
        } else {
            "Create Marketplace Load".into()
        },
        subtitle: if is_edit_mode {
            "Revise marketplace posting details before carrier booking and execution workflow begins.".into()
        } else {
            "Create a marketplace load posting for carrier bidding, booking, documents, and payment workflow.".into()
        },
        mode: if is_edit_mode {
            "edit".into()
        } else {
            "create".into()
        },
        submit_label: if is_edit_mode {
            "Save marketplace changes".into()
        } else {
            "Post marketplace load".into()
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
    _state: &AppState,
    load_id: Option<i64>,
    notes: Vec<String>,
) -> LoadBuilderScreen {
    LoadBuilderScreen {
        title: if load_id.is_some() {
            "Edit Marketplace Load".into()
        } else {
            "Create Marketplace Load".into()
        },
        subtitle: "Secure marketplace posting builder".into(),
        mode: if load_id.is_some() {
            "edit".into()
        } else {
            "create".into()
        },
        submit_label: if load_id.is_some() {
            "Save marketplace changes".into()
        } else {
            "Post marketplace load".into()
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

fn format_builder_date(value: Option<&chrono::NaiveDateTime>) -> String {
    value
        .map(|date| date.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
fn parse_date_for_storage(value: &str) -> Result<NaiveDateTime, String> {
    let date = NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| "use YYYY-MM-DD format".to_string())?;
    date.and_hms_opt(0, 0, 0)
        .ok_or_else(|| "date could not be normalized".to_string())
}

async fn resolve_leg_location_reference(
    pool: &db::DbPool,
    label: &str,
    location_id: Option<u64>,
    address: Option<&str>,
    city: Option<&str>,
    country: Option<&str>,
) -> Result<i64, String> {
    let normalized_address = address.map(str::trim).unwrap_or_default();
    if !normalized_address.is_empty() {
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

    let params = CreateLoadParams {
        title,
        owner_user_id,
        load_type_id: payload.load_type_id as i64,
        equipment_id: payload.equipment_id as i64,
        commodity_type_id: payload.commodity_type_id as i64,
        weight_unit: payload.weight_unit.clone(),
        weight: payload.weight,
        special_instructions: payload
            .special_instructions
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        is_hazardous: payload.is_hazardous,
        is_temperature_controlled: payload.is_temperature_controlled,
    };

    Ok((params, leg_params))
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
    if !is_supported_load_document_type(&document_type) {
        return Err("Document type must be rate_confirmation, bill_of_lading, delivery_pod, invoice, lumper_receipt, insurance_certificate, carrier_packet, customs_document, blockchain, or other.".into());
    }

    let file_path = payload.file_path.trim().to_string();
    if file_path.is_empty() {
        return Err("Enter a storage path or URL before saving the document row.".into());
    }

    if payload.file_size.unwrap_or(0) < 0 {
        return Err("File size cannot be negative.".into());
    }
    if payload.file_size.unwrap_or(0) as usize > max_document_upload_bytes() {
        return Err(format!(
            "Document metadata cannot exceed {} MB.",
            max_document_upload_bytes() / 1024 / 1024
        ));
    }
    if let Some(mime_type) = payload
        .mime_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !is_supported_document_mime_type(mime_type) {
            return Err(format!(
                "Document MIME type {} is not allowed for production upload.",
                mime_type
            ));
        }
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
        file_hash: None,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
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

fn document_review_tone(status: &str) -> &'static str {
    match status {
        "approved" => "success",
        "rejected" => "danger",
        "revision_requested" => "warning",
        "pending_review" => "warning",
        _ => "secondary",
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
    if !is_supported_load_document_type(&document_type) {
        return Err("Document type must be rate_confirmation, bill_of_lading, delivery_pod, invoice, lumper_receipt, insurance_certificate, carrier_packet, customs_document, or other.".into());
    }
    let bytes = bytes.ok_or_else(|| "Choose a file before uploading a document.".to_string())?;
    if bytes.is_empty() {
        return Err("Uploaded document files cannot be empty.".into());
    }
    if bytes.len() > max_document_upload_bytes() {
        return Err(format!(
            "Uploaded document files cannot exceed {} MB.",
            max_document_upload_bytes() / 1024 / 1024
        ));
    }

    if let Some(mime_type) = mime_type.as_deref() {
        if !is_supported_document_mime_type(mime_type) {
            return Err(format!(
                "Document MIME type {} is not allowed for production upload.",
                mime_type
            ));
        }
    }

    Ok(ParsedUploadedDocument {
        document_name,
        document_type,
        original_name: original_name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "document.bin".into()),
        mime_type: mime_type
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        bytes,
    })
}

fn max_document_upload_bytes() -> usize {
    25 * 1024 * 1024
}

fn is_supported_load_document_type(document_type: &str) -> bool {
    matches!(
        document_type,
        "rate_confirmation"
            | "bill_of_lading"
            | "delivery_pod"
            | "invoice"
            | "lumper_receipt"
            | "insurance_certificate"
            | "carrier_packet"
            | "customs_document"
            | "blockchain"
            | "other"
    )
}

fn is_supported_document_mime_type(mime_type: &str) -> bool {
    let normalized = mime_type.trim().to_ascii_lowercase();
    normalized == "application/pdf"
        || normalized == "image/jpeg"
        || normalized == "image/png"
        || normalized == "text/plain"
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
            vec!["Sign in before opening a load profile.".into()],
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
            vec![format!("Load #{} was not found.", load_id)],
        );
    };

    if !can_view_load_profile(viewer, &load) {
        return empty_load_profile_screen(
            state,
            load_id,
            vec![
                "This account cannot open the selected load profile.".into(),
                "Owner and admin visibility is enabled first so document and history parity can land safely.".into(),
            ],
        );
    }

    let can_manage_documents = can_manage_load_documents(viewer, load.user_id);

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

    let info_fields = vec![
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
                        format!("Anchored: {}", tx_id)
                    }
                    _ => "Anchored with stored hash".into(),
                })
            } else if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("Pending blockchain anchor".into())
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
                review_status_label: profile_title_case(&document.review_status),
                review_status_tone: document_review_tone(&document.review_status).into(),
                malware_scan_status_label: profile_title_case(&document.malware_scan_status),
                payment_ready_blocked: document.payment_ready_blocked,
                can_review: can_manage_documents,
                uploaded_at_label: format_profile_datetime(&document.created_at),
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

    let stloads_handoff = handoff.map(|handoff| {
        let (compliance_label, compliance_tone) =
            profile_compliance_badge(handoff.compliance_passed, &handoff.status);
        LoadHandoffSummary {
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
            compliance_label,
            compliance_tone,
            packet_id: handoff.paperwork_packet_id,
            bol_number: handoff.bol_number,
            freight_bill_number: handoff.freight_bill_number,
            document_packet_hash: handoff.document_packet_hash,
            document_packet_url: handoff.document_packet_url,
            document_status_label: profile_document_status_label(
                handoff.required_documents_status.as_ref(),
            ),
            blocker_label: profile_blocker_label(
                handoff.compliance_blockers.as_ref(),
                handoff.last_push_result.as_deref(),
            ),
            customs_status_label: profile_customs_status_label(
                handoff.customs_movement_type.as_deref(),
                handoff.customs_readiness.as_deref(),
                handoff.ace_entry_number.as_deref(),
                handoff.isf_status.as_deref(),
                handoff.in_bond_status.as_deref(),
                handoff.aes_itn.as_deref(),
                handoff.pga_requirements.as_ref(),
            ),
        }
    });

    let notes = vec![
        "Upload, review, and manage documents from this load profile.".into(),
        "Files are visible only to admins and the uploader.".into(),
        format!("Storage backend: {}.", state.document_storage.backend()),
    ];

    LoadProfileScreen {
        title: "Load Profile".into(),
        subtitle: "Review marketplace load details, documents, and execution status.".into(),
        load_id: load.id.max(0) as u64,
        load_number: load.load_number,
        can_manage_documents,
        info_fields,
        legs: leg_rows,
        documents: document_rows,
        history: history_rows,
        stloads_handoff,
        notes,
    }
}

fn empty_load_profile_screen(
    _state: &AppState,
    load_id: i64,
    notes: Vec<String>,
) -> LoadProfileScreen {
    LoadProfileScreen {
        title: "Load Profile".into(),
        subtitle: "Secure load detail view".into(),
        load_id: load_id.max(0) as u64,
        load_number: None,
        can_manage_documents: false,
        info_fields: Vec::new(),
        legs: Vec::new(),
        documents: Vec::new(),
        history: Vec::new(),
        stloads_handoff: None,
        notes,
    }
}

fn can_manage_load_documents(
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

fn can_view_load_profile(
    viewer: &crate::auth_session::ResolvedSession,
    load: &db::dispatch::LoadRecord,
) -> bool {
    can_manage_load_documents(viewer, load.user_id)
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
        "quarantined" | "blocked" => "danger",
        "published" => "success",
        "push_failed" => "danger",
        "requeue_required" => "primary",
        "withdrawn" => "secondary",
        "closed" => "dark",
        _ => "secondary",
    }
}

fn profile_compliance_badge(compliance_passed: bool, status: &str) -> (String, String) {
    if matches!(status, "quarantined" | "blocked") {
        ("Hold".into(), "danger".into())
    } else if compliance_passed {
        ("Clear".into(), "success".into())
    } else {
        ("Review".into(), "warning".into())
    }
}

fn profile_document_status_label(value: Option<&serde_json::Value>) -> String {
    let Some(object) = value.and_then(|value| value.as_object()) else {
        return "Docs not received".into();
    };
    let ready = object
        .values()
        .filter_map(|value| value.as_str())
        .filter(|status| matches!(*status, "generated" | "attached" | "clear" | "ready"))
        .count();
    if object.is_empty() {
        "Docs empty".into()
    } else if ready == object.len() {
        format!("Docs clear ({}/{})", ready, object.len())
    } else {
        format!("Docs pending ({}/{})", ready, object.len())
    }
}

fn profile_blocker_label(
    blockers: Option<&serde_json::Value>,
    fallback: Option<&str>,
) -> Option<String> {
    if let Some(array) = blockers.and_then(|value| value.as_array()) {
        if !array.is_empty() {
            return Some(format!("{} blocker(s)", array.len()));
        }
    }
    fallback
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.chars().take(140).collect())
}

fn profile_customs_status_label(
    movement_type: Option<&str>,
    readiness: Option<&str>,
    ace_entry: Option<&str>,
    isf_status: Option<&str>,
    in_bond_status: Option<&str>,
    aes_itn: Option<&str>,
    pga_requirements: Option<&serde_json::Value>,
) -> Option<String> {
    let has_customs_signal = [
        movement_type,
        readiness,
        ace_entry,
        isf_status,
        in_bond_status,
        aes_itn,
    ]
    .into_iter()
    .flatten()
    .any(|value| !value.trim().is_empty())
        || pga_requirements.is_some();
    if !has_customs_signal {
        return None;
    }
    Some(format!(
        "{} / ACE {} / ISF {} / In-bond {} / AES {} / PGA {}",
        readiness
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("pending"),
        ace_entry
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("pending"),
        isf_status
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("pending"),
        in_bond_status
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("pending"),
        aes_itn
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("pending"),
        pga_requirements
            .and_then(|value| value.as_array())
            .map(|items| items.len().to_string())
            .unwrap_or_else(|| "0".into())
    ))
}

fn format_profile_date(value: Option<&chrono::NaiveDateTime>) -> String {
    value
        .map(|date| date.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| "TBD".into())
}

fn format_profile_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
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
