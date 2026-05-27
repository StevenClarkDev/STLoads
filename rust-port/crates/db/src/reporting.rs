use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct BusinessMetricDefinitionRecord {
    pub metric_key: String,
    pub display_name: String,
    pub category: String,
    pub owner_team: String,
    pub business_definition: String,
    pub numerator_definition: Option<String>,
    pub denominator_definition: Option<String>,
    pub grain: String,
    pub refresh_cadence: String,
    pub target_direction: String,
    pub accepted_by: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ReportingReadModelRecord {
    pub model_key: String,
    pub display_name: String,
    pub source_tables: Vec<String>,
    pub target_table: String,
    pub refresh_strategy: String,
    pub refresh_cadence: String,
    pub owner_team: String,
    pub operational_screen_safe: bool,
    pub last_refresh_status: String,
    pub last_refreshed_at: Option<NaiveDateTime>,
    pub warehouse_export_enabled: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct CustomerScorecardRecord {
    pub id: i64,
    pub organization_id: i64,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub posted_loads: i32,
    pub booked_loads: i32,
    pub acceptance_rate: Option<f64>,
    pub on_time_pickup_rate: Option<f64>,
    pub on_time_delivery_rate: Option<f64>,
    pub document_cycle_minutes: Option<i32>,
    pub gross_margin_cents: Option<i64>,
    pub dispute_rate: Option<f64>,
    pub score: Option<f64>,
    pub score_tone: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CarrierScorecardRecord {
    pub id: i64,
    pub organization_id: i64,
    pub carrier_user_id: Option<i64>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub offered_loads: i32,
    pub accepted_loads: i32,
    pub acceptance_rate: Option<f64>,
    pub tracking_compliance_rate: Option<f64>,
    pub on_time_pickup_rate: Option<f64>,
    pub on_time_delivery_rate: Option<f64>,
    pub claims_rate: Option<f64>,
    pub document_quality_rate: Option<f64>,
    pub payout_cycle_hours: Option<i32>,
    pub score: Option<f64>,
    pub score_tone: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct LanePricingRecommendationRecord {
    pub id: i64,
    pub organization_id: i64,
    pub lane_key: String,
    pub equipment_type: Option<String>,
    pub recommended_rate_cents: i64,
    pub low_rate_cents: Option<i64>,
    pub high_rate_cents: Option<i64>,
    pub currency: String,
    pub confidence_score: f64,
    pub sample_size: i32,
    pub anomaly_status: String,
    pub recommendation_reason: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct GlobalSearchDocumentRecord {
    pub id: i64,
    pub organization_id: i64,
    pub entity_type: String,
    pub entity_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub href: Option<String>,
    pub permission_key: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct DataQualityRuleRecord {
    pub rule_key: String,
    pub category: String,
    pub severity: String,
    pub owner_team: String,
    pub cadence: String,
    pub alert_threshold: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct DataQualityFindingRecord {
    pub id: i64,
    pub organization_id: Option<i64>,
    pub rule_key: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub severity: String,
    pub finding_status: String,
    pub owner_team: String,
    pub title: String,
    pub detail: String,
}

pub async fn list_business_metric_definitions(
    pool: &DbPool,
) -> Result<Vec<BusinessMetricDefinitionRecord>, sqlx::Error> {
    sqlx::query_as::<_, BusinessMetricDefinitionRecord>(
        "SELECT metric_key, display_name, category, owner_team, business_definition,
             numerator_definition, denominator_definition, grain, refresh_cadence,
             target_direction, accepted_by
         FROM business_metric_definitions
         WHERE active = TRUE
         ORDER BY category, metric_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_reporting_read_models(
    pool: &DbPool,
) -> Result<Vec<ReportingReadModelRecord>, sqlx::Error> {
    sqlx::query_as::<_, ReportingReadModelRecord>(
        "SELECT model_key, display_name, source_tables, target_table, refresh_strategy,
             refresh_cadence, owner_team, operational_screen_safe, last_refresh_status,
             last_refreshed_at, warehouse_export_enabled
         FROM reporting_read_models
         ORDER BY model_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_customer_scorecards(
    pool: &DbPool,
    organization_id: i64,
) -> Result<Vec<CustomerScorecardRecord>, sqlx::Error> {
    sqlx::query_as::<_, CustomerScorecardRecord>(
        "SELECT id, organization_id, period_start, period_end, posted_loads, booked_loads,
             acceptance_rate::double precision AS acceptance_rate,
             on_time_pickup_rate::double precision AS on_time_pickup_rate,
             on_time_delivery_rate::double precision AS on_time_delivery_rate,
             document_cycle_minutes, gross_margin_cents,
             dispute_rate::double precision AS dispute_rate,
             score::double precision AS score, score_tone
         FROM customer_scorecards
         WHERE organization_id = $1
         ORDER BY period_end DESC, score DESC NULLS LAST",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn list_carrier_scorecards(
    pool: &DbPool,
    organization_id: i64,
) -> Result<Vec<CarrierScorecardRecord>, sqlx::Error> {
    sqlx::query_as::<_, CarrierScorecardRecord>(
        "SELECT id, organization_id, carrier_user_id, period_start, period_end, offered_loads,
             accepted_loads, acceptance_rate::double precision AS acceptance_rate,
             tracking_compliance_rate::double precision AS tracking_compliance_rate,
             on_time_pickup_rate::double precision AS on_time_pickup_rate,
             on_time_delivery_rate::double precision AS on_time_delivery_rate,
             claims_rate::double precision AS claims_rate,
             document_quality_rate::double precision AS document_quality_rate,
             payout_cycle_hours, score::double precision AS score, score_tone
         FROM carrier_scorecards
         WHERE organization_id = $1
         ORDER BY period_end DESC, score DESC NULLS LAST",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn list_lane_pricing_recommendations(
    pool: &DbPool,
    organization_id: i64,
) -> Result<Vec<LanePricingRecommendationRecord>, sqlx::Error> {
    sqlx::query_as::<_, LanePricingRecommendationRecord>(
        "SELECT id, organization_id, lane_key, equipment_type, recommended_rate_cents,
             low_rate_cents, high_rate_cents, currency,
             confidence_score::double precision AS confidence_score, sample_size,
             anomaly_status, recommendation_reason
         FROM lane_pricing_recommendations
         WHERE organization_id = $1
           AND (valid_until IS NULL OR valid_until >= CURRENT_DATE)
         ORDER BY anomaly_status DESC, confidence_score DESC, lane_key",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn search_global_documents(
    pool: &DbPool,
    organization_id: i64,
    query: &str,
    permission_keys: &[String],
) -> Result<Vec<GlobalSearchDocumentRecord>, sqlx::Error> {
    let like_query = format!("%{}%", query.trim());
    sqlx::query_as::<_, GlobalSearchDocumentRecord>(
        "SELECT id, organization_id, entity_type, entity_id, title, subtitle, href, permission_key
         FROM global_search_documents
         WHERE organization_id = $1
           AND (
                permission_key IS NULL
                OR cardinality($3::TEXT[]) = 0
                OR permission_key = ANY($3::TEXT[])
           )
           AND (
                searchable_text ILIKE $2
                OR title ILIKE $2
                OR entity_id ILIKE $2
           )
         ORDER BY last_indexed_at DESC
         LIMIT 50",
    )
    .bind(organization_id)
    .bind(like_query)
    .bind(permission_keys)
    .fetch_all(pool)
    .await
}

pub async fn list_data_quality_rules(
    pool: &DbPool,
) -> Result<Vec<DataQualityRuleRecord>, sqlx::Error> {
    sqlx::query_as::<_, DataQualityRuleRecord>(
        "SELECT rule_key, category, severity, owner_team, cadence, alert_threshold
         FROM data_quality_rules
         WHERE active = TRUE
         ORDER BY severity DESC, category, rule_key",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_open_data_quality_findings(
    pool: &DbPool,
    organization_id: Option<i64>,
) -> Result<Vec<DataQualityFindingRecord>, sqlx::Error> {
    sqlx::query_as::<_, DataQualityFindingRecord>(
        "SELECT finding.id, finding.organization_id, rule.rule_key, finding.entity_type,
             finding.entity_id, finding.severity, finding.finding_status, finding.owner_team,
             finding.title, finding.detail
         FROM data_quality_findings finding
         INNER JOIN data_quality_rules rule ON rule.id = finding.rule_id
         WHERE finding.finding_status IN ('open', 'assigned', 'in_repair')
           AND ($1::BIGINT IS NULL OR finding.organization_id = $1)
         ORDER BY finding.detected_at DESC",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}
