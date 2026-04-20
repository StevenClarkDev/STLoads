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
        CreateLoadLegParams, CreateLoadParams, UpsertLoadDocumentParams,
        append_dispatch_desk_follow_up, book_load_leg,
        create_load_document as insert_load_document, create_load_with_legs, find_load_by_id,
        find_load_document_by_id, find_load_document_scope, find_load_id_and_status_for_leg,
        find_load_leg_by_id, find_load_leg_scope, list_load_builder_legs_for_load,
        list_load_documents_for_load, list_load_history_for_load, list_load_legs_for_load,
        list_load_profile_legs_for_load, update_load_document as persist_load_document_updates,
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
use shared::{
    ApiResponse, BookLoadLegRequest, BookLoadLegResponse, CreateLoadRequest, CreateLoadResponse,
    DispatchDeskFollowUpRequest, DispatchDeskFollowUpResponse, DispatchDeskScreen, LoadBoardScreen,
    LoadBuilderDraft, LoadBuilderLegDraft, LoadBuilderOption, LoadBuilderScreen, LoadDocumentRow,
    LoadHandoffSummary, LoadHistoryRow, LoadProfileField, LoadProfileLegRow, LoadProfileScreen,
    RealtimeEvent, RealtimeEventKind, RealtimeTopic, UpsertLoadDocumentRequest,
    UpsertLoadDocumentResponse, VerifyLoadDocumentRequest, VerifyLoadDocumentResponse,
};
use std::collections::HashMap;
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
}
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/legacy-statuses", get(legacy_statuses))
        .route("/load-board", get(load_board))
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
        screen_data::load_board_screen(&state, viewer.as_ref(), query.tab).await,
    ))
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
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: "Sign in before creating a load from the Rust builder.".into(),
        }));
    };

    if !can_manage_loads(&session) {
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
        Ok(created) => Json(ApiResponse::ok(CreateLoadResponse {
            success: true,
            load_id: Some(created.load_id),
            load_number: Some(created.load_number.clone()),
            leg_count: created.leg_count,
            message: format!(
                "{} created load {} with {} leg(s) from the Rust builder. Continue in the Rust load profile for document and follow-up workflow.",
                session.user.name, created.load_number, created.leg_count
            ),
        })),
        Err(error) => Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: None,
            load_number: None,
            leg_count: 0,
            message: format!("Load creation failed: {}", error),
        })),
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
        return Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: None,
            leg_count: 0,
            message: "Sign in before editing a load from the Rust builder.".into(),
        }));
    };

    if !can_manage_loads(&session) {
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

    let (params, leg_params) =
        match build_load_mutation_inputs(pool, session.user.id, &payload).await {
            Ok(values) => values,
            Err(response) => return Json(ApiResponse::ok(response)),
        };

    match update_load_with_legs(pool, load_id, &params, &leg_params, Some(session.user.id)).await {
        Ok(Some(updated)) => Json(ApiResponse::ok(CreateLoadResponse {
            success: true,
            load_id: Some(updated.load_id),
            load_number: Some(updated.load_number.clone()),
            leg_count: updated.leg_count,
            message: format!(
                "{} updated load {} from the Rust builder. Continue in the Rust load profile for documents and follow-up workflow.",
                session.user.name, updated.load_number
            ),
        })),
        Ok(None) => Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: existing_load.load_number,
            leg_count: 0,
            message: format!(
                "Load #{} was not found while applying the Rust builder update.",
                load_id
            ),
        })),
        Err(error) => Json(ApiResponse::ok(CreateLoadResponse {
            success: false,
            load_id: Some(load_id),
            load_number: existing_load.load_number,
            leg_count: 0,
            message: format!("Load update failed: {}", error),
        })),
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
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "Sign in before uploading load documents from the Rust profile.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
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

    if !can_manage_load_documents(&session, load.user_id) {
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
        Ok(Some(document)) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
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
        })),
        Ok(None) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "The target load could not be found while saving the uploaded document."
                .into(),
        })),
        Err(error) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: format!("Document upload create failed: {}", error),
        })),
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
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "Sign in before adding load documents from the Rust profile.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
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

    if !can_manage_load_documents(&session, load.user_id) {
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
            return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id,
                document_id: None,
                message,
            }));
        }
    };

    match insert_load_document(pool, load_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
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
        })),
        Ok(None) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: "The target load could not be found while saving the document.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id,
            document_id: None,
            message: format!("Document create failed: {}", error),
        })),
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
        return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id: Some(document_id),
            message: "Sign in before editing load documents from the Rust profile.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
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

    if !can_manage_load_documents(&session, scope.load_owner_user_id) {
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
            return Json(ApiResponse::ok(UpsertLoadDocumentResponse {
                success: false,
                load_id: scope.load_id,
                document_id: Some(document_id),
                message,
            }));
        }
    };

    match persist_load_document_updates(pool, document_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: true,
            load_id: document.load_id,
            document_id: Some(document.id),
            message: format!(
                "{} updated document {} from the Rust load profile.",
                session.user.name, document.document_name
            ),
        })),
        Ok(None) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id: Some(document_id),
            message: "The requested document disappeared before the update completed.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(UpsertLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id: Some(document_id),
            message: format!("Document update failed: {}", error),
        })),
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
        return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: 0,
            document_id,
            hash: None,
            message: "Sign in before triggering blockchain follow-up from the Rust profile.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
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

    if !can_manage_load_documents(&session, scope.load_owner_user_id) {
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
            return Json(ApiResponse::ok(VerifyLoadDocumentResponse {
                success: true,
                load_id: document.load_id,
                document_id: document.id,
                hash: Some(existing_hash),
                message: format!(
                    "{} is already anchored with a stored hash in the Rust document ledger.",
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
        Ok(Some(document)) => Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: true,
            load_id: document.load_id,
            document_id: document.id,
            hash: document.hash,
            message: format!(
                "{} anchored document {} with a mock blockchain proof for the Rust migration slice.",
                session.user.name, document.document_name
            ),
        })),
        Ok(None) => Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            hash: None,
            message: "The requested document disappeared before blockchain verification completed."
                .into(),
        })),
        Err(error) => Json(ApiResponse::ok(VerifyLoadDocumentResponse {
            success: false,
            load_id: scope.load_id,
            document_id,
            hash: None,
            message: format!("Blockchain verification failed: {}", error),
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
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Unauthorized".into(),
            message: "Sign in as a carrier before booking a leg from the Rust load board.".into(),
        }));
    };

    if session.user.primary_role() != Some(UserRole::Carrier) {
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
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Missing".into(),
            message: "The requested load leg was not found.".into(),
        }));
    };

    if existing_leg.booked_carrier_id == Some(session.user.id) {
        return Json(ApiResponse::ok(BookLoadLegResponse {
            success: true,
            leg_id,
            status_label: "Booked".into(),
            message: "This load leg is already booked by the authenticated carrier account.".into(),
        }));
    }

    if existing_leg.booked_carrier_id.is_some() || existing_leg.status_id >= 4 {
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
        Ok(None) => Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Missing".into(),
            message: "The requested load leg was not found.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(BookLoadLegResponse {
            success: false,
            leg_id,
            status_label: "Error".into(),
            message: format!("Booking action failed: {}", error),
        })),
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
    if bytes.is_empty() {
        return Err("Uploaded document files cannot be empty.".into());
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
        "The Rust load profile now supports binary document upload, restricted file viewing, metadata edits, and blockchain follow-up controls alongside load details, history, and STLOADS context.".into(),
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
        info_fields,
        legs: leg_rows,
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
