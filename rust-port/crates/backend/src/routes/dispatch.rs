use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use chrono::{NaiveDate, NaiveDateTime};
use db::{
    dispatch::{
        CreateLoadLegParams, CreateLoadParams, book_load_leg, create_load_with_legs,
        find_load_leg_by_id, find_load_leg_scope,
    },
    master_data::{list_commodity_types, list_equipments, list_load_types, list_locations},
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
    LoadBoardScreen, LoadBuilderOption, LoadBuilderScreen, RealtimeEvent, RealtimeEventKind,
    RealtimeTopic,
};

use crate::{auth_session, realtime_bus::RoutedRealtimeEvent, screen_data, state::AppState};

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
        .route("/load-builder", get(load_builder))
        .route("/loads", post(create_load))
        .route("/load-board/{leg_id}/book", post(book_leg))
}

async fn index() -> Json<ApiResponse<DispatchOverview>> {
    let contract = load_module_contract();
    Json(ApiResponse::ok(DispatchOverview {
        document_kinds: contract.document_kinds.len(),
        legacy_status_count: legacy_load_leg_status_descriptors().len(),
        screen_routes: vec!["/dispatch/load-board", "/dispatch/load-builder"],
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
        build_load_builder_screen(&state, viewer.as_ref()).await,
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
        if leg.pickup_location_id == leg.delivery_location_id {
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

        leg_params.push(CreateLoadLegParams {
            pickup_location_id: leg.pickup_location_id as i64,
            delivery_location_id: leg.delivery_location_id as i64,
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
                "{} created load {} with {} leg(s) from the Rust builder. Document uploads will be ported next.",
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
) -> LoadBuilderScreen {
    let Some(viewer) = viewer else {
        return empty_load_builder_screen(
            state,
            vec![
                "Sign in before creating a load from the Rust builder.".into(),
                "This route intentionally avoids Laravel fallback forms during staged cutover."
                    .into(),
            ],
        );
    };

    if !can_manage_loads(viewer) {
        return empty_load_builder_screen(
            state,
            vec![
                "The authenticated session does not have load creation access in the Rust slice."
                    .into(),
            ],
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return empty_load_builder_screen(
            state,
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
        "This first Rust load builder ports core load creation with structured master-data selects and multi-leg posting.".into(),
        "Document upload rows and Google autocomplete will be layered in after the stable create path is fully exercised.".into(),
    ];

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so create-load requests remain proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    LoadBuilderScreen {
        title: "Create Load".into(),
        subtitle: "First-pass Rust builder for core load details and multi-leg creation.".into(),
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

fn empty_load_builder_screen(state: &AppState, mut notes: Vec<String>) -> LoadBuilderScreen {
    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so create-load requests remain proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    LoadBuilderScreen {
        title: "Create Load".into(),
        subtitle: "Secure Rust builder".into(),
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

fn parse_date_for_storage(value: &str) -> Result<NaiveDateTime, String> {
    let date = NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| "use YYYY-MM-DD format".to_string())?;
    date.and_hms_opt(0, 0, 0)
        .ok_or_else(|| "date could not be normalized".to_string())
}
