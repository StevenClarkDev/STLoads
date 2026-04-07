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
