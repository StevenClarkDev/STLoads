use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{NaiveDateTime, Utc};
use db::tracking::{
    CreateLegDocumentParams, advance_leg_execution, create_leg_document, create_tracking_point,
    find_execution_leg_by_id, find_leg_document_by_id, find_leg_document_scope,
    latest_tracking_point_for_leg, list_execution_note_history_for_leg, list_leg_documents,
    list_leg_events, list_tracking_points_for_leg,
};
use domain::{
    auth::UserRole,
    dispatch::LegacyLoadLegStatusCode,
    tracking::{LegEventType, TrackingModuleContract, leg_event_types, tracking_module_contract},
};
use serde::Serialize;
use shared::{
    ApiResponse, ExecutionActionItem, ExecutionDocumentItem, ExecutionDocumentTypeOption,
    ExecutionLegActionRequest, ExecutionLegActionResponse, ExecutionLegScreen,
    ExecutionLocationPingRequest, ExecutionLocationPingResponse, ExecutionNoteItem,
    ExecutionTimelineItem, ExecutionTrackingPointItem, ExecutionUploadDocumentResponse,
    RealtimeEvent, RealtimeEventKind, RealtimeTopic,
};

use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, state::AppState};

#[derive(Debug, Serialize)]
struct ExecutionOverview {
    contract: TrackingModuleContract,
    leg_event_types: usize,
    trackable_statuses: usize,
    screen_routes: Vec<&'static str>,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/leg-event-types", get(event_types))
        .route("/legs/{leg_id}", get(leg_screen))
        .route("/legs/{leg_id}/actions", post(run_leg_action))
        .route(
            "/legs/{leg_id}/documents/upload",
            post(upload_leg_document).layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route("/legs/{leg_id}/location", post(store_leg_location))
        .route(
            "/documents/{document_id}/file",
            get(download_leg_document_file),
        )
}

async fn index() -> Json<ApiResponse<ExecutionOverview>> {
    let contract = tracking_module_contract();
    Json(ApiResponse::ok(ExecutionOverview {
        leg_event_types: leg_event_types().len(),
        trackable_statuses: contract.trackable_status_codes.len(),
        screen_routes: vec!["/execution/legs/{leg_id}"],
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("execution route group ready"))
}

async fn contract() -> Json<ApiResponse<TrackingModuleContract>> {
    Json(ApiResponse::ok(tracking_module_contract()))
}

async fn event_types() -> Json<ApiResponse<Vec<LegEventType>>> {
    Json(ApiResponse::ok(leg_event_types().to_vec()))
}

async fn leg_screen(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ExecutionLegScreen>>, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some(leg) = find_execution_leg_by_id(pool, leg_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !can_view_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    let tracking_points = list_tracking_points_for_leg(pool, leg_id)
        .await
        .unwrap_or_default();
    let latest_location = latest_tracking_point_for_leg(pool, leg_id)
        .await
        .unwrap_or(None);
    let events = list_leg_events(pool, leg_id).await.unwrap_or_default();
    let documents = list_leg_documents(pool, leg_id).await.unwrap_or_default();
    let execution_notes = list_execution_note_history_for_leg(pool, leg_id)
        .await
        .unwrap_or_default();

    Ok(Json(ApiResponse::ok(build_execution_screen(
        &session,
        &leg,
        latest_location.as_ref(),
        tracking_points,
        events,
        documents,
        execution_notes,
    ))))
}

async fn run_leg_action(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionLegActionRequest>,
) -> Result<Json<ApiResponse<ExecutionLegActionResponse>>, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some(existing) = find_execution_leg_by_id(pool, leg_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !can_manage_execution(&session, &existing) {
        return Err(StatusCode::FORBIDDEN);
    }

    let existing_documents = list_leg_documents(pool, leg_id).await.unwrap_or_default();
    let has_delivery_pod = existing_documents
        .iter()
        .any(|document| document.r#type == "delivery_pod");

    if payload.action_key.trim() == "complete_delivery" && !has_delivery_pod {
        return Ok(Json(ApiResponse::ok(ExecutionLegActionResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: status_label_from_code(existing.status_id),
            message: "Upload a delivery POD document before completing delivery in the Rust execution flow.".into(),
        })));
    }

    if payload.action_key.trim() == "complete_delivery"
        && payload
            .note
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        return Ok(Json(ApiResponse::ok(ExecutionLegActionResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: status_label_from_code(existing.status_id),
            message: "Add a short delivery completion note before closing the leg in the Rust execution flow.".into(),
        })));
    }

    let Some((next_status, event_type, success_label)) =
        resolve_execution_action(existing.status_id, &payload.action_key)
    else {
        return Ok(Json(ApiResponse::ok(ExecutionLegActionResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: status_label_from_code(existing.status_id),
            message: "This execution action is not available for the leg's current Rust status."
                .into(),
        })));
    };

    let updated = advance_leg_execution(
        pool,
        leg_id,
        next_status,
        event_type,
        Some(session.user.id),
        payload.note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(updated_leg) = updated else {
        return Err(StatusCode::NOT_FOUND);
    };

    let summary = format!(
        "{} marked {} as {}.",
        session.user.name,
        updated_leg
            .leg_code
            .clone()
            .unwrap_or_else(|| format!("leg #{}", leg_id)),
        success_label
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::LegExecutionUpdated,
            leg_id: Some(leg_id.max(0) as u64),
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: updated_leg
                .booked_carrier_id
                .map(|value| value.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_user_ids(target_execution_user_ids(&updated_leg))
        .for_permission_keys(["manage_tracking", "access_admin_portal", "manage_loads"])
        .with_topics([
            RealtimeTopic::ExecutionTracking.as_key(),
            RealtimeTopic::LoadBoard.as_key(),
        ]),
    );

    Ok(Json(ApiResponse::ok(ExecutionLegActionResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        status_label: status_label_from_code(updated_leg.status_id),
        message: format!("Rust execution updated the leg to {}.", success_label),
    })))
}

async fn store_leg_location(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionLocationPingRequest>,
) -> Result<Json<ApiResponse<ExecutionLocationPingResponse>>, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some(existing) = find_execution_leg_by_id(pool, leg_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    if !can_manage_execution(&session, &existing) {
        return Err(StatusCode::FORBIDDEN);
    }

    if !can_send_location_ping(existing.status_id) {
        return Ok(Json(ApiResponse::ok(ExecutionLocationPingResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            latest_location_label: None,
            message: "Location ping is only accepted while the leg is in an active GPS-tracked execution state.".into(),
        })));
    }

    if !(-90.0..=90.0).contains(&payload.lat) || !(-180.0..=180.0).contains(&payload.lng) {
        return Ok(Json(ApiResponse::ok(ExecutionLocationPingResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            latest_location_label: None,
            message: "Latitude or longitude is outside valid Earth coordinate bounds.".into(),
        })));
    }

    let recorded_at = payload.recorded_at.as_deref().and_then(parse_recorded_at);

    let point = create_tracking_point(pool, leg_id, payload.lat, payload.lng, recorded_at)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let latest_label = format!(
        "{:.5}, {:.5} at {}",
        point.lat,
        point.lng,
        format_datetime(&point.recorded_at)
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::LegLocationUpdated,
            leg_id: Some(leg_id.max(0) as u64),
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: existing.booked_carrier_id.map(|value| value.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} sent a fresh location ping for {}.",
                session.user.name,
                existing
                    .leg_code
                    .clone()
                    .unwrap_or_else(|| format!("leg #{}", leg_id))
            ),
        })
        .for_user_ids(target_execution_user_ids(&existing))
        .for_permission_keys(["manage_tracking", "access_admin_portal", "manage_loads"])
        .with_topics([RealtimeTopic::ExecutionTracking.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ExecutionLocationPingResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        latest_location_label: Some(latest_label),
        message: "Rust tracking stored the latest location ping.".into(),
    })))
}

#[derive(Debug)]
struct ParsedUploadedDocument {
    document_name: String,
    document_type: String,
    original_name: String,
    mime_type: Option<String>,
    bytes: Vec<u8>,
}

async fn upload_leg_document(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Json<ApiResponse<ExecutionUploadDocumentResponse>> {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message:
                "Sign in before uploading execution documents from the Rust tracking workspace."
                    .into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: format!(
                "Execution document uploads are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(existing) = find_execution_leg_by_id(pool, leg_id)
        .await
        .unwrap_or_default()
    else {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: format!("Leg #{} was not found in the Rust tracking store.", leg_id),
        }));
    };

    if !can_upload_execution_documents(&session, &existing) {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: "Only the booked carrier or admin-scoped operators can attach execution documents for this leg in the current Rust slice.".into(),
        }));
    }

    let upload = match parse_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => {
            return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
                success: false,
                leg_id: leg_id.max(0) as u64,
                document_id: None,
                message,
            }));
        }
    };

    if !is_supported_execution_document_type(&upload.document_type) {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: "Execution document type must be one of pickup_bol, pickup_photo, delivery_pod, delivery_photo, or other.".into(),
        }));
    }

    let stored_file = match state
        .document_storage
        .save_execution_document(leg_id, &upload.original_name, &upload.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
                success: false,
                leg_id: leg_id.max(0) as u64,
                document_id: None,
                message: format!("Execution document upload storage failed: {}", error),
            }));
        }
    };

    let params = CreateLegDocumentParams {
        document_name: upload.document_name,
        document_type: upload.document_type,
        file_path: stored_file.file_path,
        storage_provider: stored_file.storage_provider,
        original_name: Some(upload.original_name),
        mime_type: upload.mime_type,
        file_size: Some(upload.bytes.len() as i64),
    };

    match create_leg_document(pool, leg_id, &params, Some(session.user.id)).await {
        Ok(Some(document)) => {
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::LegExecutionUpdated,
                    leg_id: Some(leg_id.max(0) as u64),
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: existing.booked_carrier_id.map(|value| value.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: format!(
                        "{} attached {} to {}.",
                        session.user.name,
                        params.document_name,
                        existing
                            .leg_code
                            .clone()
                            .unwrap_or_else(|| format!("leg #{}", leg_id))
                    ),
                })
                .for_user_ids(target_execution_user_ids(&existing))
                .for_permission_keys(["manage_tracking", "access_admin_portal", "manage_loads"])
                .with_topics([
                    RealtimeTopic::ExecutionTracking.as_key(),
                    RealtimeTopic::LoadBoard.as_key(),
                ]),
            );

            Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
                success: true,
                leg_id: leg_id.max(0) as u64,
                document_id: Some(document.id.max(0) as u64),
                message: format!(
                    "{} uploaded {}. Only admin users and the uploader can open the binary file in this Rust execution slice.",
                    session.user.name, params.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: "The target leg disappeared while saving the execution document.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: format!("Execution document create failed: {}", error),
        })),
    }
}

async fn download_leg_document_file(
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
            "Sign in before viewing execution document files from the Rust tracking workspace.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Execution document file access is unavailable because the database is disabled.",
        );
    };

    let Some(scope) = find_leg_document_scope(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        return text_response(
            StatusCode::NOT_FOUND,
            "The requested execution document was not found.",
        );
    };

    if !can_view_execution_document_file(&session, scope.uploaded_by_user_id) {
        return text_response(
            StatusCode::FORBIDDEN,
            "Only admin users and the profile that uploaded this execution document can view the file in the current Rust slice.",
        );
    }

    let Some(document) = find_leg_document_by_id(pool, document_id)
        .await
        .unwrap_or_default()
    else {
        return text_response(
            StatusCode::NOT_FOUND,
            "The requested execution document was not found.",
        );
    };

    let storage_provider = execution_document_storage_provider(&document);
    let bytes = match state
        .document_storage
        .read_document(&storage_provider, &document.path)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Execution document file could not be opened: {}", error),
            );
        }
    };

    let mime_type = execution_document_meta_string(&document, "mime_type")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "application/octet-stream".into());
    let file_name = execution_document_original_name(&document)
        .unwrap_or_else(|| execution_file_label(&document.path));
    let content_disposition = format!(
        "inline; filename=\"{}\"",
        sanitize_header_file_name(&file_name)
    );

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    response
}

fn build_execution_screen(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
    latest_location: Option<&db::tracking::LegLocationRecord>,
    tracking_points: Vec<db::tracking::LegLocationRecord>,
    events: Vec<db::tracking::LegEventRecord>,
    documents: Vec<db::tracking::LegDocumentRecord>,
    execution_notes: Vec<db::tracking::ExecutionNoteRecord>,
) -> ExecutionLegScreen {
    let can_manage = can_manage_execution(session, leg);
    let live_tracking_available =
        leg.booked_carrier_id == Some(session.user.id) && can_send_location_ping(leg.status_id);
    let can_upload_documents = can_upload_execution_documents(session, leg);
    let delivery_completion_ready = documents
        .iter()
        .any(|document| document.r#type == "delivery_pod");
    let action_items = execution_action_items(leg.status_id, can_manage, delivery_completion_ready);
    let route_label = format!(
        "{} -> {}",
        leg.pickup_location_name
            .clone()
            .unwrap_or_else(|| "Pickup TBD".into()),
        leg.delivery_location_name
            .clone()
            .unwrap_or_else(|| "Delivery TBD".into())
    );
    let (tracking_health_label, tracking_health_tone) =
        execution_tracking_health(latest_location, leg.status_id);
    let next_action_label = execution_next_action_label(leg.status_id, delivery_completion_ready);

    ExecutionLegScreen {
        title: format!(
            "Tracking {}",
            leg.leg_code.clone().unwrap_or_else(|| format!("Leg #{}", leg.leg_id))
        ),
        subtitle: format!(
            "Rust execution workspace for {} on load {}.",
            route_label,
            leg.load_number.clone().unwrap_or_else(|| format!("#{}", leg.load_id))
        ),
        leg_id: leg.leg_id.max(0) as u64,
        load_id: leg.load_id.max(0) as u64,
        load_number: leg.load_number.clone(),
        leg_code: leg.leg_code.clone().unwrap_or_else(|| format!("LEG-{}", leg.leg_id)),
        route_label,
        status_label: status_label_from_code(leg.status_id),
        status_tone: status_tone_from_code(leg.status_id).into(),
        carrier_label: leg.booked_carrier_name.clone(),
        operator_mode_label: execution_operator_mode_label(session, leg).into(),
        latest_location_label: latest_location.map(|point| format_datetime(&point.recorded_at)),
        latest_coordinate_label: latest_location.map(|point| format!("{:.5}, {:.5}", point.lat, point.lng)),
        latest_map_url: latest_location.map(|point| google_maps_url(point.lat, point.lng)),
        tracking_summary_label: Some(format!(
            "{} GPS ping(s) recorded for this leg.",
            tracking_points.len()
        )),
        tracking_health_label,
        tracking_health_tone: tracking_health_tone.into(),
        next_action_label,
        can_manage_execution: can_manage,
        can_send_location_ping: can_manage && can_send_location_ping(leg.status_id),
        live_tracking_available,
        live_tracking_note: Some(if live_tracking_available {
            "Driver view is active. Keep this page open to let the Rust workspace keep sending fresh GPS updates automatically.".into()
        } else if leg.booked_carrier_id == Some(session.user.id) {
            "Automatic GPS tracking unlocks only while this leg is in an active pickup, transit, or arrival state.".into()
        } else {
            "Live tracking is reserved for the booked carrier while this leg is in an active execution stage.".into()
        }),
        can_upload_documents,
        delivery_completion_ready,
        delivery_completion_note: Some(if delivery_completion_ready {
            "Delivery POD is attached, so the final completion step is unlocked.".into()
        } else {
            "Upload at least one Delivery POD document before using Complete delivery.".into()
        }),
        document_type_options: execution_document_type_options(),
        action_items,
        timeline: events
            .into_iter()
            .map(|event| ExecutionTimelineItem {
                id: event.id.max(0) as u64,
                event_type_key: event.r#type.clone(),
                event_type_label: event_label(&event.r#type),
                created_at_label: format_datetime(&event.created_at),
            })
            .collect(),
        notes_history: execution_notes
            .into_iter()
            .map(|item| ExecutionNoteItem {
                id: item.id.max(0) as u64,
                actor_label: item
                    .actor_name
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| {
                        item.admin_id
                            .map(|value| format!("User #{}", value))
                            .unwrap_or_else(|| "System".into())
                    }),
                status_label: status_label_from_code(item.status),
                remarks_label: item
                    .remarks
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "No execution note was captured.".into()),
                created_at_label: format_datetime(&item.created_at),
            })
            .collect(),
        tracking_points: tracking_points
            .into_iter()
            .enumerate()
            .map(|(index, point)| ExecutionTrackingPointItem {
                id: point.id.max(0) as u64,
                lat: point.lat,
                lng: point.lng,
                recorded_at_label: format_datetime(&point.recorded_at),
                is_latest: index == 0,
            })
            .collect(),
        documents: documents
            .into_iter()
            .map(|document| ExecutionDocumentItem {
                id: document.id.max(0) as u64,
                document_type_key: document.r#type.clone(),
                document_type_label: event_label(&document.r#type),
                file_label: execution_document_original_name(&document)
                    .unwrap_or_else(|| execution_file_label(&document.path)),
                source_path: document.path.clone(),
                download_path: can_view_execution_document_file(
                    session,
                    execution_document_uploaded_by_user_id(&document),
                )
                .then(|| execution_document_download_path(document.id.max(0) as u64)),
                uploaded_by_label: execution_document_uploaded_by_user_id(&document).map(|user_id| {
                    if user_id == session.user.id {
                        "Uploaded by you".to_string()
                    } else {
                        format!("Uploaded by user #{}", user_id)
                    }
                }),
                can_view_file: can_view_execution_document_file(
                    session,
                    execution_document_uploaded_by_user_id(&document),
                ),
                created_at_label: format_datetime(&document.created_at),
            })
            .collect(),
        notes: vec![
            "Execution actions follow the legacy pickup/depart/arrival/delivery sequence rather than allowing arbitrary jumps.".into(),
            "Location pings are currently accepted only while the leg is in an active GPS-tracked execution status.".into(),
            "Execution documents now support pickup BOL, pickup photos, delivery POD, delivery photos, and other attachments from the Rust tracking workspace.".into(),
            "Delivery completion is now guarded so the Rust flow requires a Delivery POD document plus a completion note before the leg can be marked delivered.".into(),
            "Admins, the load owner, and the booked carrier can view this screen, while state changes are limited to the booked carrier or admin-scoped operators.".into(),
        ],
    }
}

fn execution_tracking_health(
    latest_location: Option<&db::tracking::LegLocationRecord>,
    status_id: i16,
) -> (Option<String>, &'static str) {
    let is_active_tracking_status = can_send_location_ping(status_id);

    match latest_location {
        Some(point) if is_active_tracking_status => {
            let now = Utc::now().naive_utc();
            let age_minutes = (now - point.recorded_at).num_minutes().max(0);

            if age_minutes <= 15 {
                (
                    Some(format!("Tracking looks healthy. Latest GPS ping arrived {} minute(s) ago.", age_minutes)),
                    "success",
                )
            } else if age_minutes <= 45 {
                (
                    Some(format!(
                        "Tracking is still active, but the latest GPS ping is {} minute(s) old. A fresh update would help operations.",
                        age_minutes
                    )),
                    "warning",
                )
            } else {
                (
                    Some(format!(
                        "Tracking is stale. The latest GPS ping is {} minute(s) old while this leg is still in an active execution state.",
                        age_minutes
                    )),
                    "danger",
                )
            }
        }
        Some(point) => (
            Some(format!(
                "Execution is not in a live-tracking stage right now. The last recorded GPS ping was captured at {}.",
                format_datetime(&point.recorded_at)
            )),
            "info",
        ),
        None if is_active_tracking_status => (
            Some(
                "Live tracking is expected for the current execution stage, but the first GPS ping has not arrived yet."
                    .into(),
            ),
            "warning",
        ),
        None => (
            Some(
                "No GPS points have been recorded for this leg yet. Tracking will appear here after pickup or transit updates start."
                    .into(),
            ),
            "secondary",
        ),
    }
}

fn execution_next_action_label(status_id: i16, delivery_completion_ready: bool) -> Option<String> {
    let next_step = match LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(LegacyLoadLegStatusCode::Booked) => {
            "Next step: dispatch the driver and use Start pickup when the truck rolls toward origin."
        }
        Some(LegacyLoadLegStatusCode::PickupStarted) => {
            "Next step: mark Arrive at pickup once the truck reaches the shipper."
        }
        Some(LegacyLoadLegStatusCode::AtPickup) => {
            "Next step: confirm freight is loaded, then use Depart pickup to move the leg into transit."
        }
        Some(LegacyLoadLegStatusCode::InTransit) => {
            "Next step: keep live tracking running and use Arrive at delivery when the truck reaches destination."
        }
        Some(LegacyLoadLegStatusCode::EscrowFunded | LegacyLoadLegStatusCode::AtDelivery) => {
            if delivery_completion_ready {
                "Next step: add the closing delivery note and complete delivery from this Rust execution workspace."
            } else {
                "Next step: upload a Delivery POD document before attempting to complete delivery."
            }
        }
        Some(LegacyLoadLegStatusCode::Delivered) => {
            "Next step: execution is complete. Finance and archive follow-up now move back to the admin and closeout workflows."
        }
        Some(LegacyLoadLegStatusCode::PaidOut) => {
            "Next step: this leg is financially closed. Use the profile and desk pages only for audit follow-up."
        }
        _ => return None,
    };

    Some(next_step.into())
}

fn execution_operator_mode_label(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
) -> &'static str {
    if leg.booked_carrier_id == Some(session.user.id) {
        "Driver View"
    } else if session.user.primary_role() == Some(UserRole::Admin)
        || session.session.permissions.iter().any(|permission| {
            permission == "manage_tracking" || permission == "access_admin_portal"
        })
    {
        "Operations View"
    } else {
        "Customer Visibility"
    }
}

fn can_view_execution(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
) -> bool {
    session.user.primary_role() == Some(UserRole::Admin)
        || leg.load_owner_user_id == Some(session.user.id)
        || leg.booked_carrier_id == Some(session.user.id)
        || session.session.permissions.iter().any(|permission| {
            permission == "manage_tracking"
                || permission == "manage_loads"
                || permission == "access_admin_portal"
        })
}

fn can_manage_execution(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
) -> bool {
    session.user.primary_role() == Some(UserRole::Admin)
        || leg.booked_carrier_id == Some(session.user.id)
        || session.session.permissions.iter().any(|permission| {
            permission == "manage_tracking" || permission == "access_admin_portal"
        })
}

fn can_send_location_ping(status_id: i16) -> bool {
    matches!(status_id, 5 | 6 | 7 | 9)
}

fn can_upload_execution_documents(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
) -> bool {
    can_manage_execution(session, leg) && matches!(leg.status_id, 4 | 5 | 6 | 7 | 8 | 9 | 10)
}

fn execution_action_items(
    status_id: i16,
    can_manage: bool,
    delivery_completion_ready: bool,
) -> Vec<ExecutionActionItem> {
    [
        (
            "start_pickup",
            "Start pickup",
            "Carrier confirms the pickup workflow has started.",
            matches!(status_id, 4 | 8),
        ),
        (
            "arrive_pickup",
            "Arrive pickup",
            "Carrier arrived at pickup.",
            status_id == 5,
        ),
        (
            "depart_pickup",
            "Depart pickup",
            "Carrier departed pickup and is now in transit.",
            status_id == 6,
        ),
        (
            "arrive_delivery",
            "Arrive delivery",
            "Carrier arrived at delivery.",
            status_id == 7,
        ),
        (
            "complete_delivery",
            "Complete delivery",
            "Physical delivery is complete. A delivery POD and completion note are required before this final step unlocks.",
            status_id == 9 && delivery_completion_ready,
        ),
    ]
    .into_iter()
    .map(
        |(key, label, description, is_enabled)| ExecutionActionItem {
            key: key.into(),
            label: label.into(),
            description: description.into(),
            is_enabled: can_manage && is_enabled,
        },
    )
    .collect()
}

fn execution_document_type_options() -> Vec<ExecutionDocumentTypeOption> {
    vec![
        ExecutionDocumentTypeOption {
            key: "pickup_bol".into(),
            label: "Pickup BOL".into(),
            description: "Bill of lading or signed pickup paperwork.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "pickup_photo".into(),
            label: "Pickup Photos".into(),
            description: "Pickup condition or loading proof captured on site.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "delivery_pod".into(),
            label: "Delivery POD".into(),
            description: "Proof of delivery or signed receiving confirmation.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "delivery_photo".into(),
            label: "Delivery Photos".into(),
            description: "Delivery condition or unload proof captured on site.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "other".into(),
            label: "Other".into(),
            description: "Additional execution-stage attachment kept with the leg.".into(),
        },
    ]
}

fn is_supported_execution_document_type(value: &str) -> bool {
    matches!(
        value,
        "pickup_bol" | "pickup_photo" | "delivery_pod" | "delivery_photo" | "other"
    )
}

fn resolve_execution_action(
    current_status: i16,
    action_key: &str,
) -> Option<(LegacyLoadLegStatusCode, &'static str, &'static str)> {
    match action_key.trim() {
        "start_pickup" if matches!(current_status, 4 | 8) => Some((
            LegacyLoadLegStatusCode::PickupStarted,
            "pickup_started",
            "Pickup Started",
        )),
        "arrive_pickup" if current_status == 5 => Some((
            LegacyLoadLegStatusCode::AtPickup,
            "pickup_arrived",
            "At Pickup",
        )),
        "depart_pickup" if current_status == 6 => Some((
            LegacyLoadLegStatusCode::InTransit,
            "departed_pickup",
            "In Transit",
        )),
        "arrive_delivery" if current_status == 7 => Some((
            LegacyLoadLegStatusCode::AtDelivery,
            "delivery_arrived",
            "At Delivery",
        )),
        "complete_delivery" if current_status == 9 => {
            Some((LegacyLoadLegStatusCode::Delivered, "delivered", "Delivered"))
        }
        _ => None,
    }
}

fn target_execution_user_ids(leg: &db::tracking::ExecutionLegRecord) -> Vec<u64> {
    let mut users = Vec::new();
    if let Some(load_owner_user_id) = leg.load_owner_user_id {
        if load_owner_user_id > 0 {
            users.push(load_owner_user_id as u64);
        }
    }
    if let Some(booked_carrier_id) = leg.booked_carrier_id {
        if booked_carrier_id > 0 && !users.contains(&(booked_carrier_id as u64)) {
            users.push(booked_carrier_id as u64);
        }
    }
    users
}

fn status_label_from_code(status_id: i16) -> String {
    match LegacyLoadLegStatusCode::from_legacy_code(status_id) {
        Some(LegacyLoadLegStatusCode::Draft) => "Draft".into(),
        Some(LegacyLoadLegStatusCode::New) => "New".into(),
        Some(LegacyLoadLegStatusCode::Reviewed) => "Reviewed".into(),
        Some(LegacyLoadLegStatusCode::OfferReady) => "Offer Ready".into(),
        Some(LegacyLoadLegStatusCode::Booked) => "Booked".into(),
        Some(LegacyLoadLegStatusCode::PickupStarted) => "Pickup Started".into(),
        Some(LegacyLoadLegStatusCode::AtPickup) => "At Pickup".into(),
        Some(LegacyLoadLegStatusCode::InTransit) => "In Transit".into(),
        Some(LegacyLoadLegStatusCode::EscrowFunded) => "Escrow Funded".into(),
        Some(LegacyLoadLegStatusCode::AtDelivery) => "At Delivery".into(),
        Some(LegacyLoadLegStatusCode::Delivered) => "Delivered".into(),
        Some(LegacyLoadLegStatusCode::PaidOut) => "Paid Out".into(),
        None => format!("Status {}", status_id),
    }
}

fn status_tone_from_code(status_id: i16) -> &'static str {
    match status_id {
        8 => "success",
        5 | 6 => "primary",
        7 => "info",
        9 => "warning",
        10 | 11 => "success",
        4 => "secondary",
        _ => "secondary",
    }
}

fn event_label(value: &str) -> String {
    match value {
        "pickup_started" => "Pickup Started".into(),
        "pickup_arrived" => "Arrived At Pickup".into(),
        "departed_pickup" => "Departed Pickup".into(),
        "delivery_arrived" => "Arrived At Delivery".into(),
        "delivered" => "Delivery Completed".into(),
        "pickup_bol" => "Pickup BOL".into(),
        "pickup_photo" => "Pickup Photos".into(),
        "delivery_pod" => "Delivery POD".into(),
        "delivery_photo" => "Delivery Photos".into(),
        "document_uploaded" => "Execution Document Uploaded".into(),
        "location_ping" => "Location Ping".into(),
        other => other
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn format_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
}

fn parse_recorded_at(value: &str) -> Option<NaiveDateTime> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.naive_utc())
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
        .map_err(|error| format!("Execution document upload parsing failed: {}", error))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        match field_name.as_str() {
            "document_name" => {
                let value = field.text().await.map_err(|error| {
                    format!("Execution document name parsing failed: {}", error)
                })?;
                document_name = Some(value.trim().to_string());
            }
            "document_type" => {
                let value = field.text().await.map_err(|error| {
                    format!("Execution document type parsing failed: {}", error)
                })?;
                document_type = Some(value.trim().to_ascii_lowercase().replace([' ', '-'], "_"));
            }
            "file" => {
                original_name = field.file_name().map(str::to_string);
                mime_type = field.content_type().map(str::to_string);
                let payload = field.bytes().await.map_err(|error| {
                    format!("Execution document file parsing failed: {}", error)
                })?;
                bytes = Some(payload.to_vec());
            }
            _ => {}
        }
    }

    let document_type = document_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Choose a document type before uploading a file.".to_string())?;
    let document_name = document_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| event_label(&document_type));
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

fn can_view_execution_document_file(
    viewer: &crate::auth_session::ResolvedSession,
    uploaded_by_user_id: Option<i64>,
) -> bool {
    if viewer.user.primary_role() == Some(UserRole::Admin) {
        return true;
    }

    uploaded_by_user_id == Some(viewer.user.id)
}

fn execution_document_download_path(document_id: u64) -> String {
    format!("/execution/documents/{}/file", document_id)
}

fn execution_document_uploaded_by_user_id(
    document: &db::tracking::LegDocumentRecord,
) -> Option<i64> {
    document
        .meta
        .as_ref()
        .and_then(|meta| meta.get("uploaded_by"))
        .and_then(|value| value.as_i64())
}

fn execution_document_original_name(document: &db::tracking::LegDocumentRecord) -> Option<String> {
    execution_document_meta_string(document, "original_name")
}

fn execution_document_storage_provider(document: &db::tracking::LegDocumentRecord) -> String {
    execution_document_meta_string(document, "storage_provider")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| infer_document_storage_provider(&document.path))
}

fn execution_document_meta_string(
    document: &db::tracking::LegDocumentRecord,
    key: &str,
) -> Option<String> {
    document
        .meta
        .as_ref()
        .and_then(|meta| meta.get(key))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn execution_file_label(file_path: &str) -> String {
    file_path
        .rsplit('/')
        .next()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(file_path)
        .to_string()
}

fn infer_document_storage_provider(file_path: &str) -> String {
    let normalized = file_path.trim().to_ascii_lowercase();
    if normalized.starts_with("ibm-cos://") {
        "ibm_cos".into()
    } else if normalized.starts_with("s3://") {
        "s3".into()
    } else if normalized.starts_with("http://") || normalized.starts_with("https://") {
        "external_url".into()
    } else {
        "local".into()
    }
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

fn google_maps_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat:.5},{lng:.5}")
}
