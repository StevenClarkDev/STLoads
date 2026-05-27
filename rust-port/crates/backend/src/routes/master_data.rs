use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::{Local, NaiveDate, Utc};
use domain::master_data::{
    LOAD_CREATION_MASTER_DATA, MasterDataDescriptor, MasterDataKind, master_data_descriptors,
};
use serde::Serialize;
use shared::{
    ApiResponse, CityUpsertRequest, CountryUpsertRequest, CustomerConfigurationRuleUpsertRequest,
    DocumentRequirementRuleUpsertRequest, GovernedCatalogUpsertRequest, LocationUpsertRequest,
    MasterDataCityOption, MasterDataDeleteRequest, MasterDataExportResponse,
    MasterDataImportRequest, MasterDataMutationResponse, MasterDataOption,
    MasterDataRollbackRequest, MasterDataRow, MasterDataScreen, MasterDataSection,
    MasterDataSummaryCard, SimpleCatalogUpsertRequest,
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
        .route("/governed/export", get(export_governed_master_data))
        .route("/governed/import", post(import_governed_master_data))
        .route("/governed/rollback", post(rollback_governed_change))
        .route("/countries", post(upsert_country_handler))
        .route("/cities", post(upsert_city_handler))
        .route("/load-types", post(upsert_load_type_handler))
        .route("/equipments", post(upsert_equipment_handler))
        .route("/commodity-types", post(upsert_commodity_type_handler))
        .route("/locations", post(upsert_location_handler))
        .route("/service-levels", post(upsert_service_level_handler))
        .route("/rejection-reasons", post(upsert_rejection_reason_handler))
        .route("/exception-reasons", post(upsert_exception_reason_handler))
        .route("/trailer-types", post(upsert_trailer_type_handler))
        .route("/hazmat-classes", post(upsert_hazmat_class_handler))
        .route("/accessorials", post(upsert_accessorial_handler))
        .route(
            "/document-requirements",
            post(upsert_document_requirement_handler),
        )
        .route(
            "/customer-configurations",
            post(upsert_customer_configuration_handler),
        )
        .route("/countries/delete", post(delete_country_handler))
        .route("/cities/delete", post(delete_city_handler))
        .route("/load-types/delete", post(delete_load_type_handler))
        .route("/equipments/delete", post(delete_equipment_handler))
        .route(
            "/commodity-types/delete",
            post(delete_commodity_type_handler),
        )
        .route("/locations/delete", post(delete_location_handler))
        .route("/service-levels/delete", post(delete_service_level_handler))
        .route(
            "/rejection-reasons/delete",
            post(delete_rejection_reason_handler),
        )
        .route(
            "/exception-reasons/delete",
            post(delete_exception_reason_handler),
        )
        .route("/trailer-types/delete", post(delete_trailer_type_handler))
        .route("/hazmat-classes/delete", post(delete_hazmat_class_handler))
        .route("/accessorials/delete", post(delete_accessorial_handler))
        .route(
            "/document-requirements/delete",
            post(delete_document_requirement_handler),
        )
        .route(
            "/customer-configurations/delete",
            post(delete_customer_configuration_handler),
        )
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

async fn export_governed_master_data(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<MasterDataExportResponse>>, StatusCode> {
    let _session = require_master_data_access(&state, &headers).await?;
    let screen = build_master_data_screen(&state).await;
    let sections = screen
        .sections
        .into_iter()
        .filter(|section| is_governed_master_data_kind(&section.key))
        .collect::<Vec<_>>();

    Ok(Json(ApiResponse::ok(MasterDataExportResponse {
        exported_at: Utc::now().to_rfc3339(),
        sections,
    })))
}

async fn import_governed_master_data(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataImportRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            &payload.kind,
            None,
            unavailable_message(&state, "governed master-data imports"),
        ))));
    };
    let Some((table_name, label)) = governed_catalog_table(&payload.kind) else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            &payload.kind,
            None,
            "Governed import currently supports service levels, rejection reasons, exception reasons, trailer types, hazmat classes, and accessorials.",
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            &payload.kind,
            None,
            "Governed imports require an organization context.",
        ))));
    };

    let mut validated_rows = Vec::new();
    for row in payload.rows {
        let code = match normalize_catalog_code(&row.code) {
            Ok(value) => value,
            Err(message) => {
                return Ok(Json(ApiResponse::ok(failed_mutation(
                    &payload.kind,
                    row.id,
                    message,
                ))));
            }
        };
        let row_label = match validate_name(&row.label) {
            Ok(value) => value,
            Err(message) => {
                return Ok(Json(ApiResponse::ok(failed_mutation(
                    &payload.kind,
                    row.id,
                    message,
                ))));
            }
        };
        validated_rows.push((
            row.id,
            code,
            row_label,
            row.description,
            row.requires_approval,
            parse_effective_date(row.effective_from.as_deref())
                .unwrap_or_else(|| Local::now().date_naive()),
            parse_effective_date(row.effective_to.as_deref()),
        ));
    }

    if payload.dry_run {
        return Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: payload.kind,
            row_id: None,
            message: format!(
                "Validated {} governed {} import row(s); dry run did not change data.",
                validated_rows.len(),
                label
            ),
        })));
    }

    let mut imported = 0_u64;
    let mut last_row_id = None;
    for (id, code, row_label, description, requires_approval, effective_from, effective_to) in
        validated_rows
    {
        let description = description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let row_id = if let Some(id) = id {
            sqlx::query_scalar::<_, i64>(&format!(
                "UPDATE {}
                 SET code = $3,
                     label = $4,
                     description = $5,
                     requires_approval = $6,
                     effective_from = $7,
                     effective_to = $8,
                     is_active = TRUE,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE organization_id = $1 AND id = $2
                 RETURNING id",
                table_name
            ))
            .bind(organization_id)
            .bind(id as i64)
            .bind(&code)
            .bind(&row_label)
            .bind(description)
            .bind(requires_approval)
            .bind(effective_from)
            .bind(effective_to)
            .fetch_optional(pool)
            .await
        } else {
            sqlx::query_scalar::<_, i64>(&format!(
                "INSERT INTO {} (
                    organization_id, code, label, description, requires_approval,
                    is_active, effective_from, effective_to, created_at, updated_at
                 )
                 VALUES ($1, $2, $3, $4, $5, TRUE, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 ON CONFLICT (organization_id, code)
                 DO UPDATE SET
                    label = EXCLUDED.label,
                    description = EXCLUDED.description,
                    requires_approval = EXCLUDED.requires_approval,
                    is_active = TRUE,
                    effective_from = EXCLUDED.effective_from,
                    effective_to = EXCLUDED.effective_to,
                    updated_at = CURRENT_TIMESTAMP
                 RETURNING id",
                table_name
            ))
            .bind(organization_id)
            .bind(&code)
            .bind(&row_label)
            .bind(description)
            .bind(requires_approval)
            .bind(effective_from)
            .bind(effective_to)
            .fetch_optional(pool)
            .await
        };

        match row_id {
            Ok(Some(row_id)) => {
                imported += 1;
                last_row_id = Some(row_id.max(0) as u64);
                let _ = record_governed_configuration_change(
                    pool,
                    organization_id,
                    &payload.kind,
                    table_name,
                    row_id,
                    "import",
                    Some(session.user.id),
                    format!(
                        "{} imported governed {} '{}' ({}).",
                        session.user.name, label, row_label, code
                    ),
                    effective_from,
                    effective_to,
                )
                .await;
            }
            Ok(None) => {
                return Ok(Json(ApiResponse::ok(failed_mutation(
                    &payload.kind,
                    id,
                    format!("No governed {} was found for import row '{}'.", label, code),
                ))));
            }
            Err(error) => {
                return Ok(Json(ApiResponse::ok(failed_mutation(
                    &payload.kind,
                    id,
                    format!(
                        "Governed {} import failed: {}",
                        label,
                        humanize_db_error(&error)
                    ),
                ))));
            }
        }
    }

    Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
        success: true,
        kind: payload.kind,
        row_id: last_row_id,
        message: format!("Imported {} governed {} row(s).", imported, label),
    })))
}

async fn rollback_governed_change(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataRollbackRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "governed_rollback",
            Some(payload.change_id),
            unavailable_message(&state, "governed rollback"),
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "governed_rollback",
            Some(payload.change_id),
            "Governed rollback requires an organization context.",
        ))));
    };

    let change = sqlx::query_as::<_, (String, String, i64, String)>(
        "SELECT config_area, target_table, target_record_id, change_type
         FROM governed_configuration_changes
         WHERE organization_id = $1 AND id = $2",
    )
    .bind(organization_id)
    .bind(payload.change_id as i64)
    .fetch_optional(pool)
    .await;

    let Some((config_area, target_table, target_record_id, change_type)) = (match change {
        Ok(value) => value,
        Err(error) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                "governed_rollback",
                Some(payload.change_id),
                format!("Rollback lookup failed: {}", humanize_db_error(&error)),
            ))));
        }
    }) else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "governed_rollback",
            Some(payload.change_id),
            "No governed change was found for rollback.",
        ))));
    };

    if !is_allowed_governed_table(&target_table) {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "governed_rollback",
            Some(payload.change_id),
            "Rollback target is not an approved governed master-data table.",
        ))));
    }

    let today = Local::now().date_naive();
    let rollback_result = if change_type == "archive" {
        sqlx::query(&format!(
            "UPDATE {}
             SET is_active = TRUE,
                 effective_to = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1 AND id = $2",
            target_table
        ))
        .bind(organization_id)
        .bind(target_record_id)
        .execute(pool)
        .await
    } else {
        sqlx::query(&format!(
            "UPDATE {}
             SET is_active = FALSE,
                 effective_to = COALESCE(effective_to, $3),
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1 AND id = $2",
            target_table
        ))
        .bind(organization_id)
        .bind(target_record_id)
        .bind(today)
        .execute(pool)
        .await
    };

    match rollback_result {
        Ok(result) if result.rows_affected() > 0 => {
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                &config_area,
                &target_table,
                target_record_id,
                "rollback",
                Some(session.user.id),
                format!(
                    "{} rolled back governed change #{} ({}) on {} #{}.",
                    session.user.name,
                    payload.change_id,
                    change_type,
                    target_table,
                    target_record_id
                ),
                today,
                Some(today),
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: "governed_rollback".into(),
                row_id: Some(target_record_id.max(0) as u64),
                message: format!(
                    "Rolled back governed change #{} for {} #{}.",
                    payload.change_id, target_table, target_record_id
                ),
            })))
        }
        Ok(_) => Ok(Json(ApiResponse::ok(failed_mutation(
            "governed_rollback",
            Some(payload.change_id),
            "Rollback target row was not changed.",
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "governed_rollback",
            Some(payload.change_id),
            format!("Rollback failed: {}", humanize_db_error(&error)),
        )))),
    }
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

async fn upsert_country_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CountryUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let name = match validate_name(&payload.name) {
        Ok(name) => name,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                "countries",
                payload.id,
                message,
            ))));
        }
    };
    let iso_code = payload
        .iso_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_uppercase);

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "countries",
            payload.id,
            unavailable_message(&state, "country saves"),
        ))));
    };

    match db::master_data::upsert_country(
        pool,
        payload.id.map(|value| value as i64),
        &name,
        iso_code.as_deref(),
    )
    .await
    {
        Ok(row) => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: "countries".into(),
            row_id: Some(row.id.max(0) as u64),
            message: success_message(&session, payload.id.is_some(), "country", &row.name),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "countries",
            payload.id,
            format!("Country save failed: {}", humanize_db_error(&error)),
        )))),
    }
}

async fn upsert_city_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CityUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(&state, &headers).await?;
    let name = match validate_name(&payload.name) {
        Ok(name) => name,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                "cities", payload.id, message,
            ))));
        }
    };

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "cities",
            payload.id,
            unavailable_message(&state, "city saves"),
        ))));
    };

    match db::master_data::upsert_city(
        pool,
        payload.id.map(|value| value as i64),
        &name,
        payload.country_id as i64,
    )
    .await
    {
        Ok(row) => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: "cities".into(),
            row_id: Some(row.id.max(0) as u64),
            message: success_message(&session, payload.id.is_some(), "city", &row.name),
        }))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "cities",
            payload.id,
            format!("City save failed: {}", humanize_db_error(&error)),
        )))),
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

async fn upsert_service_level_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GovernedCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_governed_catalog_row(&state, &headers, "service_levels", payload).await
}

async fn upsert_rejection_reason_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GovernedCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_governed_catalog_row(&state, &headers, "rejection_reasons", payload).await
}

async fn upsert_exception_reason_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GovernedCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_governed_catalog_row(&state, &headers, "exception_reasons", payload).await
}

async fn upsert_trailer_type_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GovernedCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_governed_catalog_row(&state, &headers, "trailer_types", payload).await
}

async fn upsert_hazmat_class_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GovernedCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_governed_catalog_row(&state, &headers, "hazmat_classes", payload).await
}

async fn upsert_accessorial_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GovernedCatalogUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_governed_catalog_row(&state, &headers, "accessorials", payload).await
}

async fn upsert_document_requirement_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DocumentRequirementRuleUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_document_requirement_rule(&state, &headers, payload).await
}

async fn upsert_customer_configuration_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CustomerConfigurationRuleUpsertRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    upsert_customer_configuration_rule(&state, &headers, payload).await
}

async fn delete_country_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_hard_row(&state, &headers, "countries", "country", payload.id).await
}

async fn delete_city_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_hard_row(&state, &headers, "cities", "city", payload.id).await
}

async fn delete_load_type_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_soft_row(&state, &headers, "load_types", "load type", payload.id).await
}

async fn delete_equipment_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_soft_row(&state, &headers, "equipments", "equipment", payload.id).await
}

async fn delete_commodity_type_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_soft_row(
        &state,
        &headers,
        "commodity_types",
        "commodity type",
        payload.id,
    )
    .await
}

async fn delete_location_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_soft_row(&state, &headers, "locations", "location", payload.id).await
}

async fn delete_service_level_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_governed_catalog_row(&state, &headers, "service_levels", payload.id).await
}

async fn delete_rejection_reason_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_governed_catalog_row(&state, &headers, "rejection_reasons", payload.id).await
}

async fn delete_exception_reason_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_governed_catalog_row(&state, &headers, "exception_reasons", payload.id).await
}

async fn delete_trailer_type_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_governed_catalog_row(&state, &headers, "trailer_types", payload.id).await
}

async fn delete_hazmat_class_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_governed_catalog_row(&state, &headers, "hazmat_classes", payload.id).await
}

async fn delete_accessorial_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_governed_catalog_row(&state, &headers, "accessorials", payload.id).await
}

async fn delete_document_requirement_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_document_requirement_rule(&state, &headers, payload.id).await
}

async fn delete_customer_configuration_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MasterDataDeleteRequest>,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    delete_customer_configuration_rule(&state, &headers, payload.id).await
}

async fn delete_soft_row(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    label: &str,
    id: u64,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            unavailable_message(state, "master-data deletes"),
        ))));
    };

    match db::master_data::soft_delete_simple_catalog(pool, kind, id as i64).await {
        Ok(affected) if affected > 0 => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: kind.into(),
            row_id: Some(id),
            message: format!(
                "{} archived {} #{} from the Rust admin portal.",
                session.user.name, label, id
            ),
        }))),
        Ok(_) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            format!("No active {} was found for id #{}.", label, id),
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            format!("{} delete failed: {}", label, humanize_db_error(&error)),
        )))),
    }
}

async fn delete_hard_row(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    label: &str,
    id: u64,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            unavailable_message(state, "master-data deletes"),
        ))));
    };

    let result = match kind {
        "countries" => db::master_data::delete_country(pool, id as i64).await,
        "cities" => db::master_data::delete_city(pool, id as i64).await,
        _ => Err(sqlx::Error::Protocol(format!(
            "unsupported hard-delete kind: {}",
            kind
        ))),
    };

    match result {
        Ok(affected) if affected > 0 => Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
            success: true,
            kind: kind.into(),
            row_id: Some(id),
            message: format!(
                "{} deleted {} #{} from the Rust admin portal.",
                session.user.name, label, id
            ),
        }))),
        Ok(_) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            format!("No {} was found for id #{}.", label, id),
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            format!("{} delete failed: {}", label, humanize_db_error(&error)),
        )))),
    }
}

async fn upsert_governed_catalog_row(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    payload: GovernedCatalogUpsertRequest,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            payload.id,
            unavailable_message(state, "governed catalog saves"),
        ))));
    };
    let Some((table_name, label)) = governed_catalog_table(kind) else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            payload.id,
            "Unsupported governed catalog kind.",
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            payload.id,
            "Governed catalog saves require an organization context.",
        ))));
    };
    let code = match normalize_catalog_code(&payload.code) {
        Ok(code) => code,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                kind, payload.id, message,
            ))));
        }
    };
    let row_label = match validate_name(&payload.label) {
        Ok(label) => label,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                kind, payload.id, message,
            ))));
        }
    };
    let effective_from = parse_effective_date(payload.effective_from.as_deref())
        .unwrap_or_else(|| Local::now().date_naive());
    let effective_to = parse_effective_date(payload.effective_to.as_deref());
    let description = payload
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let row_id = if let Some(id) = payload.id {
        sqlx::query_scalar::<_, i64>(&format!(
            "UPDATE {}
             SET code = $3,
                 label = $4,
                 description = $5,
                 requires_approval = $6,
                 effective_from = $7,
                 effective_to = $8,
                 is_active = TRUE,
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1 AND id = $2
             RETURNING id",
            table_name
        ))
        .bind(organization_id)
        .bind(id as i64)
        .bind(&code)
        .bind(&row_label)
        .bind(description)
        .bind(payload.requires_approval)
        .bind(effective_from)
        .bind(effective_to)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_scalar::<_, i64>(&format!(
            "INSERT INTO {} (
                organization_id, code, label, description, requires_approval,
                is_active, effective_from, effective_to, created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, $5, TRUE, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (organization_id, code)
             DO UPDATE SET
                label = EXCLUDED.label,
                description = EXCLUDED.description,
                requires_approval = EXCLUDED.requires_approval,
                is_active = TRUE,
                effective_from = EXCLUDED.effective_from,
                effective_to = EXCLUDED.effective_to,
                updated_at = CURRENT_TIMESTAMP
             RETURNING id",
            table_name
        ))
        .bind(organization_id)
        .bind(&code)
        .bind(&row_label)
        .bind(description)
        .bind(payload.requires_approval)
        .bind(effective_from)
        .bind(effective_to)
        .fetch_optional(pool)
        .await
    };

    match row_id {
        Ok(Some(row_id)) => {
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                kind,
                table_name,
                row_id,
                if payload.id.is_some() {
                    "update"
                } else {
                    "upsert"
                },
                Some(session.user.id),
                format!(
                    "{} saved governed {} '{}' ({}) from the Rust admin portal.",
                    session.user.name, label, row_label, code
                ),
                effective_from,
                effective_to,
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: kind.into(),
                row_id: Some(row_id.max(0) as u64),
                message: format!(
                    "{} saved governed {} '{}' with approval flag {}.",
                    session.user.name,
                    label,
                    row_label,
                    if payload.requires_approval {
                        "on"
                    } else {
                        "off"
                    }
                ),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            payload.id,
            format!("No governed {} was found for this organization.", label),
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            payload.id,
            format!(
                "Governed {} save failed: {}",
                label,
                humanize_db_error(&error)
            ),
        )))),
    }
}

async fn delete_governed_catalog_row(
    state: &AppState,
    headers: &HeaderMap,
    kind: &str,
    id: u64,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            unavailable_message(state, "governed catalog archives"),
        ))));
    };
    let Some((table_name, label)) = governed_catalog_table(kind) else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            "Unsupported governed catalog kind.",
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            "Governed catalog archives require an organization context.",
        ))));
    };

    match sqlx::query_scalar::<_, i64>(&format!(
        "UPDATE {}
         SET is_active = FALSE,
             effective_to = COALESCE(effective_to, CURRENT_DATE),
             updated_at = CURRENT_TIMESTAMP
         WHERE organization_id = $1 AND id = $2 AND is_active = TRUE
         RETURNING id",
        table_name
    ))
    .bind(organization_id)
    .bind(id as i64)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row_id)) => {
            let today = Local::now().date_naive();
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                kind,
                table_name,
                row_id,
                "archive",
                Some(session.user.id),
                format!(
                    "{} archived governed {} #{} from the Rust admin portal.",
                    session.user.name, label, row_id
                ),
                today,
                Some(today),
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: kind.into(),
                row_id: Some(row_id.max(0) as u64),
                message: format!(
                    "{} archived governed {} #{}.",
                    session.user.name, label, row_id
                ),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            format!("No active governed {} was found for id #{}.", label, id),
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            kind,
            Some(id),
            format!(
                "Governed {} archive failed: {}",
                label,
                humanize_db_error(&error)
            ),
        )))),
    }
}

async fn upsert_document_requirement_rule(
    state: &AppState,
    headers: &HeaderMap,
    payload: DocumentRequirementRuleUpsertRequest,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            payload.id,
            unavailable_message(state, "document requirement saves"),
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            payload.id,
            "Document requirement saves require an organization context.",
        ))));
    };

    let rule_key = match normalize_catalog_code(&payload.rule_key) {
        Ok(value) => value,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                "document_requirements",
                payload.id,
                message,
            ))));
        }
    };
    let label = match validate_name(&payload.label) {
        Ok(value) => value,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                "document_requirements",
                payload.id,
                message,
            ))));
        }
    };
    let requirement_scope =
        normalize_catalog_code(&payload.requirement_scope).unwrap_or_else(|_| "load".into());
    let lifecycle_state =
        normalize_catalog_code(&payload.lifecycle_state).unwrap_or_else(|_| "booking".into());
    let document_type_key =
        normalize_catalog_code(&payload.document_type_key).unwrap_or_else(|_| "standard".into());
    let role_key = payload
        .role_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| normalize_catalog_code(value).ok());
    let effective_from = parse_effective_date(payload.effective_from.as_deref())
        .unwrap_or_else(|| Local::now().date_naive());
    let effective_to = parse_effective_date(payload.effective_to.as_deref());

    let row_id = if let Some(id) = payload.id {
        sqlx::query_scalar::<_, i64>(
            "UPDATE required_document_rules
             SET rule_key = $3,
                 label = $4,
                 requirement_scope = $5,
                 role_key = $6,
                 lifecycle_state = $7,
                 document_type_key = $8,
                 blocks_transition = $9,
                 requires_approval = $10,
                 effective_from = $11,
                 effective_to = $12,
                 is_active = TRUE,
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1 AND id = $2
             RETURNING id",
        )
        .bind(organization_id)
        .bind(id as i64)
        .bind(&rule_key)
        .bind(&label)
        .bind(&requirement_scope)
        .bind(&role_key)
        .bind(&lifecycle_state)
        .bind(&document_type_key)
        .bind(payload.blocks_transition)
        .bind(payload.requires_approval)
        .bind(effective_from)
        .bind(effective_to)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO required_document_rules (
                rule_key, label, requirement_scope, role_key, organization_id,
                lifecycle_state, document_type_key, blocks_transition,
                requires_approval, effective_from, effective_to, is_active,
                created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (rule_key)
             DO UPDATE SET
                label = EXCLUDED.label,
                requirement_scope = EXCLUDED.requirement_scope,
                role_key = EXCLUDED.role_key,
                organization_id = EXCLUDED.organization_id,
                lifecycle_state = EXCLUDED.lifecycle_state,
                document_type_key = EXCLUDED.document_type_key,
                blocks_transition = EXCLUDED.blocks_transition,
                requires_approval = EXCLUDED.requires_approval,
                effective_from = EXCLUDED.effective_from,
                effective_to = EXCLUDED.effective_to,
                is_active = TRUE,
                updated_at = CURRENT_TIMESTAMP
             RETURNING id",
        )
        .bind(&rule_key)
        .bind(&label)
        .bind(&requirement_scope)
        .bind(&role_key)
        .bind(organization_id)
        .bind(&lifecycle_state)
        .bind(&document_type_key)
        .bind(payload.blocks_transition)
        .bind(payload.requires_approval)
        .bind(effective_from)
        .bind(effective_to)
        .fetch_optional(pool)
        .await
    };

    match row_id {
        Ok(Some(row_id)) => {
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                "document_requirements",
                "required_document_rules",
                row_id,
                if payload.id.is_some() {
                    "update"
                } else {
                    "upsert"
                },
                Some(session.user.id),
                format!(
                    "{} saved required document rule '{}' for {} / {}.",
                    session.user.name, label, requirement_scope, lifecycle_state
                ),
                effective_from,
                effective_to,
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: "document_requirements".into(),
                row_id: Some(row_id.max(0) as u64),
                message: format!(
                    "{} saved required document rule '{}'.",
                    session.user.name, label
                ),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            payload.id,
            "No document requirement rule was found for this organization.",
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            payload.id,
            format!(
                "Document requirement save failed: {}",
                humanize_db_error(&error)
            ),
        )))),
    }
}

async fn delete_document_requirement_rule(
    state: &AppState,
    headers: &HeaderMap,
    id: u64,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            Some(id),
            unavailable_message(state, "document requirement archives"),
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            Some(id),
            "Document requirement archives require an organization context.",
        ))));
    };

    match sqlx::query_scalar::<_, i64>(
        "UPDATE required_document_rules
         SET is_active = FALSE,
             effective_to = COALESCE(effective_to, CURRENT_DATE),
             updated_at = CURRENT_TIMESTAMP
         WHERE organization_id = $1 AND id = $2 AND is_active = TRUE
         RETURNING id",
    )
    .bind(organization_id)
    .bind(id as i64)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row_id)) => {
            let today = Local::now().date_naive();
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                "document_requirements",
                "required_document_rules",
                row_id,
                "archive",
                Some(session.user.id),
                format!(
                    "{} archived required document rule #{}.",
                    session.user.name, row_id
                ),
                today,
                Some(today),
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: "document_requirements".into(),
                row_id: Some(row_id.max(0) as u64),
                message: format!(
                    "{} archived required document rule #{}.",
                    session.user.name, row_id
                ),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            Some(id),
            format!("No active document requirement was found for id #{}.", id),
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "document_requirements",
            Some(id),
            format!(
                "Document requirement archive failed: {}",
                humanize_db_error(&error)
            ),
        )))),
    }
}

async fn upsert_customer_configuration_rule(
    state: &AppState,
    headers: &HeaderMap,
    payload: CustomerConfigurationRuleUpsertRequest,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            payload.id,
            unavailable_message(state, "customer configuration saves"),
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            payload.id,
            "Customer configuration saves require an organization context.",
        ))));
    };
    let config_key = match normalize_catalog_code(&payload.config_key) {
        Ok(value) => value,
        Err(message) => {
            return Ok(Json(ApiResponse::ok(failed_mutation(
                "customer_configurations",
                payload.id,
                message,
            ))));
        }
    };
    let config_area =
        normalize_catalog_code(&payload.config_area).unwrap_or_else(|_| "general".into());
    let carrier_group_key = payload
        .carrier_group_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| normalize_catalog_code(value).ok());
    let required_reference_keys = payload
        .required_reference_keys
        .iter()
        .filter_map(|value| normalize_catalog_code(value).ok())
        .collect::<Vec<_>>();
    let effective_from = parse_effective_date(payload.effective_from.as_deref())
        .unwrap_or_else(|| Local::now().date_naive());
    let effective_to = parse_effective_date(payload.effective_to.as_deref());
    let empty_json = serde_json::json!({});

    let row_id = if let Some(id) = payload.id {
        sqlx::query_scalar::<_, i64>(
            "UPDATE customer_configuration_rules
             SET config_key = $3,
                 config_area = $4,
                 customer_contract_id = $5,
                 customer_contract_lane_id = $6,
                 facility_id = $7,
                 carrier_group_key = $8,
                 visibility_rule = $9,
                 compliance_gate = $10,
                 billing_rules = $11,
                 notification_rules = $12,
                 required_reference_keys = $13,
                 requires_approval = $14,
                 effective_from = $15,
                 effective_to = $16,
                 is_active = TRUE,
                 updated_at = CURRENT_TIMESTAMP
             WHERE organization_id = $1 AND id = $2
             RETURNING id",
        )
        .bind(organization_id)
        .bind(id as i64)
        .bind(&config_key)
        .bind(&config_area)
        .bind(payload.customer_contract_id.map(|value| value as i64))
        .bind(payload.customer_contract_lane_id.map(|value| value as i64))
        .bind(payload.facility_id.map(|value| value as i64))
        .bind(&carrier_group_key)
        .bind(payload.visibility_rule.as_ref().unwrap_or(&empty_json))
        .bind(payload.compliance_gate.as_ref().unwrap_or(&empty_json))
        .bind(payload.billing_rules.as_ref().unwrap_or(&empty_json))
        .bind(payload.notification_rules.as_ref().unwrap_or(&empty_json))
        .bind(&required_reference_keys)
        .bind(payload.requires_approval)
        .bind(effective_from)
        .bind(effective_to)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO customer_configuration_rules (
                organization_id, config_key, config_area, customer_contract_id,
                customer_contract_lane_id, facility_id, carrier_group_key,
                visibility_rule, compliance_gate, billing_rules, notification_rules,
                required_reference_keys, requires_approval, effective_from, effective_to,
                is_active, created_at, updated_at
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             ON CONFLICT (organization_id, config_key)
             DO UPDATE SET
                config_area = EXCLUDED.config_area,
                customer_contract_id = EXCLUDED.customer_contract_id,
                customer_contract_lane_id = EXCLUDED.customer_contract_lane_id,
                facility_id = EXCLUDED.facility_id,
                carrier_group_key = EXCLUDED.carrier_group_key,
                visibility_rule = EXCLUDED.visibility_rule,
                compliance_gate = EXCLUDED.compliance_gate,
                billing_rules = EXCLUDED.billing_rules,
                notification_rules = EXCLUDED.notification_rules,
                required_reference_keys = EXCLUDED.required_reference_keys,
                requires_approval = EXCLUDED.requires_approval,
                effective_from = EXCLUDED.effective_from,
                effective_to = EXCLUDED.effective_to,
                is_active = TRUE,
                updated_at = CURRENT_TIMESTAMP
             RETURNING id",
        )
        .bind(organization_id)
        .bind(&config_key)
        .bind(&config_area)
        .bind(payload.customer_contract_id.map(|value| value as i64))
        .bind(payload.customer_contract_lane_id.map(|value| value as i64))
        .bind(payload.facility_id.map(|value| value as i64))
        .bind(&carrier_group_key)
        .bind(payload.visibility_rule.as_ref().unwrap_or(&empty_json))
        .bind(payload.compliance_gate.as_ref().unwrap_or(&empty_json))
        .bind(payload.billing_rules.as_ref().unwrap_or(&empty_json))
        .bind(payload.notification_rules.as_ref().unwrap_or(&empty_json))
        .bind(&required_reference_keys)
        .bind(payload.requires_approval)
        .bind(effective_from)
        .bind(effective_to)
        .fetch_optional(pool)
        .await
    };

    match row_id {
        Ok(Some(row_id)) => {
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                "customer_configurations",
                "customer_configuration_rules",
                row_id,
                if payload.id.is_some() {
                    "update"
                } else {
                    "upsert"
                },
                Some(session.user.id),
                format!(
                    "{} saved customer configuration '{}' in area '{}'.",
                    session.user.name, config_key, config_area
                ),
                effective_from,
                effective_to,
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: "customer_configurations".into(),
                row_id: Some(row_id.max(0) as u64),
                message: format!(
                    "{} saved customer configuration '{}'.",
                    session.user.name, config_key
                ),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            payload.id,
            "No customer configuration was found for this organization.",
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            payload.id,
            format!(
                "Customer configuration save failed: {}",
                humanize_db_error(&error)
            ),
        )))),
    }
}

async fn delete_customer_configuration_rule(
    state: &AppState,
    headers: &HeaderMap,
    id: u64,
) -> Result<Json<ApiResponse<MasterDataMutationResponse>>, StatusCode> {
    let session = require_master_data_access(state, headers).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            Some(id),
            unavailable_message(state, "customer configuration archives"),
        ))));
    };
    let Some(organization_id) = master_data_organization_id(pool, &session).await else {
        return Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            Some(id),
            "Customer configuration archives require an organization context.",
        ))));
    };

    match sqlx::query_scalar::<_, i64>(
        "UPDATE customer_configuration_rules
         SET is_active = FALSE,
             effective_to = COALESCE(effective_to, CURRENT_DATE),
             updated_at = CURRENT_TIMESTAMP
         WHERE organization_id = $1 AND id = $2 AND is_active = TRUE
         RETURNING id",
    )
    .bind(organization_id)
    .bind(id as i64)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row_id)) => {
            let today = Local::now().date_naive();
            let _ = record_governed_configuration_change(
                pool,
                organization_id,
                "customer_configurations",
                "customer_configuration_rules",
                row_id,
                "archive",
                Some(session.user.id),
                format!(
                    "{} archived customer configuration #{}.",
                    session.user.name, row_id
                ),
                today,
                Some(today),
            )
            .await;
            Ok(Json(ApiResponse::ok(MasterDataMutationResponse {
                success: true,
                kind: "customer_configurations".into(),
                row_id: Some(row_id.max(0) as u64),
                message: format!(
                    "{} archived customer configuration #{}.",
                    session.user.name, row_id
                ),
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            Some(id),
            format!("No active customer configuration was found for id #{}.", id),
        )))),
        Err(error) => Ok(Json(ApiResponse::ok(failed_mutation(
            "customer_configurations",
            Some(id),
            format!(
                "Customer configuration archive failed: {}",
                humanize_db_error(&error)
            ),
        )))),
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

    let mut sections = descriptors
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
                        editable: true,
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
                        editable: true,
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

    for (table, key, label) in [
        ("service_level_catalog", "service_levels", "Service levels"),
        (
            "rejection_reason_catalog",
            "rejection_reasons",
            "Rejection reasons",
        ),
        (
            "exception_reason_catalog",
            "exception_reasons",
            "Exception reasons",
        ),
        ("trailer_type_catalog", "trailer_types", "Trailer types"),
        ("hazmat_class_catalog", "hazmat_classes", "Hazmat classes"),
        ("accessorial_catalog", "accessorials", "Accessorial catalog"),
    ] {
        match governed_catalog_section(pool, table, key, label).await {
            Ok(section) => sections.push(section),
            Err(error) => warn!(error = %error, table, "failed to load governed catalog section"),
        }
    }
    match document_requirement_section(pool).await {
        Ok(section) => sections.push(section),
        Err(error) => warn!(error = %error, "failed to load document requirement section"),
    }
    match customer_configuration_section(pool).await {
        Ok(section) => sections.push(section),
        Err(error) => warn!(error = %error, "failed to load customer configuration section"),
    }

    let summary_cards = sections
        .iter()
        .map(|section| MasterDataSummaryCard {
            key: section.key.clone(),
            label: section.label.clone(),
            total: section.total,
            admin_route: section.admin_route.clone(),
            note: if governed_catalog_table(&section.key).is_some()
                || section.key == "document_requirements"
                || section.key == "customer_configurations"
            {
                "Governed catalog with active/effective dates and approval flags.".into()
            } else {
                format!("{} rows available.", section.label)
            },
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
        "This master-data route is now DB-backed and writable for countries, cities, locations, load types, equipments, and commodity types in the Rust admin portal.".into(),
        "Load and offer statuses remain read-first because they drive canonical workflow state machines.".into(),
        "Governed service-level, rejection-reason, and exception-reason catalogs are visible with effective-date and approval flags; write expansion must capture approval and rollback evidence.".into(),
        "Trailer type, hazmat class, and accessorial catalogs are now governed through the same ledger-backed change model.".into(),
        "Required document rules are governed through admin writes, safe archive, and configuration ledger evidence.".into(),
        "Customer-specific configuration rules are governed for visibility, compliance, billing, notifications, and required references.".into(),
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

async fn governed_catalog_section(
    pool: &db::DbPool,
    table_name: &str,
    key: &str,
    label: &str,
) -> Result<MasterDataSection, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, bool, bool, NaiveDate, Option<NaiveDate>)>(
        &format!(
            "SELECT id, code, label, description, requires_approval, is_active, effective_from, effective_to
             FROM {}
             ORDER BY is_active DESC, label ASC
             LIMIT 25",
            table_name
        ),
    )
    .fetch_all(pool)
    .await?;
    let total = rows.len() as u64;
    let rows = rows
        .into_iter()
        .take(6)
        .map(
            |(
                id,
                code,
                label,
                description,
                requires_approval,
                is_active,
                effective_from,
                effective_to,
            )| {
                MasterDataRow {
                    id: id.max(0) as u64,
                    primary_label: label,
                    secondary_label: Some(code),
                    status_label: match (is_active, requires_approval) {
                        (true, true) => "Active approval-gated".into(),
                        (true, false) => "Active".into(),
                        (false, _) => "Inactive".into(),
                    },
                    detail: format!(
                        "{} Effective {}{}.",
                        description.unwrap_or_else(|| "Governed configuration value.".into()),
                        effective_from,
                        effective_to
                            .map(|date| format!(" to {}", date))
                            .unwrap_or_default()
                    ),
                    editable: false,
                    country_id: None,
                    city_id: None,
                }
            },
        )
        .collect::<Vec<_>>();

    Ok(MasterDataSection {
        key: key.into(),
        label: label.into(),
        admin_route: "/admin/master-data".into(),
        total,
        rows,
        empty_message: format!("No {} are configured yet.", label.to_ascii_lowercase()),
    })
}

async fn document_requirement_section(pool: &db::DbPool) -> Result<MasterDataSection, sqlx::Error> {
    let rows = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            String,
            Option<String>,
            String,
            String,
            bool,
            bool,
            bool,
        ),
    >(
        "SELECT id, rule_key, label, requirement_scope, role_key, lifecycle_state,
                document_type_key, blocks_transition, requires_approval, is_active
         FROM required_document_rules
         ORDER BY is_active DESC, requirement_scope ASC, lifecycle_state ASC, label ASC
         LIMIT 25",
    )
    .fetch_all(pool)
    .await?;
    let total = rows.len() as u64;
    let rows = rows
        .into_iter()
        .take(6)
        .map(
            |(
                id,
                rule_key,
                label,
                requirement_scope,
                role_key,
                lifecycle_state,
                document_type_key,
                blocks_transition,
                requires_approval,
                is_active,
            )| MasterDataRow {
                id: id.max(0) as u64,
                primary_label: label,
                secondary_label: Some(rule_key),
                status_label: match (is_active, requires_approval, blocks_transition) {
                    (false, _, _) => "Inactive".into(),
                    (true, true, true) => "Blocking approval-gated".into(),
                    (true, false, true) => "Blocking".into(),
                    (true, true, false) => "Optional approval-gated".into(),
                    (true, false, false) => "Optional".into(),
                },
                detail: format!(
                    "{} / {} / {}{}",
                    requirement_scope,
                    lifecycle_state,
                    document_type_key,
                    role_key
                        .map(|role| format!(" / role {}", role))
                        .unwrap_or_default()
                ),
                editable: true,
                country_id: None,
                city_id: None,
            },
        )
        .collect::<Vec<_>>();

    Ok(MasterDataSection {
        key: "document_requirements".into(),
        label: "Document requirements".into(),
        admin_route: "/admin/master-data".into(),
        total,
        rows,
        empty_message: "No document requirements are configured yet.".into(),
    })
}

async fn customer_configuration_section(
    pool: &db::DbPool,
) -> Result<MasterDataSection, sqlx::Error> {
    let rows = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
            bool,
            bool,
        ),
    >(
        "SELECT id, config_key, config_area, customer_contract_id, customer_contract_lane_id,
                facility_id, carrier_group_key, requires_approval, is_active
         FROM customer_configuration_rules
         ORDER BY is_active DESC, config_area ASC, config_key ASC
         LIMIT 25",
    )
    .fetch_all(pool)
    .await?;
    let total = rows.len() as u64;
    let rows = rows
        .into_iter()
        .take(6)
        .map(
            |(
                id,
                config_key,
                config_area,
                contract_id,
                lane_id,
                facility_id,
                carrier_group_key,
                requires_approval,
                is_active,
            )| MasterDataRow {
                id: id.max(0) as u64,
                primary_label: config_key,
                secondary_label: Some(config_area.clone()),
                status_label: match (is_active, requires_approval) {
                    (false, _) => "Inactive".into(),
                    (true, true) => "Active approval-gated".into(),
                    (true, false) => "Active".into(),
                },
                detail: format!(
                    "contract {:?} / lane {:?} / facility {:?}{}",
                    contract_id,
                    lane_id,
                    facility_id,
                    carrier_group_key
                        .map(|group| format!(" / carrier group {}", group))
                        .unwrap_or_default()
                ),
                editable: true,
                country_id: None,
                city_id: None,
            },
        )
        .collect::<Vec<_>>();

    Ok(MasterDataSection {
        key: "customer_configurations".into(),
        label: "Customer configurations".into(),
        admin_route: "/admin/master-data".into(),
        total,
        rows,
        empty_message: "No customer-specific configuration rules are configured yet.".into(),
    })
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

fn failed_mutation(
    kind: &str,
    row_id: Option<u64>,
    message: impl Into<String>,
) -> MasterDataMutationResponse {
    MasterDataMutationResponse {
        success: false,
        kind: kind.into(),
        row_id,
        message: message.into(),
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

fn governed_catalog_table(kind: &str) -> Option<(&'static str, &'static str)> {
    match kind {
        "service_levels" => Some(("service_level_catalog", "service level")),
        "rejection_reasons" => Some(("rejection_reason_catalog", "rejection reason")),
        "exception_reasons" => Some(("exception_reason_catalog", "exception reason")),
        "trailer_types" => Some(("trailer_type_catalog", "trailer type")),
        "hazmat_classes" => Some(("hazmat_class_catalog", "hazmat class")),
        "accessorials" => Some(("accessorial_catalog", "accessorial")),
        _ => None,
    }
}

fn is_governed_master_data_kind(kind: &str) -> bool {
    matches!(
        kind,
        "service_levels"
            | "rejection_reasons"
            | "exception_reasons"
            | "trailer_types"
            | "hazmat_classes"
            | "accessorials"
            | "document_requirements"
            | "customer_configurations"
    )
}

fn is_allowed_governed_table(table_name: &str) -> bool {
    matches!(
        table_name,
        "service_level_catalog"
            | "rejection_reason_catalog"
            | "exception_reason_catalog"
            | "trailer_type_catalog"
            | "hazmat_class_catalog"
            | "accessorial_catalog"
            | "required_document_rules"
            | "customer_configuration_rules"
    )
}

async fn master_data_organization_id(pool: &db::DbPool, session: &ResolvedSession) -> Option<i64> {
    if let Some(organization_id) = auth_session::session_organization_id(session) {
        return Some(organization_id);
    }
    sqlx::query_scalar::<_, i64>("SELECT id FROM organizations ORDER BY id LIMIT 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

fn normalize_catalog_code(code: &str) -> Result<String, String> {
    let normalized = code.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    if normalized.is_empty() {
        return Err("Enter a code before saving this governed catalog record.".into());
    }
    if normalized.len() > 64
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err("Codes may use lowercase letters, numbers, and underscores only.".into());
    }
    Ok(normalized)
}

fn parse_effective_date(value: Option<&str>) -> Option<NaiveDate> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
}

// Governed config audit rows must keep before/after and effective dates together
// so support can explain master-data changes without reconstructing context.
#[allow(clippy::too_many_arguments)]
async fn record_governed_configuration_change(
    pool: &db::DbPool,
    organization_id: i64,
    config_area: &str,
    target_table: &str,
    target_record_id: i64,
    change_type: &str,
    actor_user_id: Option<i64>,
    change_summary: String,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO governed_configuration_changes (
            organization_id, config_area, target_table, target_record_id, change_type,
            requested_by_user_id, approved_by_user_id, approval_status, rollback_payload,
            change_summary, effective_from, effective_to, created_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $6, 'approved', '{}'::jsonb, $7, $8, $9, CURRENT_TIMESTAMP)",
    )
    .bind(organization_id)
    .bind(config_area)
    .bind(target_table)
    .bind(target_record_id)
    .bind(change_type)
    .bind(actor_user_id)
    .bind(change_summary)
    .bind(effective_from)
    .bind(effective_to)
    .execute(pool)
    .await?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        auth_headers_for_user, insert_user_with_role_status, prepare_pool, test_state,
    };
    use domain::auth::{AccountStatus, UserRole};
    use serial_test::serial;
    use shared::{
        CityUpsertRequest, CountryUpsertRequest, CustomerConfigurationRuleUpsertRequest,
        DocumentRequirementRuleUpsertRequest, GovernedCatalogUpsertRequest, LocationUpsertRequest,
        MasterDataImportRequest, MasterDataRollbackRequest, SimpleCatalogUpsertRequest,
    };

    #[tokio::test]
    #[serial]
    async fn master_data_screen_enforces_access_and_returns_db_sections()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Admin Master Data",
            "admin-master-data@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let shipper_user = insert_user_with_role_status(
            &pool,
            "Shipper Viewer",
            "shipper-master-data@example.com",
            UserRole::Shipper,
            AccountStatus::Approved,
        )
        .await?;
        let admin_headers = auth_headers_for_user(&state, &admin_user).await?;
        let shipper_headers = auth_headers_for_user(&state, &shipper_user).await?;

        let unauthenticated = screen(State(state.clone()), HeaderMap::new()).await;
        assert_eq!(unauthenticated.unwrap_err(), StatusCode::UNAUTHORIZED);

        let forbidden = screen(State(state.clone()), shipper_headers).await;
        assert_eq!(forbidden.unwrap_err(), StatusCode::FORBIDDEN);

        let screen_payload = screen(State(state), admin_headers)
            .await
            .expect("admin master-data screen should load")
            .0
            .data;

        assert!(
            screen_payload
                .summary_cards
                .iter()
                .any(|card| card.label.contains("Writable catalogs"))
        );
        assert!(
            screen_payload
                .sections
                .iter()
                .any(|section| section.key == "countries")
        );
        assert!(
            screen_payload
                .sections
                .iter()
                .any(|section| section.key == "load_statuses")
        );
        assert!(
            screen_payload
                .sections
                .iter()
                .any(|section| section.key == "service_levels")
        );

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn master_data_route_handlers_cover_crud_and_archive_flows()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Admin Catalog Editor",
            "admin-catalog-editor@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let admin_headers = auth_headers_for_user(&state, &admin_user).await?;

        let country_response = upsert_country_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(CountryUpsertRequest {
                id: None,
                name: "United States".into(),
                iso_code: Some("us".into()),
            }),
        )
        .await
        .expect("country create should succeed")
        .0
        .data;
        assert!(country_response.success);
        let country_id = country_response.row_id.expect("country id");

        let city_response = upsert_city_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(CityUpsertRequest {
                id: None,
                name: "Dallas".into(),
                country_id,
            }),
        )
        .await
        .expect("city create should succeed")
        .0
        .data;
        assert!(city_response.success);
        let city_id = city_response.row_id.expect("city id");

        let load_type_response = upsert_load_type_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(SimpleCatalogUpsertRequest {
                id: None,
                name: "Full Truckload".into(),
            }),
        )
        .await
        .expect("load type create should succeed")
        .0
        .data;
        assert!(load_type_response.success);
        let load_type_id = load_type_response.row_id.expect("load type id");

        let equipment_response = upsert_equipment_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(SimpleCatalogUpsertRequest {
                id: None,
                name: "Dry Van".into(),
            }),
        )
        .await
        .expect("equipment create should succeed")
        .0
        .data;
        assert!(equipment_response.success);
        let equipment_id = equipment_response.row_id.expect("equipment id");

        let commodity_response = upsert_commodity_type_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(SimpleCatalogUpsertRequest {
                id: None,
                name: "Paper Goods".into(),
            }),
        )
        .await
        .expect("commodity type create should succeed")
        .0
        .data;
        assert!(commodity_response.success);
        let commodity_id = commodity_response.row_id.expect("commodity id");

        let location_response = upsert_location_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(LocationUpsertRequest {
                id: None,
                name: "Dallas Yard".into(),
                city_id: Some(city_id),
                country_id: Some(country_id),
            }),
        )
        .await
        .expect("location create should succeed")
        .0
        .data;
        assert!(location_response.success);
        let location_id = location_response.row_id.expect("location id");

        let update_equipment = upsert_equipment_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(SimpleCatalogUpsertRequest {
                id: Some(equipment_id),
                name: "53 ft Dry Van".into(),
            }),
        )
        .await
        .expect("equipment update should succeed")
        .0
        .data;
        assert!(update_equipment.success);
        assert!(update_equipment.message.contains("updated"));

        let update_location = upsert_location_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(LocationUpsertRequest {
                id: Some(location_id),
                name: "Dallas Main Yard".into(),
                city_id: Some(city_id),
                country_id: Some(country_id),
            }),
        )
        .await
        .expect("location update should succeed")
        .0
        .data;
        assert!(update_location.success);

        let governed_response = upsert_service_level_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(GovernedCatalogUpsertRequest {
                id: None,
                code: "white_glove".into(),
                label: "White Glove".into(),
                description: Some("Requires operations approval before tender.".into()),
                requires_approval: true,
                effective_from: None,
                effective_to: None,
            }),
        )
        .await
        .expect("governed service level create should succeed")
        .0
        .data;
        assert!(governed_response.success, "{}", governed_response.message);
        let governed_id = governed_response.row_id.expect("governed row id");

        let archive_governed = delete_service_level_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: governed_id }),
        )
        .await
        .expect("governed service level archive should succeed")
        .0
        .data;
        assert!(archive_governed.success, "{}", archive_governed.message);

        let governed_archive_change_id = sqlx::query_scalar::<_, i64>(
            "SELECT id
             FROM governed_configuration_changes
             WHERE target_table = 'service_level_catalog'
               AND target_record_id = $1
               AND change_type = 'archive'
             ORDER BY id DESC
             LIMIT 1",
        )
        .bind(governed_id as i64)
        .fetch_one(&pool)
        .await?;
        let rollback_governed = rollback_governed_change(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataRollbackRequest {
                change_id: governed_archive_change_id as u64,
            }),
        )
        .await
        .expect("governed rollback should succeed")
        .0
        .data;
        assert!(rollback_governed.success, "{}", rollback_governed.message);
        let restored_governed_active = sqlx::query_scalar::<_, bool>(
            "SELECT is_active FROM service_level_catalog WHERE id = $1",
        )
        .bind(governed_id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(restored_governed_active);

        let import_dry_run = import_governed_master_data(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataImportRequest {
                kind: "accessorials".into(),
                dry_run: true,
                rows: vec![GovernedCatalogUpsertRequest {
                    id: None,
                    code: "inside_delivery_test".into(),
                    label: "Inside delivery test".into(),
                    description: Some("Dry-run import row.".into()),
                    requires_approval: true,
                    effective_from: None,
                    effective_to: None,
                }],
            }),
        )
        .await
        .expect("governed dry-run import should succeed")
        .0
        .data;
        assert!(import_dry_run.success, "{}", import_dry_run.message);

        let import_response = import_governed_master_data(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataImportRequest {
                kind: "accessorials".into(),
                dry_run: false,
                rows: vec![GovernedCatalogUpsertRequest {
                    id: None,
                    code: "inside_delivery_test".into(),
                    label: "Inside delivery test".into(),
                    description: Some("Imported accessorial row.".into()),
                    requires_approval: true,
                    effective_from: None,
                    effective_to: None,
                }],
            }),
        )
        .await
        .expect("governed import should succeed")
        .0
        .data;
        assert!(import_response.success, "{}", import_response.message);

        let export_response =
            export_governed_master_data(State(state.clone()), admin_headers.clone())
                .await
                .expect("governed export should succeed")
                .0
                .data;
        assert!(
            export_response
                .sections
                .iter()
                .any(|section| section.key == "accessorials")
        );

        let hazmat_response = upsert_hazmat_class_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(GovernedCatalogUpsertRequest {
                id: None,
                code: "class_9_test".into(),
                label: "Class 9 Test".into(),
                description: Some("Miscellaneous regulated material test row.".into()),
                requires_approval: true,
                effective_from: None,
                effective_to: None,
            }),
        )
        .await
        .expect("governed hazmat create should succeed")
        .0
        .data;
        assert!(hazmat_response.success, "{}", hazmat_response.message);
        let hazmat_id = hazmat_response.row_id.expect("hazmat row id");

        let archive_hazmat = delete_hazmat_class_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: hazmat_id }),
        )
        .await
        .expect("governed hazmat archive should succeed")
        .0
        .data;
        assert!(archive_hazmat.success, "{}", archive_hazmat.message);

        let document_rule_response = upsert_document_requirement_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(DocumentRequirementRuleUpsertRequest {
                id: None,
                rule_key: "test_rate_confirmation".into(),
                label: "Test rate confirmation".into(),
                requirement_scope: "load".into(),
                role_key: None,
                lifecycle_state: "booking".into(),
                document_type_key: "rate_confirmation".into(),
                blocks_transition: true,
                requires_approval: true,
                effective_from: None,
                effective_to: None,
            }),
        )
        .await
        .expect("document rule create should succeed")
        .0
        .data;
        assert!(
            document_rule_response.success,
            "{}",
            document_rule_response.message
        );
        let document_rule_id = document_rule_response.row_id.expect("document rule id");

        let archive_document_rule = delete_document_requirement_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest {
                id: document_rule_id,
            }),
        )
        .await
        .expect("document rule archive should succeed")
        .0
        .data;
        assert!(
            archive_document_rule.success,
            "{}",
            archive_document_rule.message
        );

        let customer_config_response = upsert_customer_configuration_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(CustomerConfigurationRuleUpsertRequest {
                id: None,
                config_key: "test_customer_visibility".into(),
                config_area: "visibility".into(),
                customer_contract_id: None,
                customer_contract_lane_id: None,
                facility_id: None,
                carrier_group_key: Some("preferred_test".into()),
                visibility_rule: Some(serde_json::json!({ "posting_behavior": "contract" })),
                compliance_gate: Some(serde_json::json!({ "require_compliance_ready": true })),
                billing_rules: Some(serde_json::json!({ "currency": "USD" })),
                notification_rules: Some(serde_json::json!({ "tender": true })),
                required_reference_keys: vec!["po_number".into(), "customer_reference".into()],
                requires_approval: true,
                effective_from: None,
                effective_to: None,
            }),
        )
        .await
        .expect("customer configuration create should succeed")
        .0
        .data;
        assert!(
            customer_config_response.success,
            "{}",
            customer_config_response.message
        );
        let customer_config_id = customer_config_response.row_id.expect("customer config id");

        let archive_customer_config = delete_customer_configuration_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest {
                id: customer_config_id,
            }),
        )
        .await
        .expect("customer configuration archive should succeed")
        .0
        .data;
        assert!(
            archive_customer_config.success,
            "{}",
            archive_customer_config.message
        );

        let archive_load_type = delete_load_type_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: load_type_id }),
        )
        .await
        .expect("load type archive should succeed")
        .0
        .data;
        assert!(archive_load_type.success);

        let archive_equipment = delete_equipment_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: equipment_id }),
        )
        .await
        .expect("equipment archive should succeed")
        .0
        .data;
        assert!(archive_equipment.success);

        let archive_commodity = delete_commodity_type_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: commodity_id }),
        )
        .await
        .expect("commodity archive should succeed")
        .0
        .data;
        assert!(archive_commodity.success);

        let archive_location = delete_location_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: location_id }),
        )
        .await
        .expect("location archive should succeed")
        .0
        .data;
        assert!(archive_location.success);

        let delete_city = delete_city_handler(
            State(state.clone()),
            admin_headers.clone(),
            Json(MasterDataDeleteRequest { id: city_id }),
        )
        .await
        .expect("city delete should succeed")
        .0
        .data;
        assert!(delete_city.success);

        let delete_country = delete_country_handler(
            State(state),
            admin_headers,
            Json(MasterDataDeleteRequest { id: country_id }),
        )
        .await
        .expect("country delete should succeed")
        .0
        .data;
        assert!(delete_country.success);

        assert!(db::master_data::list_countries(&pool).await?.is_empty());
        assert!(db::master_data::list_cities(&pool).await?.is_empty());
        assert!(db::master_data::list_load_types(&pool).await?.is_empty());
        assert!(db::master_data::list_equipments(&pool).await?.is_empty());
        assert!(
            db::master_data::list_commodity_types(&pool)
                .await?
                .is_empty()
        );
        assert!(db::master_data::list_locations(&pool).await?.is_empty());

        let archived_equipment_name =
            sqlx::query_scalar::<_, String>("SELECT name FROM equipments WHERE id = $1")
                .bind(equipment_id as i64)
                .fetch_one(&pool)
                .await?;
        assert_eq!(archived_equipment_name, "53 ft Dry Van");

        let archived_location_name =
            sqlx::query_scalar::<_, String>("SELECT name FROM locations WHERE id = $1")
                .bind(location_id as i64)
                .fetch_one(&pool)
                .await?;
        assert_eq!(archived_location_name, "Dallas Main Yard");

        let governed_change_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM governed_configuration_changes
             WHERE target_table = 'service_level_catalog'
               AND target_record_id = $1",
        )
        .bind(governed_id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(governed_change_count >= 2);

        let rollback_change_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM governed_configuration_changes
             WHERE target_table = 'service_level_catalog'
               AND target_record_id = $1
               AND change_type = 'rollback'",
        )
        .bind(governed_id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(rollback_change_count >= 1);

        let import_change_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM governed_configuration_changes
             WHERE target_table = 'accessorial_catalog'
               AND change_type = 'import'",
        )
        .fetch_one(&pool)
        .await?;
        assert!(import_change_count >= 1);

        let hazmat_change_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM governed_configuration_changes
             WHERE target_table = 'hazmat_class_catalog'
               AND target_record_id = $1",
        )
        .bind(hazmat_id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(hazmat_change_count >= 2);

        let document_rule_change_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM governed_configuration_changes
             WHERE target_table = 'required_document_rules'
               AND target_record_id = $1",
        )
        .bind(document_rule_id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(document_rule_change_count >= 2);

        let customer_config_change_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM governed_configuration_changes
             WHERE target_table = 'customer_configuration_rules'
               AND target_record_id = $1",
        )
        .bind(customer_config_id as i64)
        .fetch_one(&pool)
        .await?;
        assert!(customer_config_change_count >= 2);

        Ok(())
    }
}
