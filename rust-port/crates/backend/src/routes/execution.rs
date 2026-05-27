use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose};
use std::{
    io::{Cursor, Write},
    time::Duration as StdDuration,
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use chrono::{NaiveDateTime, Utc};
use db::tracking::{
    CreateLegDocumentParams, active_customer_tracking_link, active_customer_tracking_link_by_token,
    active_tracking_consent_for_leg_user, advance_leg_execution,
    approve_execution_closeout_package, create_customer_tracking_link,
    create_execution_finance_exception, create_leg_document, create_tracking_point,
    current_route_plan, current_telematics_summary, decide_execution_finance_exceptions,
    execution_closeout_readiness, find_execution_leg_by_id, find_leg_document_by_id,
    find_leg_document_scope, insert_execution_offline_history_note, latest_tracking_point_for_leg,
    list_execution_finance_exception_summaries, list_execution_note_history_for_leg,
    list_execution_offline_submissions, list_leg_documents, list_leg_events,
    list_tracking_points_for_leg, mark_execution_offline_submission_failed,
    record_execution_offline_submission, record_telematics_execution_ping, record_tracking_consent,
    revoke_customer_tracking_links, upsert_execution_route_plan, upsert_telematics_connection,
};
use domain::{
    auth::UserRole,
    dispatch::LegacyLoadLegStatusCode,
    execution::{
        ExecutionTransitionContext, ExecutionTransitionError, execution_transition_for,
        is_trackable_execution_status,
    },
    tracking::{
        Coordinate, LegEventType, TrackingModuleContract, can_store_location_ping, haversine_km,
        is_inside_geofence, leg_event_types, tracking_module_contract,
    },
};
use serde::Serialize;
use shared::{
    ApiResponse, ExecutionActionItem, ExecutionCloseoutApprovalRequest,
    ExecutionCustomerTrackingLinkRequest, ExecutionCustomerTrackingLinkResponse,
    ExecutionCustomerTrackingRevokeRequest, ExecutionCustomerTrackingScreen, ExecutionDocumentItem,
    ExecutionDocumentTypeOption, ExecutionFinanceExceptionDecisionRequest,
    ExecutionFinanceExceptionRequest, ExecutionLegActionRequest, ExecutionLegActionResponse,
    ExecutionLegScreen, ExecutionLocationPingRequest, ExecutionLocationPingResponse,
    ExecutionNoteItem, ExecutionOfflineSubmissionRequest, ExecutionOfflineSubmissionResponse,
    ExecutionRoutePlanRequest, ExecutionStatusItem, ExecutionTelematicsConnectionRequest,
    ExecutionTelematicsPingRequest, ExecutionTimelineItem, ExecutionTrackingConsentRequest,
    ExecutionTrackingConsentResponse, ExecutionTrackingPointItem, ExecutionUploadDocumentResponse,
    ExecutionWorkflowMutationResponse, RealtimeEvent, RealtimeEventKind, RealtimeTopic,
    RequiredDocumentChecklistItem,
};

use crate::{
    auth_session, document_validation::validate_uploaded_document, rate_limit::RateLimitPolicy,
    realtime_bus::RoutedRealtimeEvent, state::AppState,
};

fn execution_document_policy(name: &'static str) -> RateLimitPolicy {
    RateLimitPolicy::new(name, 60, StdDuration::from_secs(60 * 60))
}

fn rate_limit_message(flow: &str, retry_after_seconds: u64) -> String {
    format!(
        "Too many {} attempts. Wait about {} seconds before trying again.",
        flow, retry_after_seconds
    )
}

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
        .route(
            "/customer-tracking/{share_token}",
            get(customer_tracking_screen),
        )
        .route("/legs/{leg_id}/actions", post(run_leg_action))
        .route(
            "/legs/{leg_id}/tracking-consent",
            post(capture_tracking_consent),
        )
        .route(
            "/legs/{leg_id}/offline-submissions",
            post(replay_offline_submission),
        )
        .route("/legs/{leg_id}/closeout", post(review_closeout_package))
        .route(
            "/legs/{leg_id}/closeout-package",
            get(download_closeout_package),
        )
        .route(
            "/legs/{leg_id}/finance-exceptions",
            post(create_finance_exception),
        )
        .route(
            "/legs/{leg_id}/finance-exceptions/decision",
            post(decide_finance_exception),
        )
        .route(
            "/legs/{leg_id}/customer-tracking",
            post(create_customer_tracking_share_link),
        )
        .route(
            "/legs/{leg_id}/customer-tracking/revoke",
            post(revoke_customer_tracking_share_links),
        )
        .route("/legs/{leg_id}/telematics", post(upsert_telematics_status))
        .route(
            "/legs/{leg_id}/telematics/ping",
            post(ingest_telematics_ping),
        )
        .route("/legs/{leg_id}/route-plan", post(upsert_route_plan))
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

async fn customer_tracking_screen(
    State(state): State<AppState>,
    Path(share_token): Path<String>,
) -> Result<Json<ApiResponse<ExecutionCustomerTrackingScreen>>, StatusCode> {
    let Some(pool) = state.pool.as_ref() else {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let Some(link) = active_customer_tracking_link_by_token(pool, &share_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let Some(leg) = find_execution_leg_by_id(pool, link.leg_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let latest_location = latest_tracking_point_for_leg(pool, link.leg_id)
        .await
        .unwrap_or(None);
    let (tracking_health_label, _) =
        execution_tracking_health(latest_location.as_ref(), leg.status_id);
    let (geofence_status_label, _, eta_risk_label, _) =
        execution_location_risk_labels(&leg, latest_location.as_ref());
    let route_label = format!(
        "{} -> {}",
        leg.pickup_location_name
            .clone()
            .unwrap_or_else(|| "Pickup".into()),
        leg.delivery_location_name
            .clone()
            .unwrap_or_else(|| "Delivery".into())
    );

    Ok(Json(ApiResponse::ok(ExecutionCustomerTrackingScreen {
        leg_code: leg
            .leg_code
            .unwrap_or_else(|| format!("LEG-{}", leg.leg_id)),
        load_number: leg.load_number,
        route_label,
        status_label: status_label_from_code(leg.status_id),
        latest_location_label: latest_location
            .as_ref()
            .map(|point| format_datetime(&point.recorded_at)),
        latest_coordinate_label: latest_location
            .as_ref()
            .map(|point| format!("{:.5}, {:.5}", point.lat, point.lng)),
        tracking_health_label,
        geofence_status_label,
        eta_risk_label,
        expires_at_label: format_datetime(&link.expires_at),
        visibility_scope_label: link.visibility_scope.replace('_', " "),
    })))
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
    let tracking_consent = active_tracking_consent_for_leg_user(pool, leg_id, session.user.id)
        .await
        .unwrap_or(None);
    let closeout = execution_closeout_readiness(pool, leg_id).await.ok();
    let offline_submissions = list_execution_offline_submissions(pool, leg_id)
        .await
        .unwrap_or_default();
    let finance_exceptions = list_execution_finance_exception_summaries(pool, leg_id)
        .await
        .unwrap_or_default();
    let customer_tracking_link = active_customer_tracking_link(pool, leg_id)
        .await
        .unwrap_or(None);
    let route_plan = current_route_plan(pool, leg_id).await.unwrap_or(None);
    let telematics = current_telematics_summary(pool, leg.booked_carrier_id)
        .await
        .unwrap_or(None);

    Ok(Json(ApiResponse::ok(build_execution_screen(
        &session,
        &leg,
        latest_location.as_ref(),
        tracking_consent.as_ref(),
        closeout.as_ref(),
        &offline_submissions,
        &finance_exceptions,
        customer_tracking_link.as_ref(),
        route_plan.as_ref(),
        telematics.as_ref(),
        tracking_points,
        events,
        documents,
        execution_notes,
    ))))
}

async fn replay_offline_submission(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionOfflineSubmissionRequest>,
) -> Result<Json<ApiResponse<ExecutionOfflineSubmissionResponse>>, StatusCode> {
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

    let captured_at = payload.captured_at.as_deref().and_then(parse_recorded_at);
    let submission = record_execution_offline_submission(
        pool,
        leg_id,
        Some(session.user.id),
        &payload.submission_type,
        &payload.client_submission_id,
        &payload.payload,
        captured_at,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if submission.processing_status != "duplicate" {
        let replay_result = match payload.submission_type.as_str() {
            "driver_note" => {
                let note = payload
                    .payload
                    .get("note")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("Offline driver note replayed without text.");
                insert_execution_offline_history_note(
                    pool,
                    leg_id,
                    Some(session.user.id),
                    existing.status_id,
                    note,
                )
                .await
                .map_err(|error| error.to_string())
            }
            "driver_action" => {
                replay_offline_driver_action(pool, &session, leg_id, &existing, &payload.payload)
                    .await
            }
            "gps_ping" => {
                replay_offline_gps_ping(pool, leg_id, &payload.payload, captured_at).await
            }
            "document_upload" => {
                replay_offline_document_upload(&state, pool, leg_id, &payload.payload, &session)
                    .await
            }
            _ => Ok(()),
        };

        if let Err(error) = replay_result {
            let note = format!("Offline replay failed: {}", error);
            mark_execution_offline_submission_failed(pool, submission.id, &note)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(ApiResponse::ok(ExecutionOfflineSubmissionResponse {
                success: false,
                leg_id: leg_id.max(0) as u64,
                submission_id: Some(submission.id.max(0) as u64),
                status_label: "failed".into(),
                message: note,
            })));
        }
    }

    Ok(Json(ApiResponse::ok(ExecutionOfflineSubmissionResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        submission_id: Some(submission.id.max(0) as u64),
        status_label: submission.processing_status.replace('_', " "),
        message: submission
            .reconciliation_note
            .unwrap_or_else(|| "Offline submission replayed into Rust execution.".into()),
    })))
}

async fn replay_offline_driver_action(
    pool: &db::DbPool,
    session: &auth_session::ResolvedSession,
    leg_id: i64,
    existing: &db::tracking::ExecutionLegRecord,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let action_key = payload
        .get("action_key")
        .or_else(|| payload.get("actionKey"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Offline driver action is missing action_key.".to_string())?;
    let note = payload
        .get("note")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let existing_documents = list_leg_documents(pool, leg_id)
        .await
        .map_err(|error| error.to_string())?;
    let has_delivery_pod = existing_documents
        .iter()
        .any(|document| document.r#type == "delivery_pod");
    let has_tracking_consent = active_tracking_consent_for_leg_user(pool, leg_id, session.user.id)
        .await
        .map_err(|error| error.to_string())?
        .is_some();
    let transition = execution_transition_for(
        existing.status_id,
        action_key,
        ExecutionTransitionContext {
            has_delivery_pod,
            has_completion_note: note.is_some(),
            has_tracking_consent,
        },
    )
    .map_err(execution_transition_error_message)?;

    advance_leg_execution(
        pool,
        leg_id,
        transition.to,
        transition.event_type,
        Some(session.user.id),
        note,
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn replay_offline_gps_ping(
    pool: &db::DbPool,
    leg_id: i64,
    payload: &serde_json::Value,
    captured_at: Option<NaiveDateTime>,
) -> Result<(), String> {
    let lat = payload
        .get("lat")
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "Offline GPS ping is missing latitude.".to_string())?;
    let lng = payload
        .get("lng")
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "Offline GPS ping is missing longitude.".to_string())?;
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
        return Err("Offline GPS ping had invalid coordinates.".into());
    }

    create_tracking_point(pool, leg_id, lat, lng, captured_at)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn replay_offline_document_upload(
    state: &AppState,
    pool: &db::DbPool,
    leg_id: i64,
    payload: &serde_json::Value,
    session: &auth_session::ResolvedSession,
) -> Result<(), String> {
    let document_type = payload
        .get("document_type")
        .or_else(|| payload.get("documentType"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Offline document upload is missing document_type.".to_string())?;
    if !is_supported_execution_document_type(document_type) {
        return Err("Offline document upload had an unsupported document_type.".into());
    }

    let document_name = payload
        .get("document_name")
        .or_else(|| payload.get("documentName"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(document_type);
    let original_name = payload
        .get("file_name")
        .or_else(|| payload.get("fileName"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("offline-document.bin");
    let mime_type = payload
        .get("mime_type")
        .or_else(|| payload.get("mimeType"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let raw_base64 = payload
        .get("bytes_base64")
        .or_else(|| payload.get("bytesBase64"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Offline document upload is missing file bytes.".to_string())?;
    let clean_base64 = raw_base64
        .split_once(',')
        .map(|(_, encoded)| encoded)
        .unwrap_or(raw_base64);
    let bytes = general_purpose::STANDARD
        .decode(clean_base64)
        .map_err(|error| format!("Offline document upload bytes could not be decoded: {error}"))?;

    validate_uploaded_document(original_name, mime_type, &bytes)
        .map_err(|error| error.to_string())?;
    let stored = state
        .document_storage
        .save_execution_document(leg_id, original_name, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    create_leg_document(
        pool,
        leg_id,
        &CreateLegDocumentParams {
            document_name: document_name.into(),
            document_type: document_type.into(),
            file_path: stored.file_path,
            storage_provider: stored.storage_provider,
            original_name: Some(original_name.into()),
            mime_type: mime_type.map(str::to_string),
            file_size: Some(bytes.len() as i64),
        },
        Some(session.user.id),
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn review_closeout_package(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionCloseoutApprovalRequest>,
) -> Result<Json<ApiResponse<ExecutionWorkflowMutationResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    let status = normalize_closeout_review_status(&payload.pod_review_status);
    let current_readiness = execution_closeout_readiness(pool, leg_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if status == "approved"
        && let Some(blocker) = closeout_approval_blocker(&current_readiness)
    {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Closeout blocked".into(),
            message: blocker,
        })));
    }

    let readiness = approve_execution_closeout_package(
        pool,
        leg_id,
        Some(session.user.id),
        status,
        payload.export_path.as_deref(),
        payload.note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    publish_execution_update(
        &state,
        &session,
        &leg,
        format!(
            "{} updated closeout review for leg #{} to {}.",
            session.user.name, leg_id, status
        ),
    );

    Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        status_label: status.replace('_', " "),
        message: execution_closeout_package_label(&readiness),
    })))
}

async fn create_finance_exception(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionFinanceExceptionRequest>,
) -> Result<Json<ApiResponse<ExecutionWorkflowMutationResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    let description = payload.description.trim();
    if description.is_empty() {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Missing description".into(),
            message: "Claim, detention, or accessorial requests require a description.".into(),
        })));
    }

    let exception_id = create_execution_finance_exception(
        pool,
        leg_id,
        Some(session.user.id),
        &normalize_finance_exception_type(&payload.exception_type),
        &normalize_finance_exception_status(&payload.status),
        payload.amount_cents,
        &normalize_visibility(&payload.visibility),
        description,
        payload.evidence_document_id.map(|value| value as i64),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    publish_execution_update(
        &state,
        &session,
        &leg,
        format!(
            "{} recorded finance exception #{} for leg #{}.",
            session.user.name, exception_id, leg_id
        ),
    );

    Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        status_label: "Recorded".into(),
        message: format!(
            "Finance exception #{} is recorded and visible in closeout/payment readiness.",
            exception_id
        ),
    })))
}

async fn decide_finance_exception(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionFinanceExceptionDecisionRequest>,
) -> Result<Json<ApiResponse<ExecutionWorkflowMutationResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg)
        && !session
            .session
            .permissions
            .iter()
            .any(|permission| permission == "manage_payments")
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let exception_type = normalize_finance_exception_type(&payload.exception_type);
    let status = normalize_finance_exception_status(&payload.status);
    let updated = decide_execution_finance_exceptions(
        pool,
        leg_id,
        Some(session.user.id),
        &exception_type,
        &status,
        payload.resolution_note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if updated > 0 {
        publish_execution_update(
            &state,
            &session,
            &leg,
            format!(
                "{} marked {} finance exception(s) for leg #{} as {}.",
                session.user.name, updated, leg_id, status
            ),
        );
    }

    Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
        success: updated > 0,
        leg_id: leg_id.max(0) as u64,
        status_label: if updated > 0 {
            status.replace('_', " ")
        } else {
            "No open exception".into()
        },
        message: if updated > 0 {
            format!(
                "{} {} exception(s) updated to {}; invoice, settlement, closeout, and support timeline context was recorded.",
                updated, exception_type, status
            )
        } else {
            format!(
                "No open {} finance exception was found for this leg.",
                exception_type
            )
        },
    })))
}

async fn create_customer_tracking_share_link(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionCustomerTrackingLinkRequest>,
) -> Result<Json<ApiResponse<ExecutionCustomerTrackingLinkResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    let visibility_scope = normalize_customer_tracking_scope(&payload.visibility_scope);
    let expires_in_hours = payload.expires_in_hours.unwrap_or(168).clamp(1, 24 * 30);
    let expires_at = Utc::now().naive_utc() + chrono::Duration::hours(expires_in_hours);
    let share_token = format!("trk_{}", uuid::Uuid::new_v4().simple());
    let Some(link) = create_customer_tracking_link(
        pool,
        leg_id,
        Some(session.user.id),
        &share_token,
        visibility_scope,
        expires_at,
        payload.rotate_existing,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    publish_execution_update(
        &state,
        &session,
        &leg,
        format!(
            "{} created a customer tracking link for leg #{}.",
            session.user.name, leg_id
        ),
    );

    Ok(Json(ApiResponse::ok(
        ExecutionCustomerTrackingLinkResponse {
            success: true,
            leg_id: leg_id.max(0) as u64,
            customer_tracking_path: Some(format!("/track/{}", link.share_token)),
            expires_at_label: Some(format_datetime(&link.expires_at)),
            status_label: "Active".into(),
            message: format!(
                "Customer tracking link is active until {} with {} visibility.",
                format_datetime(&link.expires_at),
                link.visibility_scope.replace('_', " ")
            ),
        },
    )))
}

async fn revoke_customer_tracking_share_links(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionCustomerTrackingRevokeRequest>,
) -> Result<Json<ApiResponse<ExecutionCustomerTrackingLinkResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    let revoked = revoke_customer_tracking_links(
        pool,
        leg_id,
        Some(session.user.id),
        payload.reason.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if revoked > 0 {
        publish_execution_update(
            &state,
            &session,
            &leg,
            format!(
                "{} revoked customer tracking access for leg #{}.",
                session.user.name, leg_id
            ),
        );
    }

    Ok(Json(ApiResponse::ok(
        ExecutionCustomerTrackingLinkResponse {
            success: true,
            leg_id: leg_id.max(0) as u64,
            customer_tracking_path: None,
            expires_at_label: None,
            status_label: if revoked > 0 {
                "Revoked".into()
            } else {
                "No active link".into()
            },
            message: if revoked > 0 {
                format!("Revoked {} active customer tracking link(s).", revoked)
            } else {
                "No active customer tracking link was found for this leg.".into()
            },
        },
    )))
}

async fn upsert_telematics_status(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionTelematicsConnectionRequest>,
) -> Result<Json<ApiResponse<ExecutionWorkflowMutationResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    if leg.booked_carrier_id.is_none() {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "No carrier".into(),
            message: "Telematics provider status requires a booked carrier.".into(),
        })));
    }

    upsert_telematics_connection(
        pool,
        leg.booked_carrier_id,
        &payload.provider_key,
        &payload.status,
        &payload.fallback_behavior,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        status_label: payload.status.trim().replace('_', " "),
        message: "Telematics provider decision/status is saved for this carrier.".into(),
    })))
}

async fn ingest_telematics_ping(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionTelematicsPingRequest>,
) -> Result<Json<ApiResponse<ExecutionWorkflowMutationResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg)
        && !session
            .session
            .permissions
            .iter()
            .any(|permission| permission == "manage_tms_operations")
    {
        return Err(StatusCode::FORBIDDEN);
    }

    if let (Some(lat), Some(lng)) = (payload.lat, payload.lng)
        && (!(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng))
    {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Invalid coordinates".into(),
            message: "Telematics ping latitude or longitude is outside valid bounds.".into(),
        })));
    }

    let provider = payload.provider_key.trim();
    if provider.is_empty() {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Missing provider".into(),
            message: "Telematics ping requires a provider key.".into(),
        })));
    }

    let recorded_at = payload.recorded_at.as_deref().and_then(parse_recorded_at);
    record_telematics_execution_ping(
        pool,
        leg_id,
        provider,
        payload.lat,
        payload.lng,
        recorded_at,
        payload.event_type.as_deref(),
        payload.hos_status.as_deref(),
        payload.truck_id.as_deref(),
        payload.trailer_id.as_deref(),
        Some(session.user.id),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    publish_execution_update(
        &state,
        &session,
        &leg,
        format!(
            "{} ingested {} telematics event for leg #{}.",
            session.user.name, provider, leg_id
        ),
    );

    Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        status_label: "Telematics ingested".into(),
        message:
            "Normalized telematics ping was recorded into location, event, and execution history."
                .into(),
    })))
}

async fn upsert_route_plan(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionRoutePlanRequest>,
) -> Result<Json<ApiResponse<ExecutionWorkflowMutationResponse>>, StatusCode> {
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

    if !can_manage_execution(&session, &leg) {
        return Err(StatusCode::FORBIDDEN);
    }

    if payload.provider_key.trim().is_empty() {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Missing provider".into(),
            message: "Route plan requires a provider key such as manual, truck_safe_provider, or a contracted routing provider.".into(),
        })));
    }
    if payload
        .distance_miles
        .is_some_and(|value| !(0.0..=10000.0).contains(&value))
    {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Invalid mileage".into(),
            message:
                "Route mileage must be greater than zero and within a realistic freight range."
                    .into(),
        })));
    }
    if payload
        .duration_minutes
        .is_some_and(|value| !(1..=60 * 24 * 30).contains(&value))
    {
        return Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            status_label: "Invalid duration".into(),
            message:
                "Route duration must be at least one minute and within a realistic freight range."
                    .into(),
        })));
    }

    upsert_execution_route_plan(
        pool,
        leg_id,
        Some(session.user.id),
        &payload.provider_key,
        payload.distance_miles,
        payload.duration_minutes,
        payload.truck_safe,
        &payload.status,
        &payload.constraints,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::ok(ExecutionWorkflowMutationResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        status_label: payload.status.trim().replace('_', " "),
        message: "Route plan source is saved with truck-safe constraints and a history entry for ETA, pricing, exception, and settlement readiness.".into(),
    })))
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
    let has_completion_note = payload
        .note
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let has_tracking_consent = active_tracking_consent_for_leg_user(pool, leg_id, session.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    let transition = match execution_transition_for(
        existing.status_id,
        &payload.action_key,
        ExecutionTransitionContext {
            has_delivery_pod,
            has_completion_note,
            has_tracking_consent,
        },
    ) {
        Ok(value) => value,
        Err(error) => {
            return Ok(Json(ApiResponse::ok(ExecutionLegActionResponse {
                success: false,
                leg_id: leg_id.max(0) as u64,
                status_label: status_label_from_code(existing.status_id),
                message: execution_transition_error_message(error),
            })));
        }
    };

    let updated = advance_leg_execution(
        pool,
        leg_id,
        transition.to,
        transition.event_type,
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
        transition.success_label
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
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
        message: format!(
            "Rust execution updated the leg to {}.",
            transition.success_label
        ),
    })))
}

async fn capture_tracking_consent(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ExecutionTrackingConsentRequest>,
) -> Result<Json<ApiResponse<ExecutionTrackingConsentResponse>>, StatusCode> {
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

    let consent_text = payload.consent_text.trim();
    if consent_text.len() < 20 {
        return Ok(Json(ApiResponse::ok(ExecutionTrackingConsentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            message: "Tracking consent text is required before GPS tracking can start.".into(),
        })));
    }

    record_tracking_consent(pool, leg_id, session.user.id, consent_text)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
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
                "{} accepted tracking consent for leg #{}.",
                session.user.name, leg_id
            ),
        })
        .for_user_ids(target_execution_user_ids(&existing))
        .for_permission_keys(["manage_tracking", "access_admin_portal", "manage_loads"])
        .with_topics([RealtimeTopic::ExecutionTracking.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ExecutionTrackingConsentResponse {
        success: true,
        leg_id: leg_id.max(0) as u64,
        message: "Tracking consent is recorded. GPS and pickup start actions are now unlocked."
            .into(),
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

    let has_tracking_consent = active_tracking_consent_for_leg_user(pool, leg_id, session.user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();

    if !can_store_location_ping(existing.status_id, has_tracking_consent) {
        return Ok(Json(ApiResponse::ok(ExecutionLocationPingResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            latest_location_label: None,
            message:
                "Location ping requires active execution status and recorded tracking consent."
                    .into(),
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
            request_id: None,
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
    if state.config.kill_switch_document_uploads {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: "Document uploads are temporarily disabled by an operational kill switch."
                .into(),
        }));
    }

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

    let rate_decision = state
        .check_rate_limit(
            execution_document_policy("execution_document_upload"),
            format!("{}:{}", session.user.id, leg_id),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(ExecutionUploadDocumentResponse {
            success: false,
            leg_id: leg_id.max(0) as u64,
            document_id: None,
            message: rate_limit_message(
                "execution document upload",
                rate_decision.retry_after_seconds,
            ),
        }));
    }

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
                    request_id: None,
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

async fn download_closeout_package(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before exporting the closeout package.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Closeout export is unavailable while database access is offline.",
        );
    };

    let Some(leg) = find_execution_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return text_response(StatusCode::NOT_FOUND, "Leg was not found.");
    };

    if !can_view_execution(&session, &leg) {
        return text_response(
            StatusCode::FORBIDDEN,
            "This session cannot export closeout for the leg.",
        );
    }

    let documents = list_leg_documents(pool, leg_id).await.unwrap_or_default();
    let closeout = execution_closeout_readiness(pool, leg_id)
        .await
        .unwrap_or_else(|_| default_closeout_readiness(leg_id, false));
    let exceptions = list_execution_finance_exception_summaries(pool, leg_id)
        .await
        .unwrap_or_default();

    let mut body = String::new();
    body.push_str("STLoads Closeout Package\n");
    body.push_str("========================\n");
    body.push_str(&format!(
        "Leg: {}\nLoad: {}\nRoute: {} -> {}\nStatus: {}\nCloseout: {}\nGenerated: {}\n\n",
        leg.leg_code
            .clone()
            .unwrap_or_else(|| format!("LEG-{}", leg_id)),
        leg.load_number
            .clone()
            .unwrap_or_else(|| format!("#{}", leg.load_id)),
        leg.pickup_location_name
            .clone()
            .unwrap_or_else(|| "Pickup".into()),
        leg.delivery_location_name
            .clone()
            .unwrap_or_else(|| "Delivery".into()),
        status_label_from_code(leg.status_id),
        execution_closeout_package_label(&closeout),
        format_datetime(&Utc::now().naive_utc())
    ));
    body.push_str("Closeout checklist\n");
    for item in execution_closeout_checklist(&closeout) {
        body.push_str(&format!(
            "- {}: {} - {}\n",
            item.label, item.status_label, item.detail
        ));
    }
    body.push_str("\nRequired artifacts manifest\n");
    body.push_str("- Delivery POD: required for release approval\n");
    body.push_str("- POD review: approved or accepted before payment release\n");
    body.push_str("- Claims/accessorials: no open pending, disputed, or review items\n");
    body.push_str("- Offline replay: no pending, received, or failed submissions\n\n");
    body.push_str("Documents\n");
    let mut document_entries = Vec::new();
    let mut document_warnings = Vec::new();
    for document in &documents {
        let document_name = execution_document_original_name(document)
            .unwrap_or_else(|| execution_file_label(&document.path));
        body.push_str(&format!(
            "- {}: {} ({}) source={} secure_link=/execution/documents/{}/file\n",
            event_label(&document.r#type),
            document_name,
            document_version_label(document.current_version, document.version_count),
            document.path,
            document.id
        ));

        let storage_provider = execution_document_storage_provider(document);
        match state
            .document_storage
            .read_document(&storage_provider, &document.path)
            .await
        {
            Ok(bytes) => {
                document_entries.push((
                    closeout_document_zip_name(document.id, &document_name),
                    bytes,
                ));
            }
            Err(error) if document.r#type == "delivery_pod" => {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!(
                        "Closeout package cannot be generated because required delivery POD '{}' could not be read: {}",
                        document_name, error
                    ),
                );
            }
            Err(error) => {
                let warning = format!(
                    "{} ({}) could not be embedded: {}",
                    document_name,
                    event_label(&document.r#type),
                    error
                );
                document_warnings.push(warning.clone());
                document_entries.push((
                    closeout_document_zip_name(
                        document.id,
                        &format!("{document_name}-READ-ERROR.txt"),
                    ),
                    warning.into_bytes(),
                ));
            }
        }
    }
    if !document_warnings.is_empty() {
        body.push_str("\nDocument embed warnings\n");
        for warning in document_warnings {
            body.push_str(&format!("- {warning}\n"));
        }
    }
    body.push_str("\nClaims, detention, and accessorials\n");
    if exceptions.is_empty() {
        body.push_str("- None recorded\n");
    } else {
        for item in exceptions {
            body.push_str(&format!(
                "- {} {} x{} amount {}\n",
                item.exception_type,
                item.status,
                item.count,
                item.amount_cents
                    .map(|value| format!("${:.2}", value as f64 / 100.0))
                    .unwrap_or_else(|| "not set".into())
            ));
        }
    }

    let package_bytes = match build_closeout_zip_package(&body, document_entries) {
        Ok(bytes) => bytes,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Closeout package generation failed: {error}"),
            );
        }
    };

    let mut response = Response::new(Body::from(package_bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    if let Ok(value) = HeaderValue::from_str(&format!(
        "attachment; filename=\"stloads-closeout-leg-{}.zip\"",
        leg_id
    )) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    response
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

    let rate_decision = state
        .check_rate_limit(
            execution_document_policy("execution_document_read"),
            format!("{}:{}", session.user.id, document_id),
        )
        .await;
    if !rate_decision.allowed {
        return text_response(
            StatusCode::TOO_MANY_REQUESTS,
            &rate_limit_message("execution document read", rate_decision.retry_after_seconds),
        );
    }

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

// Execution screens aggregate many independently loaded operational panels.
// A wide signature keeps the data dependencies visible until this becomes a
// dedicated read model.
#[allow(clippy::too_many_arguments)]
fn build_execution_screen(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
    latest_location: Option<&db::tracking::LegLocationRecord>,
    tracking_consent: Option<&db::tracking::ExecutionTrackingConsentRecord>,
    closeout: Option<&db::tracking::ExecutionCloseoutReadinessRecord>,
    offline_submissions: &[db::tracking::ExecutionOfflineSubmissionRecord],
    finance_exceptions: &[db::tracking::ExecutionFinanceExceptionSummaryRecord],
    customer_tracking_link: Option<&db::tracking::ExecutionCustomerTrackingLinkRecord>,
    route_plan: Option<&db::tracking::ExecutionRoutePlanRecord>,
    telematics: Option<&db::tracking::ExecutionTelematicsSummaryRecord>,
    tracking_points: Vec<db::tracking::LegLocationRecord>,
    events: Vec<db::tracking::LegEventRecord>,
    documents: Vec<db::tracking::LegDocumentRecord>,
    execution_notes: Vec<db::tracking::ExecutionNoteRecord>,
) -> ExecutionLegScreen {
    let can_manage = can_manage_execution(session, leg);
    let tracking_consent_granted = tracking_consent.is_some();
    let live_tracking_available = leg.booked_carrier_id == Some(session.user.id)
        && can_store_location_ping(leg.status_id, tracking_consent_granted);
    let can_upload_documents = can_upload_execution_documents(session, leg);
    let delivery_completion_ready = documents
        .iter()
        .any(|document| document.r#type == "delivery_pod");
    let required_documents = execution_required_document_checklist(delivery_completion_ready);
    let action_items = execution_action_items(
        leg.status_id,
        can_manage,
        delivery_completion_ready,
        tracking_consent_granted,
    );
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
    let (geofence_status_label, geofence_status_tone, eta_risk_label, eta_risk_tone) =
        execution_location_risk_labels(leg, latest_location);
    let closeout = closeout
        .cloned()
        .unwrap_or_else(|| default_closeout_readiness(leg.leg_id, delivery_completion_ready));
    let closeout_ready = execution_closeout_ready(&closeout);
    let pending_offline_submission_count = offline_submissions
        .iter()
        .filter(|item| {
            matches!(
                item.processing_status.as_str(),
                "received" | "pending" | "failed"
            )
        })
        .count() as u64;

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
        tracking_consent_required: leg.booked_carrier_id == Some(session.user.id),
        tracking_consent_granted,
        tracking_consent_text: execution_tracking_consent_text(leg),
        tracking_retention_label: tracking_consent
            .map(|consent| {
                format!(
                    "Location data for this leg is retained for {} day(s) unless legal hold or customer contract requires otherwise.",
                    consent.retention_days
                )
            })
            .unwrap_or_else(|| {
                "Location data retention target is 90 days for active execution tracking.".into()
            }),
        customer_tracking_scope_label: tracking_consent
            .map(|consent| consent.customer_visible_scope.replace('_', " "))
            .unwrap_or_else(|| "latest location and status after consent".into()),
        field_capture_strategy_label:
            "Camera-first web capture for BOL, POD, seals, damage, accessorial evidence, and stop photos."
                .into(),
        offline_strategy_label:
            "PWA-first offline queue: actions, notes, GPS pings, and uploads must be marked pending until reconciled by the API."
                .into(),
        mobile_support_label:
            "Supported target: current Chrome on Android and Safari on iOS; native app remains a later go/no-go decision."
                .into(),
        geofence_status_label,
        geofence_status_tone: geofence_status_tone.into(),
        eta_risk_label,
        eta_risk_tone: eta_risk_tone.into(),
        closeout_ready,
        closeout_package_label: execution_closeout_package_label(&closeout),
        closeout_package_tone: if closeout_ready { "success" } else { "warning" }.into(),
        closeout_export_path: closeout.export_path.clone(),
        customer_tracking_path: customer_tracking_link
            .map(|link| format!("/track/{}", link.share_token)),
        offline_submission_count: offline_submissions.len() as u64,
        pending_offline_submission_count,
        offline_submission_status_label: if pending_offline_submission_count == 0 {
            "Offline queue clear".into()
        } else {
            format!("{} offline submission(s) need reconciliation", pending_offline_submission_count)
        },
        telematics_status_label: execution_telematics_label(telematics),
        route_plan_label: execution_route_plan_label(route_plan),
        route_plan_tone: if route_plan.is_some() { "success" } else { "warning" }.into(),
        closeout_checklist: execution_closeout_checklist(&closeout),
        claims_accessorial_items: execution_claims_accessorial_items(finance_exceptions),
        next_action_label,
        can_manage_execution: can_manage,
        can_send_location_ping: can_manage && can_store_location_ping(leg.status_id, tracking_consent_granted),
        live_tracking_available,
        live_tracking_note: Some(if live_tracking_available {
            "Driver view is active. Keep this page open to let the Rust workspace keep sending fresh GPS updates automatically.".into()
        } else if leg.booked_carrier_id == Some(session.user.id) && !tracking_consent_granted {
            "Tracking consent is required before live GPS or pickup start actions unlock.".into()
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
                current_version: document.current_version.max(1) as u32,
                version_count: document.version_count.max(1) as u64,
                version_history_label: document_version_label(
                    document.current_version,
                    document.version_count,
                ),
            })
            .collect(),
        required_documents,
        notes: vec![
            "Execution actions now use the central domain transition state machine rather than route-local status jumps.".into(),
            "Location pings require both an active GPS-tracked execution status and recorded tracking consent.".into(),
            "Execution documents now support pickup BOL, pickup photos, delivery POD, delivery photos, and other attachments from the Rust tracking workspace.".into(),
            "Mobile capture is web/PWA first with camera input, pending offline reconciliation, and explicit supported-browser targets.".into(),
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

fn execution_location_risk_labels(
    leg: &db::tracking::ExecutionLegRecord,
    latest_location: Option<&db::tracking::LegLocationRecord>,
) -> (Option<String>, &'static str, Option<String>, &'static str) {
    let Some(point) = latest_location else {
        return (
            Some("No GPS ping is available for geofence or ETA risk scoring yet.".into()),
            "warning",
            Some("ETA risk unknown until the first mobile location ping is captured.".into()),
            "warning",
        );
    };

    let current = point.coordinate();
    let pickup = leg
        .pickup_latitude
        .zip(leg.pickup_longitude)
        .map(|(lat, lng)| Coordinate { lat, lng });
    let delivery = leg
        .delivery_latitude
        .zip(leg.delivery_longitude)
        .map(|(lat, lng)| Coordinate { lat, lng });

    let target = match LegacyLoadLegStatusCode::from_legacy_code(leg.status_id) {
        Some(LegacyLoadLegStatusCode::PickupStarted) | Some(LegacyLoadLegStatusCode::AtPickup) => {
            pickup.map(|coordinate| ("pickup", coordinate, leg.pickup_date))
        }
        Some(LegacyLoadLegStatusCode::InTransit) | Some(LegacyLoadLegStatusCode::AtDelivery) => {
            delivery.map(|coordinate| ("delivery", coordinate, leg.delivery_date))
        }
        _ => None,
    };

    let Some((stop_label, stop_coordinate, appointment_at)) = target else {
        return (
            Some(
                "Geofence monitoring activates during pickup, transit, and delivery stages.".into(),
            ),
            "info",
            Some("ETA risk is idle outside active execution movement.".into()),
            "info",
        );
    };

    let distance_km = haversine_km(current, stop_coordinate);
    let geofence = if is_inside_geofence(current, stop_coordinate, 0.5) {
        (
            Some(format!("Inside the {} geofence within 0.5 km.", stop_label)),
            "success",
        )
    } else {
        (
            Some(format!(
                "{:.1} km from the {} geofence. Keep monitoring route progress.",
                distance_km, stop_label
            )),
            if distance_km <= 20.0 {
                "warning"
            } else {
                "info"
            },
        )
    };

    let eta = appointment_at
        .map(|appointment| {
            let minutes_until_due = (appointment - Utc::now().naive_utc()).num_minutes();
            if minutes_until_due < -30 {
                (
                    Some(format!(
                        "{} appointment is {} minute(s) overdue.",
                        stop_label,
                        minutes_until_due.abs()
                    )),
                    "danger",
                )
            } else if minutes_until_due <= 60 && distance_km > 80.0 {
                (
                    Some(format!(
                        "Delay risk: {:.1} km from {} with {} minute(s) until appointment.",
                        distance_km, stop_label, minutes_until_due
                    )),
                    "warning",
                )
            } else {
                (
                    Some(format!(
                        "ETA risk normal: {:.1} km from {} with {} minute(s) until appointment.",
                        distance_km, stop_label, minutes_until_due
                    )),
                    "success",
                )
            }
        })
        .unwrap_or_else(|| {
            (
                Some("ETA risk needs pickup/delivery appointment times before scoring.".into()),
                "info",
            )
        });

    (geofence.0, geofence.1, eta.0, eta.1)
}

fn default_closeout_readiness(
    leg_id: i64,
    delivery_pod_attached: bool,
) -> db::tracking::ExecutionCloseoutReadinessRecord {
    db::tracking::ExecutionCloseoutReadinessRecord {
        leg_id,
        closeout_status: None,
        pod_review_status: None,
        export_path: None,
        delivery_pod_count: if delivery_pod_attached { 1 } else { 0 },
        open_exception_count: 0,
        pending_offline_count: 0,
    }
}

fn execution_closeout_ready(closeout: &db::tracking::ExecutionCloseoutReadinessRecord) -> bool {
    closeout.delivery_pod_count > 0
        && closeout.open_exception_count == 0
        && closeout.pending_offline_count == 0
        && matches!(
            closeout.pod_review_status.as_deref(),
            Some("approved") | Some("accepted")
        )
}

fn execution_closeout_package_label(
    closeout: &db::tracking::ExecutionCloseoutReadinessRecord,
) -> String {
    if execution_closeout_ready(closeout) {
        "Closeout package approved and payment-release ready.".into()
    } else if closeout.delivery_pod_count == 0 {
        "Closeout blocked: delivery POD is missing.".into()
    } else if closeout.open_exception_count > 0 {
        format!(
            "Closeout blocked by {} open claim/accessorial exception(s).",
            closeout.open_exception_count
        )
    } else if closeout.pending_offline_count > 0 {
        format!(
            "Closeout waiting on {} offline replay item(s).",
            closeout.pending_offline_count
        )
    } else {
        "Closeout waiting on POD review approval.".into()
    }
}

fn closeout_approval_blocker(
    closeout: &db::tracking::ExecutionCloseoutReadinessRecord,
) -> Option<String> {
    if closeout.delivery_pod_count == 0 {
        Some("POD closeout approval is blocked until at least one delivery POD is attached.".into())
    } else if closeout.open_exception_count > 0 {
        Some(format!(
            "POD closeout approval is blocked by {} open claim/accessorial exception(s).",
            closeout.open_exception_count
        ))
    } else if closeout.pending_offline_count > 0 {
        Some(format!(
            "POD closeout approval is blocked while {} offline submission(s) still need reconciliation.",
            closeout.pending_offline_count
        ))
    } else {
        None
    }
}

fn execution_closeout_checklist(
    closeout: &db::tracking::ExecutionCloseoutReadinessRecord,
) -> Vec<ExecutionStatusItem> {
    vec![
        ExecutionStatusItem {
            key: "delivery_pod".into(),
            label: "Delivery POD".into(),
            status_label: if closeout.delivery_pod_count > 0 {
                "Ready"
            } else {
                "Missing"
            }
            .into(),
            status_tone: if closeout.delivery_pod_count > 0 {
                "success"
            } else {
                "warning"
            }
            .into(),
            detail: format!(
                "{} delivery POD document(s) attached.",
                closeout.delivery_pod_count
            ),
        },
        ExecutionStatusItem {
            key: "pod_review".into(),
            label: "POD review".into(),
            status_label: closeout
                .pod_review_status
                .clone()
                .unwrap_or_else(|| "Pending".into()),
            status_tone: if matches!(
                closeout.pod_review_status.as_deref(),
                Some("approved") | Some("accepted")
            ) {
                "success"
            } else {
                "warning"
            }
            .into(),
            detail: "POD must be approved before finance release.".into(),
        },
        ExecutionStatusItem {
            key: "finance_exceptions".into(),
            label: "Claims and accessorials".into(),
            status_label: if closeout.open_exception_count == 0 {
                "Clear"
            } else {
                "Open"
            }
            .into(),
            status_tone: if closeout.open_exception_count == 0 {
                "success"
            } else {
                "danger"
            }
            .into(),
            detail: format!(
                "{} open claim/accessorial exception(s).",
                closeout.open_exception_count
            ),
        },
        ExecutionStatusItem {
            key: "offline_replay".into(),
            label: "Offline replay".into(),
            status_label: if closeout.pending_offline_count == 0 {
                "Clear"
            } else {
                "Pending"
            }
            .into(),
            status_tone: if closeout.pending_offline_count == 0 {
                "success"
            } else {
                "warning"
            }
            .into(),
            detail: format!(
                "{} offline submission(s) still need reconciliation.",
                closeout.pending_offline_count
            ),
        },
    ]
}

fn execution_claims_accessorial_items(
    items: &[db::tracking::ExecutionFinanceExceptionSummaryRecord],
) -> Vec<ExecutionStatusItem> {
    if items.is_empty() {
        return vec![ExecutionStatusItem {
            key: "none".into(),
            label: "Claims/accessorials".into(),
            status_label: "None open".into(),
            status_tone: "success".into(),
            detail: "No detention, accessorial, claim, damage, shortage, dispute, or service-failure records are open for this leg.".into(),
        }];
    }

    items
        .iter()
        .map(|item| ExecutionStatusItem {
            key: format!("{}:{}", item.exception_type, item.status),
            label: item.exception_type.replace('_', " "),
            status_label: item.status.replace('_', " "),
            status_tone: if matches!(item.status.as_str(), "approved" | "resolved") {
                "success"
            } else if matches!(item.status.as_str(), "rejected") {
                "secondary"
            } else {
                "warning"
            }
            .into(),
            detail: format!(
                "{} item(s), amount {}",
                item.count,
                item.amount_cents
                    .map(|value| format!("${:.2}", value as f64 / 100.0))
                    .unwrap_or_else(|| "not set".into())
            ),
        })
        .collect()
}

fn execution_route_plan_label(
    route_plan: Option<&db::tracking::ExecutionRoutePlanRecord>,
) -> String {
    route_plan
        .map(|plan| {
            let distance = plan
                .distance_miles
                .map(|value| format!("{:.1} mi", value))
                .unwrap_or_else(|| "distance TBD".into());
            let duration = plan
                .duration_minutes
                .map(|value| format!("{} min", value))
                .unwrap_or_else(|| "duration TBD".into());
            format!(
                "{} route via {}: {}, {}, truck-safe {}.",
                plan.status,
                plan.provider_key,
                distance,
                duration,
                if plan.truck_safe { "yes" } else { "not confirmed" }
            )
        })
        .unwrap_or_else(|| {
            "Route source pending: mileage, ETA, pricing, settlement, and exception logic need a documented provider or manual plan.".into()
        })
}

fn execution_telematics_label(
    telematics: Option<&db::tracking::ExecutionTelematicsSummaryRecord>,
) -> String {
    telematics
        .map(|item| {
            let last_ping = item
                .last_ping_at
                .as_ref()
                .map(format_datetime)
                .unwrap_or_else(|| "no provider ping yet".into());
            format!(
                "{} telematics is {}; last ping {}; fallback: {}",
                item.provider_key, item.status, last_ping, item.fallback_behavior
            )
        })
        .unwrap_or_else(|| {
            "No ELD/telematics provider connected. Manual/mobile tracking remains the fallback source."
                .into()
        })
}

fn execution_tracking_consent_text(leg: &db::tracking::ExecutionLegRecord) -> String {
    format!(
        "I consent to STLoads collecting and sharing GPS location for {} while this leg is active, limited to dispatch, customer-visible tracking status, exception handling, and delivery proof. Retention target: 90 days unless legal hold or contract rules require otherwise.",
        leg.leg_code
            .clone()
            .unwrap_or_else(|| format!("leg #{}", leg.leg_id))
    )
}

fn execution_transition_error_message(error: ExecutionTransitionError) -> String {
    match error {
        ExecutionTransitionError::UnknownAction => {
            "This execution action is unknown to the Rust state machine.".into()
        }
        ExecutionTransitionError::InvalidCurrentState => {
            "This execution action is not available for the leg's current Rust status.".into()
        }
        ExecutionTransitionError::MissingTrackingConsent => {
            "Record tracking consent before starting pickup or GPS tracking.".into()
        }
        ExecutionTransitionError::MissingDeliveryPod => {
            "Upload a delivery POD document before completing delivery in the Rust execution flow."
                .into()
        }
        ExecutionTransitionError::MissingCompletionNote => {
            "Add a short delivery completion note before closing the leg in the Rust execution flow."
                .into()
        }
    }
}

fn publish_execution_update(
    state: &AppState,
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
    summary: String,
) {
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::LegExecutionUpdated,
            leg_id: Some(leg.leg_id.max(0) as u64),
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: leg.booked_carrier_id.map(|value| value.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_user_ids(target_execution_user_ids(leg))
        .for_permission_keys(["manage_tracking", "access_admin_portal", "manage_loads"])
        .with_topics([
            RealtimeTopic::ExecutionTracking.as_key(),
            RealtimeTopic::LoadBoard.as_key(),
        ]),
    );
}

fn normalize_closeout_review_status(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "approved" | "accepted" => "approved",
        "rejected" => "rejected",
        "needs_review" | "review" => "needs_review",
        _ => "pending",
    }
}

fn normalize_finance_exception_type(value: &str) -> String {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "damage" | "shortage" | "late_delivery" | "charge_dispute" | "service_failure"
        | "detention" | "accessorial" | "lumper" | "layover" => {
            value.trim().to_ascii_lowercase().replace([' ', '-'], "_")
        }
        _ => "accessorial".into(),
    }
}

fn normalize_finance_exception_status(value: &str) -> String {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "approved" | "rejected" | "resolved" | "disputed" | "review" => {
            value.trim().to_ascii_lowercase().replace([' ', '-'], "_")
        }
        _ => "pending".into(),
    }
}

fn normalize_visibility(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "customer" | "carrier" | "shared" => value.trim().to_ascii_lowercase(),
        _ => "internal".into(),
    }
}

fn normalize_customer_tracking_scope(value: &str) -> &'static str {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "status_eta" => "status_eta",
        "status_eta_latest_location" | "latest_location" => "status_eta_latest_location",
        "status_eta_latest_location_documents" | "documents" => {
            "status_eta_latest_location_documents"
        }
        _ => "status_eta_latest_location",
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
    is_trackable_execution_status(status_id)
}

fn can_upload_execution_documents(
    session: &auth_session::ResolvedSession,
    leg: &db::tracking::ExecutionLegRecord,
) -> bool {
    can_manage_execution(session, leg) && matches!(leg.status_id, 4..=10)
}

fn execution_action_items(
    status_id: i16,
    can_manage: bool,
    delivery_completion_ready: bool,
    tracking_consent_granted: bool,
) -> Vec<ExecutionActionItem> {
    [
        (
            "start_pickup",
            "Start pickup",
            "Carrier confirms consented tracking and starts the pickup workflow.",
            matches!(status_id, 4 | 8) && tracking_consent_granted,
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
            mobile_capture_hint: "Camera or PDF upload at pickup.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "pickup_photo".into(),
            label: "Pickup Photos".into(),
            description: "Pickup condition or loading proof captured on site.".into(),
            mobile_capture_hint: "Camera-first pickup evidence.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "delivery_pod".into(),
            label: "Delivery POD".into(),
            description: "Proof of delivery or signed receiving confirmation.".into(),
            mobile_capture_hint: "Camera or PDF proof required for closeout.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "delivery_photo".into(),
            label: "Delivery Photos".into(),
            description: "Delivery condition or unload proof captured on site.".into(),
            mobile_capture_hint: "Camera-first delivery evidence.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "seal_photo".into(),
            label: "Seal Photos".into(),
            description: "Seal number, intact seal, or seal exception proof.".into(),
            mobile_capture_hint: "Camera capture before departure or delivery.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "damage_photo".into(),
            label: "Damage Evidence".into(),
            description: "Damage, shortage, overage, or exception evidence.".into(),
            mobile_capture_hint: "Camera capture with dispatch note required.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "accessorial_evidence".into(),
            label: "Accessorial Evidence".into(),
            description: "Detention, lumper, layover, or accessorial proof.".into(),
            mobile_capture_hint: "Receipt or camera proof for billing review.".into(),
        },
        ExecutionDocumentTypeOption {
            key: "other".into(),
            label: "Other".into(),
            description: "Additional execution-stage attachment kept with the leg.".into(),
            mobile_capture_hint: "Camera or file upload.".into(),
        },
    ]
}

fn is_supported_execution_document_type(value: &str) -> bool {
    matches!(
        value,
        "pickup_bol"
            | "pickup_photo"
            | "delivery_pod"
            | "delivery_photo"
            | "seal_photo"
            | "damage_photo"
            | "accessorial_evidence"
            | "other"
    )
}

fn target_execution_user_ids(leg: &db::tracking::ExecutionLegRecord) -> Vec<u64> {
    let mut users = Vec::new();
    if let Some(load_owner_user_id) = leg.load_owner_user_id
        && load_owner_user_id > 0
    {
        users.push(load_owner_user_id as u64);
    }
    if let Some(booked_carrier_id) = leg.booked_carrier_id
        && booked_carrier_id > 0
        && !users.contains(&(booked_carrier_id as u64))
    {
        users.push(booked_carrier_id as u64);
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

fn document_version_label(current_version: i32, version_count: i64) -> String {
    let current = current_version.max(1);
    let count = version_count.max(1);
    if count == 1 {
        format!("v{} (original)", current)
    } else {
        format!("v{} of {}", current, count)
    }
}

fn execution_required_document_checklist(
    delivery_pod_attached: bool,
) -> Vec<RequiredDocumentChecklistItem> {
    vec![RequiredDocumentChecklistItem {
        key: "delivery_pod".into(),
        label: "Delivery POD".into(),
        requirement_scope: "Execution closeout".into(),
        lifecycle_state: "complete_delivery".into(),
        is_required: true,
        is_satisfied: delivery_pod_attached,
        status_label: if delivery_pod_attached {
            "Ready"
        } else {
            "Missing"
        }
        .into(),
        status_tone: if delivery_pod_attached {
            "success"
        } else {
            "warning"
        }
        .into(),
        blocking_message: (!delivery_pod_attached)
            .then(|| "Delivery POD is required before Complete delivery unlocks.".into()),
    }]
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

fn sanitize_zip_entry_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\r' | '\n' => '_',
            _ => ch,
        })
        .collect::<String>();

    let trimmed = sanitized.trim_matches(['.', ' ']).trim();
    if trimmed.is_empty() {
        "document.bin".into()
    } else {
        trimmed.into()
    }
}

fn closeout_document_zip_name(document_id: i64, document_name: &str) -> String {
    format!(
        "documents/{}-{}",
        document_id,
        sanitize_zip_entry_name(document_name)
    )
}

fn build_closeout_zip_package(
    manifest: &str,
    documents: Vec<(String, Vec<u8>)>,
) -> anyhow::Result<Vec<u8>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    writer.start_file("manifest.txt", options)?;
    writer.write_all(manifest.as_bytes())?;

    for (entry_name, bytes) in documents {
        writer.start_file(entry_name, options)?;
        writer.write_all(&bytes)?;
    }

    Ok(writer.finish()?.into_inner())
}

fn google_maps_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat:.5},{lng:.5}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        auth_headers_for_user, insert_load_fixture, prepare_pool, read_leg_status, test_state,
    };
    use db::tracking::{CreateLegDocumentParams, create_leg_document};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn execution_routes_enforce_pod_note_and_document_visibility()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let fixture = insert_load_fixture(&pool, 9).await?;
        let carrier_headers = auth_headers_for_user(&state, &fixture.carrier_user).await?;
        let owner_headers = auth_headers_for_user(&state, &fixture.owner_user).await?;

        let missing_pod = run_leg_action(
            State(state.clone()),
            Path(fixture.leg_id),
            carrier_headers.clone(),
            Json(ExecutionLegActionRequest {
                action_key: "complete_delivery".into(),
                note: Some("Driver says delivery is complete.".into()),
            }),
        )
        .await
        .expect("execution action should return a structured response")
        .0
        .data;
        assert!(!missing_pod.success);
        assert!(missing_pod.message.contains("delivery POD"));

        let blocked_closeout = review_closeout_package(
            State(state.clone()),
            Path(fixture.leg_id),
            carrier_headers.clone(),
            Json(ExecutionCloseoutApprovalRequest {
                pod_review_status: "approved".into(),
                export_path: Some(format!(
                    "/execution/legs/{}/closeout-package",
                    fixture.leg_id
                )),
                note: Some("Trying to approve before POD.".into()),
            }),
        )
        .await
        .expect("closeout review should return a structured response")
        .0
        .data;
        assert!(!blocked_closeout.success);
        assert!(blocked_closeout.message.contains("delivery POD"));

        let saved = state
            .document_storage
            .save_execution_document(fixture.leg_id, "pod.pdf", b"pod-bytes")
            .await?;
        let document = create_leg_document(
            &pool,
            fixture.leg_id,
            &CreateLegDocumentParams {
                document_name: "Proof of Delivery".into(),
                document_type: "delivery_pod".into(),
                file_path: saved.file_path,
                storage_provider: saved.storage_provider,
                original_name: Some("pod.pdf".into()),
                mime_type: Some("application/pdf".into()),
                file_size: Some(8),
            },
            Some(fixture.carrier_user.id),
        )
        .await?
        .expect("leg document created");

        let owner_download =
            download_leg_document_file(State(state.clone()), Path(document.id), owner_headers)
                .await;
        assert_eq!(owner_download.status(), StatusCode::FORBIDDEN);

        let carrier_download = download_leg_document_file(
            State(state.clone()),
            Path(document.id),
            carrier_headers.clone(),
        )
        .await;
        assert_eq!(carrier_download.status(), StatusCode::OK);

        let approved_closeout = review_closeout_package(
            State(state.clone()),
            Path(fixture.leg_id),
            carrier_headers.clone(),
            Json(ExecutionCloseoutApprovalRequest {
                pod_review_status: "approved".into(),
                export_path: Some(format!(
                    "/execution/legs/{}/closeout-package",
                    fixture.leg_id
                )),
                note: Some("POD is attached and reviewed.".into()),
            }),
        )
        .await
        .expect("closeout review should return a structured response")
        .0
        .data;
        assert!(approved_closeout.success);
        assert!(approved_closeout.message.contains("payment-release ready"));

        let closeout_package = download_closeout_package(
            State(state.clone()),
            Path(fixture.leg_id),
            carrier_headers.clone(),
        )
        .await;
        assert_eq!(closeout_package.status(), StatusCode::OK);
        assert_eq!(
            closeout_package
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/zip")
        );
        assert!(
            closeout_package
                .headers()
                .get(header::CONTENT_DISPOSITION)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains(".zip"))
        );
        let closeout_bytes = axum::body::to_bytes(closeout_package.into_body(), usize::MAX).await?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(closeout_bytes.to_vec()))?;
        assert!(archive.by_name("manifest.txt").is_ok());
        let archive_names = archive
            .file_names()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        assert!(
            archive_names
                .iter()
                .any(|value| value.starts_with("documents/") && value.ends_with("pod.pdf"))
        );

        let missing_note = run_leg_action(
            State(state.clone()),
            Path(fixture.leg_id),
            carrier_headers.clone(),
            Json(ExecutionLegActionRequest {
                action_key: "complete_delivery".into(),
                note: None,
            }),
        )
        .await
        .expect("execution action should return a structured response")
        .0
        .data;
        assert!(!missing_note.success);
        assert!(missing_note.message.contains("delivery completion note"));

        let delivered = run_leg_action(
            State(state),
            Path(fixture.leg_id),
            carrier_headers,
            Json(ExecutionLegActionRequest {
                action_key: "complete_delivery".into(),
                note: Some("POD uploaded and consignee signed.".into()),
            }),
        )
        .await
        .expect("delivery completion should return a structured response")
        .0
        .data;
        assert!(delivered.success);
        assert_eq!(delivered.status_label, "Delivered");
        assert_eq!(read_leg_status(&pool, fixture.leg_id).await?, 10);

        Ok(())
    }
}
