use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataSummaryCard {
    pub key: String,
    pub label: String,
    pub total: u64,
    pub admin_route: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataRow {
    pub id: u64,
    pub primary_label: String,
    pub secondary_label: Option<String>,
    pub status_label: String,
    pub detail: String,
    pub editable: bool,
    pub country_id: Option<u64>,
    pub city_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataSection {
    pub key: String,
    pub label: String,
    pub admin_route: String,
    pub total: u64,
    pub rows: Vec<MasterDataRow>,
    pub empty_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataOption {
    pub id: u64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataCityOption {
    pub id: u64,
    pub country_id: u64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataScreen {
    pub title: String,
    pub subtitle: String,
    pub summary_cards: Vec<MasterDataSummaryCard>,
    pub sections: Vec<MasterDataSection>,
    pub country_options: Vec<MasterDataOption>,
    pub city_options: Vec<MasterDataCityOption>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleCatalogUpsertRequest {
    pub id: Option<u64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryUpsertRequest {
    pub id: Option<u64>,
    pub name: String,
    pub iso_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityUpsertRequest {
    pub id: Option<u64>,
    pub name: String,
    pub country_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationUpsertRequest {
    pub id: Option<u64>,
    pub name: String,
    pub country_id: Option<u64>,
    pub city_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataMutationResponse {
    pub success: bool,
    pub kind: String,
    pub row_id: Option<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDataDeleteRequest {
    pub id: u64,
}
