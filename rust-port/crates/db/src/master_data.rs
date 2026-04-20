use crate::{
    DbPool,
    dispatch::{
        CityRecord, CommodityTypeRecord, CountryRecord, EquipmentRecord, LoadStatusMasterRecord,
        LoadTypeRecord, LocationRecord,
    },
    marketplace::OfferStatusMasterRecord,
};
use domain::master_data::MasterDataKind;

pub const fn backing_table(kind: MasterDataKind) -> &'static str {
    match kind {
        MasterDataKind::Countries => "countries",
        MasterDataKind::Cities => "cities",
        MasterDataKind::Locations => "locations",
        MasterDataKind::LoadTypes => "load_types",
        MasterDataKind::Equipments => "equipments",
        MasterDataKind::CommodityTypes => "commodity_types",
        MasterDataKind::LoadStatuses => "load_status_master",
        MasterDataKind::OfferStatuses => "offer_status_master",
    }
}

pub async fn list_countries(pool: &DbPool) -> Result<Vec<CountryRecord>, sqlx::Error> {
    sqlx::query_as::<_, CountryRecord>(
        "SELECT id, name, iso_code, created_at, updated_at FROM countries ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_cities(pool: &DbPool) -> Result<Vec<CityRecord>, sqlx::Error> {
    sqlx::query_as::<_, CityRecord>(
        "SELECT id, country_id, name, created_at, updated_at FROM cities ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_cities_by_country(
    pool: &DbPool,
    country_id: i64,
) -> Result<Vec<CityRecord>, sqlx::Error> {
    sqlx::query_as::<_, CityRecord>(
        "SELECT id, country_id, name, created_at, updated_at FROM cities WHERE country_id = $1 ORDER BY name",
    )
    .bind(country_id)
    .fetch_all(pool)
    .await
}

pub async fn list_locations(pool: &DbPool) -> Result<Vec<LocationRecord>, sqlx::Error> {
    sqlx::query_as::<_, LocationRecord>(
        "SELECT id, name, city_id, country_id, created_at, updated_at, deleted_at
         FROM locations
         WHERE deleted_at IS NULL
         ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_load_types(pool: &DbPool) -> Result<Vec<LoadTypeRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadTypeRecord>(
        "SELECT id, name, created_at, updated_at, deleted_at
         FROM load_types
         WHERE deleted_at IS NULL
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_equipments(pool: &DbPool) -> Result<Vec<EquipmentRecord>, sqlx::Error> {
    sqlx::query_as::<_, EquipmentRecord>(
        "SELECT id, name, created_at, updated_at, deleted_at
         FROM equipments
         WHERE deleted_at IS NULL
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_commodity_types(pool: &DbPool) -> Result<Vec<CommodityTypeRecord>, sqlx::Error> {
    sqlx::query_as::<_, CommodityTypeRecord>(
        "SELECT id, name, created_at, updated_at, deleted_at
         FROM commodity_types
         WHERE deleted_at IS NULL
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_load_statuses(pool: &DbPool) -> Result<Vec<LoadStatusMasterRecord>, sqlx::Error> {
    sqlx::query_as::<_, LoadStatusMasterRecord>(
        "SELECT id, name, slug, description, sort_order, is_terminal
         FROM load_status_master
         ORDER BY sort_order, id",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_offer_statuses(
    pool: &DbPool,
) -> Result<Vec<OfferStatusMasterRecord>, sqlx::Error> {
    sqlx::query_as::<_, OfferStatusMasterRecord>(
        "SELECT id, name, slug, description, sort_order, is_terminal
         FROM offer_status_master
         ORDER BY sort_order, id",
    )
    .fetch_all(pool)
    .await
}

pub async fn upsert_load_type(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
) -> Result<LoadTypeRecord, sqlx::Error> {
    match id {
        Some(id) => {
            sqlx::query_as::<_, LoadTypeRecord>(
                "UPDATE load_types
                 SET name = $2,
                     updated_at = CURRENT_TIMESTAMP,
                     deleted_at = NULL
                 WHERE id = $1
                 RETURNING id, name, created_at, updated_at, deleted_at",
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, LoadTypeRecord>(
                "INSERT INTO load_types (name)
                 VALUES ($1)
                 RETURNING id, name, created_at, updated_at, deleted_at",
            )
            .bind(name)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn upsert_country(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
    iso_code: Option<&str>,
) -> Result<CountryRecord, sqlx::Error> {
    match id {
        Some(id) => {
            sqlx::query_as::<_, CountryRecord>(
                "UPDATE countries
                 SET name = $2,
                     iso_code = $3,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1
                 RETURNING id, name, iso_code, created_at, updated_at",
            )
            .bind(id)
            .bind(name)
            .bind(iso_code)
            .fetch_one(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, CountryRecord>(
                "INSERT INTO countries (name, iso_code, created_at, updated_at)
                 VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 RETURNING id, name, iso_code, created_at, updated_at",
            )
            .bind(name)
            .bind(iso_code)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn upsert_city(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
    country_id: i64,
) -> Result<CityRecord, sqlx::Error> {
    match id {
        Some(id) => {
            sqlx::query_as::<_, CityRecord>(
                "UPDATE cities
                 SET name = $2,
                     country_id = $3,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1
                 RETURNING id, country_id, name, created_at, updated_at",
            )
            .bind(id)
            .bind(name)
            .bind(country_id)
            .fetch_one(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, CityRecord>(
                "INSERT INTO cities (name, country_id, created_at, updated_at)
                 VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                 RETURNING id, country_id, name, created_at, updated_at",
            )
            .bind(name)
            .bind(country_id)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn upsert_equipment(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
) -> Result<EquipmentRecord, sqlx::Error> {
    match id {
        Some(id) => {
            sqlx::query_as::<_, EquipmentRecord>(
                "UPDATE equipments
                 SET name = $2,
                     updated_at = CURRENT_TIMESTAMP,
                     deleted_at = NULL
                 WHERE id = $1
                 RETURNING id, name, created_at, updated_at, deleted_at",
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, EquipmentRecord>(
                "INSERT INTO equipments (name)
                 VALUES ($1)
                 RETURNING id, name, created_at, updated_at, deleted_at",
            )
            .bind(name)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn upsert_commodity_type(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
) -> Result<CommodityTypeRecord, sqlx::Error> {
    match id {
        Some(id) => {
            sqlx::query_as::<_, CommodityTypeRecord>(
                "UPDATE commodity_types
                 SET name = $2,
                     updated_at = CURRENT_TIMESTAMP,
                     deleted_at = NULL
                 WHERE id = $1
                 RETURNING id, name, created_at, updated_at, deleted_at",
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, CommodityTypeRecord>(
                "INSERT INTO commodity_types (name)
                 VALUES ($1)
                 RETURNING id, name, created_at, updated_at, deleted_at",
            )
            .bind(name)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn upsert_location(
    pool: &DbPool,
    id: Option<i64>,
    name: &str,
    city_id: Option<i64>,
    country_id: Option<i64>,
) -> Result<LocationRecord, sqlx::Error> {
    match id {
        Some(id) => {
            sqlx::query_as::<_, LocationRecord>(
                "UPDATE locations
                 SET name = $2,
                     city_id = $3,
                     country_id = $4,
                     updated_at = CURRENT_TIMESTAMP,
                     deleted_at = NULL
                 WHERE id = $1
                 RETURNING id, name, city_id, country_id, created_at, updated_at, deleted_at",
            )
            .bind(id)
            .bind(name)
            .bind(city_id)
            .bind(country_id)
            .fetch_one(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, LocationRecord>(
                "INSERT INTO locations (name, city_id, country_id)
                 VALUES ($1, $2, $3)
                 RETURNING id, name, city_id, country_id, created_at, updated_at, deleted_at",
            )
            .bind(name)
            .bind(city_id)
            .bind(country_id)
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn find_country_by_name_like(
    pool: &DbPool,
    name: &str,
) -> Result<Option<CountryRecord>, sqlx::Error> {
    sqlx::query_as::<_, CountryRecord>(
        "SELECT id, name, iso_code, created_at, updated_at
         FROM countries
         WHERE name ILIKE $1
         ORDER BY LENGTH(name), id
         LIMIT 1",
    )
    .bind(format!("%{}%", name.trim()))
    .fetch_optional(pool)
    .await
}

pub async fn soft_delete_simple_catalog(
    pool: &DbPool,
    table_name: &str,
    id: i64,
) -> Result<u64, sqlx::Error> {
    let sql = match table_name {
        "load_types" => {
            "UPDATE load_types SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL"
        }
        "equipments" => {
            "UPDATE equipments SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL"
        }
        "commodity_types" => {
            "UPDATE commodity_types SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL"
        }
        "locations" => {
            "UPDATE locations SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL"
        }
        _ => {
            return Err(sqlx::Error::Protocol(
                format!("unsupported soft-delete table: {}", table_name).into(),
            ));
        }
    };

    Ok(sqlx::query(sql)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn delete_city(pool: &DbPool, id: i64) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM cities WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn delete_country(pool: &DbPool, id: i64) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM countries WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn ensure_country_by_name(
    pool: &DbPool,
    name: &str,
) -> Result<CountryRecord, sqlx::Error> {
    if let Some(country) = find_country_by_name_like(pool, name).await? {
        return Ok(country);
    }

    sqlx::query_as::<_, CountryRecord>(
        "INSERT INTO countries (name, iso_code, created_at, updated_at)
         VALUES ($1, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, name, iso_code, created_at, updated_at",
    )
    .bind(name.trim())
    .fetch_one(pool)
    .await
}

pub async fn find_city_by_name_like(
    pool: &DbPool,
    country_id: i64,
    name: &str,
) -> Result<Option<CityRecord>, sqlx::Error> {
    sqlx::query_as::<_, CityRecord>(
        "SELECT id, country_id, name, created_at, updated_at
         FROM cities
         WHERE country_id = $1 AND name ILIKE $2
         ORDER BY LENGTH(name), id
         LIMIT 1",
    )
    .bind(country_id)
    .bind(format!("%{}%", name.trim()))
    .fetch_optional(pool)
    .await
}

pub async fn ensure_city_by_name(
    pool: &DbPool,
    country_id: i64,
    name: &str,
) -> Result<CityRecord, sqlx::Error> {
    if let Some(city) = find_city_by_name_like(pool, country_id, name).await? {
        return Ok(city);
    }

    sqlx::query_as::<_, CityRecord>(
        "INSERT INTO cities (country_id, name, created_at, updated_at)
         VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, country_id, name, created_at, updated_at",
    )
    .bind(country_id)
    .bind(name.trim())
    .fetch_one(pool)
    .await
}
