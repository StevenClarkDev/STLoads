use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use domain::master_data::{
    LOAD_CREATION_MASTER_DATA, MasterDataDescriptor, MasterDataKind, master_data_descriptors,
};
use serde::Serialize;
use shared::{
    ApiResponse, LocationUpsertRequest, MasterDataCityOption, MasterDataMutationResponse,
    MasterDataOption, MasterDataRow, MasterDataScreen, MasterDataSection, MasterDataSummaryCard,
    SimpleCatalogUpsertRequest,
};
use tracing::warn;

use crate::{auth_session, auth_session::ResolvedSession, state::AppState};

#[derive(Debug, Serialize)]
struct MasterDataOverview {
    total_resources: usize,
    admin_managed_resources: usize,
    resources: Vec<MasterDataDescriptor>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/catalog", get(catalog))
        .route("/load-creation", get(load_creation_catalog))
        .route("/screen", get(screen))
        .route("/load-types", post(upsert_load_type_handler))
        .route("/equipments", post(upsert_equipment_handler))
        .route("/commodity-types", post(upsert_commodity_type_handler))
        .route("/locations", post(upsert_location_handler))
}

async fn index() -> Json<ApiResponse<MasterDataOverview>> {
    let resources = master_data_descriptors().to_vec();
    Json(ApiResponse::ok(MasterDataOverview {
        total_resources: resources.len(),
        admin_managed_resources: resources.len(),
        resources,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("master-data route group ready"))
}

async fn catalog() -> Json<ApiResponse<Vec<MasterDataDescriptor>>> {
    Json(ApiResponse::ok(master_data_descriptors().to_vec()))
}

async fn load_creation_catalog() -> Json<ApiResponse<Vec<MasterDataDescriptor>>> {
    let resources = master_data_descriptors()
        .iter()
        .filter(|descriptor| LOAD_CREATION_MASTER_DATA.contains(&descriptor.kind))
        .cloned()
        .collect::<Vec<_>>();

    Json(ApiResponse::ok(resources))
}

async fn screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MasterDataScreen>>, StatusCode> {
    let _session = require_master_data_access(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        build_master_data_screen(&state).await,
    )))
}

async fn upsert_load_type_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SimpleCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let name = match validate_name(&payload.name) {
        Ok(name) => name,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "load_types".into(),
                row_id: None,
                message,
            })));
        }
    };

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: false,
            kind: "load_types".into(),
            row_id: None,
            message: unavailable_message(&state, "master-data saves"),
        })));
    };

    match db::master_data::upsert_load_type(pool, payload.id.map(|value| value as i64), &name).await
    {
        Ok(row) => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: "load_types".into(),
            row_id: Some(row.id.max(0) as u64),
            message: success_message(&session, payload.id.is_some(), "load type", &row.name),
        }))),
        Err(error) => {
            warn!(error = %error, "failed to save load type");
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "load_types".into(),
                row_id: payload.id,
                message: format!(
                    "{} save failed: {}",
                    row_kind_label("load_types"),
                    humanize_db_error(&error)
                ),
            })))
        }
    }
}

async fn upsert_equipment_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SimpleCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let name = match validate_name(&payload.name) {
        Ok(name) => name,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "equipments".into(),
                row_id: None,
                message,
            })));
        }
    };

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: false,
            kind: "equipments".into(),
            row_id: None,
            message: unavailable_message(&state, "master-data saves"),
        })));
    };

    match db::master_data::upsert_equipment(pool, payload.id.map(|value| value as i64), &name).await
    {
        Ok(row) => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: "equipments".into(),
            row_id: Some(row.id.max(0) as u64),
            message: success_message(&session, payload.id.is_some(), "equipment", &row.name),
        }))),
        Err(error) => {
            warn!(error = %error, "failed to save equipment");
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "equipments".into(),
                row_id: payload.id,
                message: format!(
                    "{} save failed: {}",
                    row_kind_label("equipments"),
                    humanize_db_error(&error)
                ),
            })))
        }
    }
}

async fn upsert_commodity_type_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SimpleCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let name = match validate_name(&payload.name) {
        Ok(name) => name,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "commodity_types".into(),
                row_id: None,
                message,
            })));
        }
    };

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: false,
            kind: "commodity_types".into(),
            row_id: None,
            message: unavailable_message(&state, "master-data saves"),
        })));
    };

    match db::master_data::upsert_commodity_type(pool, payload.id.map(|value| value as i64), &name)
        .await
    {
        Ok(row) => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: "commodity_types".into(),
            row_id: Some(row.id.max(0) as u64),
            message: success_message(&session, payload.id.is_some(), "commodity type", &row.name),
        }))),
        Err(error) => {
            warn!(error = %error, "failed to save commodity type");
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "commodity_types".into(),
                row_id: payload.id,
                message: format!(
                    "{} save failed: {}",
                    row_kind_label("commodity_types"),
                    humanize_db_error(&error)
                ),
            })))
        }
    }
}

async fn upsert_location_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LocationUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let name = match validate_name(&payload.name) {
        Ok(name) => name,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "locations".into(),
                row_id: None,
                message,
            })));
        }
    };

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: false,
            kind: "locations".into(),
            row_id: None,
            message: unavailable_message(&state, "location saves"),
        })));
    };

    let location_id = payload.id.map(|value| value as i64);
    let city_id = payload.city_id.map(|value| value as i64);
    let country_id = payload.country_id.map(|value| value as i64);

    match db::master_data::upsert_location(pool, location_id, &name, city_id, country_id).await {
        Ok(row) => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: "locations".into(),
            row_id: Some(row.id.max(0) as u64),
            message: success_message(&session, payload.id.is_some(), "location", &row.name),
        }))),
        Err(error) => {
            warn!(error = %error, "failed to save location");
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: false,
                kind: "locations".into(),
                row_id: payload.id,
                message: format!("Location save failed: {}", humanize_db_error(&error)),
            })))
        }
    }
}

async fn require_master_data_access(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<ResolvedSession, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(state, headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let allowed = session.session.permissions.iter().any(|permission| {
        permission == "access_admin_portal" || permission == "manage_master_data"
    });

    if allowed {
        Ok(session)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn build_master_data_screen(state: &AppState) -> MasterDataScreen {
    let descriptors = master_data_descriptors().to_vec();

    let Some(pool) = state.pool.as_ref() else {
        return fallback_master_data_screen(
            state,
            descriptors,
            format!(
                "Master data is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        );
    };

    let countries = match db::master_data::list_countries(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load countries for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading countries: {}",
                    error
                ),
            );
        }
    };

    let cities = match db::master_data::list_cities(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load cities for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!("Master data screen failed while loading cities: {}", error),
            );
        }
    };

    let locations = match db::master_data::list_locations(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load locations for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading locations: {}",
                    error
                ),
            );
        }
    };

    let load_types = match db::master_data::list_load_types(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load load types for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading load types: {}",
                    error
                ),
            );
        }
    };

    let equipments = match db::master_data::list_equipments(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load equipments for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading equipments: {}",
                    error
                ),
            );
        }
    };

    let commodity_types = match db::master_data::list_commodity_types(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load commodity types for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading commodity types: {}",
                    error
                ),
            );
        }
    };

    let load_statuses = match db::master_data::list_load_statuses(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load load statuses for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading load statuses: {}",
                    error
                ),
            );
        }
    };

    let offer_statuses = match db::master_data::list_offer_statuses(pool).await {
        Ok(rows) => rows,
        Err(error) => {
            warn!(error = %error, "failed to load offer statuses for master-data screen");
            return fallback_master_data_screen(
                state,
                descriptors,
                format!(
                    "Master data screen failed while loading offer statuses: {}",
                    error
                ),
            );
        }
    };

    let country_name_map = countries
        .iter()
        .map(|row| (row.id, row.name.clone()))
        .collect::<HashMap<_, _>>();
    let city_name_map = cities
        .iter()
        .map(|row| (row.id, row.name.clone()))
        .collect::<HashMap<_, _>>();

    let sections = descriptors
        .iter()
        .map(|descriptor| match descriptor.kind {
            MasterDataKind::Countries => MasterDataSection {
                key: kind_key(descriptor.kind).into(),
                label: descriptor.label.into(),
                admin_route: descriptor.admin_route.into(),
                total: countries.len() as u64,
                rows: countries
                    .iter()
                    .take(6)
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: row.iso_code.clone(),
                        status_label: "Active".into(),
                        detail: usage_detail(descriptor),
                        editable: false,
                        country_id: None,
                        city_id: None,
                    })
                    .collect(),
                empty_message: format!("No {} are available yet.", descriptor.label.to_lowercase()),
            },
            MasterDataKind::Cities => MasterDataSection {
                key: kind_key(descriptor.kind).into(),
                label: descriptor.label.into(),
                admin_route: descriptor.admin_route.into(),
                total: cities.len() as u64,
                rows: cities
                    .iter()
                    .take(6)
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: country_name_map.get(&row.country_id).cloned(),
                        status_label: "Active".into(),
                        detail: usage_detail(descriptor),
                        editable: false,
                        country_id: Some(row.country_id.max(0) as u64),
                        city_id: None,
                    })
                    .collect(),
                empty_message: format!("No {} are available yet.", descriptor.label.to_lowercase()),
            },
            MasterDataKind::Locations => MasterDataSection {
                key: kind_key(descriptor.kind).into(),
                label: descriptor.label.into(),
                admin_route: descriptor.admin_route.into(),
                total: locations.len() as u64,
                rows: locations
                    .iter()
                    .take(6)
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: location_secondary_label(
                            row.city_id,
                            row.country_id,
                            &city_name_map,
                            &country_name_map,
                        ),
                        status_label: if row.deleted_at.is_some() {
                            "Soft Deleted".into()
                        } else {
                            "Active".into()
                        },
                        detail: usage_detail(descriptor),
                        editable: true,
                        country_id: row.country_id.map(|value| value.max(0) as u64),
                        city_id: row.city_id.map(|value| value.max(0) as u64),
                    })
                    .collect(),
                empty_message: format!("No {} are available yet.", descriptor.label.to_lowercase()),
            },
            MasterDataKind::LoadTypes => simple_lookup_section(
                descriptor,
                load_types
                    .iter()
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: None,
                        status_label: if row.deleted_at.is_some() {
                            "Soft Deleted".into()
                        } else {
                            "Active".into()
                        },
                        detail: usage_detail(descriptor),
                        editable: true,
                        country_id: None,
                        city_id: None,
                    })
                    .collect(),
            ),
            MasterDataKind::Equipments => simple_lookup_section(
                descriptor,
                equipments
                    .iter()
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: None,
                        status_label: if row.deleted_at.is_some() {
                            "Soft Deleted".into()
                        } else {
                            "Active".into()
                        },
                        detail: usage_detail(descriptor),
                        editable: true,
                        country_id: None,
                        city_id: None,
                    })
                    .collect(),
            ),
            MasterDataKind::CommodityTypes => simple_lookup_section(
                descriptor,
                commodity_types
                    .iter()
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: None,
                        status_label: if row.deleted_at.is_some() {
                            "Soft Deleted".into()
                        } else {
                            "Active".into()
                        },
                        detail: usage_detail(descriptor),
                        editable: true,
                        country_id: None,
                        city_id: None,
                    })
                    .collect(),
            ),
            MasterDataKind::LoadStatuses => simple_lookup_section(
                descriptor,
                load_statuses
                    .iter()
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: Some(row.slug.clone()),
                        status_label: if row.is_terminal {
                            "Terminal".into()
                        } else {
                            "Lifecycle".into()
                        },
                        detail: row
                            .description
                            .clone()
                            .unwrap_or_else(|| usage_detail(descriptor)),
                        editable: false,
                        country_id: None,
                        city_id: None,
                    })
                    .collect(),
            ),
            MasterDataKind::OfferStatuses => simple_lookup_section(
                descriptor,
                offer_statuses
                    .iter()
                    .map(|row| MasterDataRow {
                        id: row.id.max(0) as u64,
                        primary_label: row.name.clone(),
                        secondary_label: Some(row.slug.clone()),
                        status_label: if row.is_terminal {
                            "Terminal".into()
                        } else {
                            "Workflow".into()
                        },
                        detail: row
                            .description
                            .clone()
                            .unwrap_or_else(|| usage_detail(descriptor)),
                        editable: false,
                        country_id: None,
                        city_id: None,
                    })
                    .collect(),
            ),
        })
        .collect::<Vec<_>>();

    let summary_cards = descriptors
        .iter()
        .map(|descriptor| MasterDataSummaryCard {
            key: kind_key(descriptor.kind).into(),
            label: descriptor.label.into(),
            total: sections
                .iter()
                .find(|section| section.key == kind_key(descriptor.kind))
                .map(|section| section.total)
                .unwrap_or(0),
            admin_route: descriptor.admin_route.into(),
            note: usage_detail(descriptor),
        })
        .collect::<Vec<_>>();

    let country_options = countries
        .iter()
        .map(|row| MasterDataOption {
            id: row.id.max(0) as u64,
            label: row.name.clone(),
        })
        .collect::<Vec<_>>();

    let city_options = cities
        .iter()
        .map(|row| MasterDataCityOption {
            id: row.id.max(0) as u64,
            country_id: row.country_id.max(0) as u64,
            label: row.name.clone(),
        })
        .collect::<Vec<_>>();

    let mut notes = vec![
        "This master-data route is now DB-backed and writable for locations, load types, equipments, and commodity types in the Rust admin portal.".into(),
        "Countries, cities, and status masters remain read-first for now so we can keep the load-builder dependency graph stable while porting forms.".into(),
    ];

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so the admin master-data surface stays proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    MasterDataScreen {
        title: "Master Data Catalog".into(),
        subtitle: "Admin visibility plus first-write workflows for the lookup data that powers load creation in the Rust port.".into(),
        summary_cards,
        sections,
        country_options,
        city_options,
        notes,
    }
}

fn fallback_master_data_screen(
    state: &AppState,
    descriptors: Vec<MasterDataDescriptor>,
    reason: String,
) -> MasterDataScreen {
    let mut notes = vec![
        reason,
        "This fallback keeps the admin master-data route alive even when the PostgreSQL connection is unavailable.".into(),
    ];

    if let Some(public_base_url) = state.config.public_base_url.as_ref() {
        notes.push(format!(
            "IBM deployment note: PUBLIC_BASE_URL is set to {} so the admin master-data surface stays proxy-safe during staged cutover.",
            public_base_url
        ));
    }

    MasterDataScreen {
        title: "Master Data Catalog".into(),
        subtitle: "Read-first admin visibility for the Rust port.".into(),
        summary_cards: descriptors
            .iter()
            .map(|descriptor| MasterDataSummaryCard {
                key: kind_key(descriptor.kind).into(),
                label: descriptor.label.into(),
                total: 0,
                admin_route: descriptor.admin_route.into(),
                note: usage_detail(descriptor),
            })
            .collect(),
        sections: descriptors
            .iter()
            .map(|descriptor| MasterDataSection {
                key: kind_key(descriptor.kind).into(),
                label: descriptor.label.into(),
                admin_route: descriptor.admin_route.into(),
                total: 0,
                rows: Vec::new(),
                empty_message: format!(
                    "{} will appear here once the Rust backend can read PostgreSQL successfully.",
                    descriptor.label
                ),
            })
            .collect(),
        country_options: Vec::new(),
        city_options: Vec::new(),
        notes,
    }
}

fn simple_lookup_section(
    descriptor: &MasterDataDescriptor,
    rows: Vec<MasterDataRow>,
) -> MasterDataSection {
    let total = rows.len() as u64;
    MasterDataSection {
        key: kind_key(descriptor.kind).into(),
        label: descriptor.label.into(),
        admin_route: descriptor.admin_route.into(),
        total,
        rows: rows.into_iter().take(6).collect(),
        empty_message: format!("No {} are available yet.", descriptor.label.to_lowercase()),
    }
}

fn validate_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Err("Enter a name before saving this master-data record.".into())
    } else {
        Ok(trimmed.to_string())
    }
}

fn unavailable_message(state: &AppState, action: &str) -> String {
    format!(
        "{} are unavailable because the database is {} on {}.",
        action,
        state.database_state(),
        state.config.deployment_target
    )
}

fn success_message(
    session: &ResolvedSession,
    is_edit: bool,
    record_label: &str,
    record_name: &str,
) -> String {
    if is_edit {
        format!(
            "{} updated the {} '{}' from the Rust admin portal.",
            session.user.name, record_label, record_name
        )
    } else {
        format!(
            "{} created the {} '{}' from the Rust admin portal.",
            session.user.name, record_label, record_name
        )
    }
}

fn humanize_db_error(error: &sqlx::Error) -> String {
    if let Some(code) = error
        .as_database_error()
        .and_then(|db_error| db_error.code().map(|value| value.to_string()))
    {
        if code == "23505" {
            return "a record with the same name already exists".into();
        }

        if code == "23503" {
            return "one of the selected related records no longer exists".into();
        }
    }

    error.to_string()
}

fn row_kind_label(kind: &str) -> &'static str {
    match kind {
        "load_types" => "Load type",
        "equipments" => "Equipment",
        "commodity_types" => "Commodity type",
        _ => "Master data",
    }
}

fn usage_detail(descriptor: &MasterDataDescriptor) -> String {
    format!("Used by {}", descriptor.used_by.join(", "))
}

fn kind_key(kind: MasterDataKind) -> &'static str {
    match kind {
        MasterDataKind::Countries => "countries",
        MasterDataKind::Cities => "cities",
        MasterDataKind::Locations => "locations",
        MasterDataKind::LoadTypes => "load_types",
        MasterDataKind::Equipments => "equipments",
        MasterDataKind::CommodityTypes => "commodity_types",
        MasterDataKind::LoadStatuses => "load_statuses",
        MasterDataKind::OfferStatuses => "offer_statuses",
    }
}

fn location_secondary_label(
    city_id: Option<i64>,
    country_id: Option<i64>,
    city_name_map: &HashMap<i64, String>,
    country_name_map: &HashMap<i64, String>,
) -> Option<String> {
    let city_name = city_id.and_then(|id| city_name_map.get(&id).cloned());
    let country_name = country_id.and_then(|id| country_name_map.get(&id).cloned());

    match (city_name, country_name) {
        (Some(city), Some(country)) => Some(format!("{}, {}", city, country)),
        (Some(city), None) => Some(city),
        (None, Some(country)) => Some(country),
        (None, None) => None,
    }
}
