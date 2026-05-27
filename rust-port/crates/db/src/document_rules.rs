use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequiredDocumentRuleRecord {
    pub id: i64,
    pub rule_key: String,
    pub label: String,
    pub requirement_scope: String,
    pub role_key: Option<String>,
    pub organization_id: Option<i64>,
    pub lifecycle_state: String,
    pub document_type_key: String,
    pub blocks_transition: bool,
}

pub async fn list_required_document_rules(
    pool: &DbPool,
    requirement_scope: &str,
    lifecycle_state: &str,
    role_key: Option<&str>,
    organization_id: Option<i64>,
) -> Result<Vec<RequiredDocumentRuleRecord>, sqlx::Error> {
    sqlx::query_as::<_, RequiredDocumentRuleRecord>(
        "SELECT id, rule_key, label, requirement_scope, role_key, organization_id,
                lifecycle_state, document_type_key, blocks_transition
         FROM required_document_rules
         WHERE is_active = TRUE
           AND requirement_scope = $1
           AND lifecycle_state = $2
           AND (role_key IS NULL OR role_key = $3)
           AND (organization_id IS NULL OR organization_id = $4)
         ORDER BY organization_id NULLS FIRST, role_key NULLS FIRST, label",
    )
    .bind(requirement_scope)
    .bind(lifecycle_state)
    .bind(role_key)
    .bind(organization_id)
    .fetch_all(pool)
    .await
}
