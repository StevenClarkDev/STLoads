use domain::atmp_contract::{AtmpContractAction, AtmpContractEnvelope};
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

fn publish_payload(idempotency_key: &str) -> serde_json::Value {
    json!({
        "contract_version": "2026-05-01",
        "action": "publish",
        "tenant_id": "tenant-p4",
        "event_id": format!("evt-{}", Uuid::new_v4()),
        "correlation_id": format!("corr-{}", Uuid::new_v4()),
        "idempotency_key": idempotency_key,
        "atmp_load_id": "ATMP-P4-1001",
        "release_gate": "stloads_ready",
        "payload": {
            "party_type": "broker",
            "freight_mode": "road",
            "equipment_type": "dry_van",
            "commodity_description": "Production freight",
            "weight": 42000.0,
            "weight_unit": "lbs",
            "pickup_city": "Houston",
            "pickup_state": "TX",
            "pickup_country": "US",
            "pickup_address": "100 Port Way",
            "pickup_window_start": "2026-05-21T09:00:00Z",
            "dropoff_city": "Dallas",
            "dropoff_state": "TX",
            "dropoff_country": "US",
            "dropoff_address": "200 Market Road",
            "dropoff_window_start": "2026-05-22T17:00:00Z",
            "board_rate": 2450.0,
            "rate_currency": "USD",
            "bid_type": "open",
            "readiness": "ready"
        }
    })
}

#[test]
fn atmp_contract_rejects_unsupported_versions_with_explicit_error() {
    let mut payload = publish_payload("idem-version");
    payload["contract_version"] = json!("2024-01-01");

    let error = AtmpContractEnvelope::try_from_value(payload).unwrap_err();

    assert!(error.code().contains("unsupported_contract_version"));
    assert!(error.to_string().contains("2024-01-01"));
}

#[test]
fn atmp_contract_rejects_missing_identity_fields() {
    let mut payload = publish_payload("idem-missing");
    payload["tenant_id"] = json!("");
    payload["atmp_load_id"] = json!("");
    payload["idempotency_key"] = json!("");

    let error = AtmpContractEnvelope::try_from_value(payload).unwrap_err();
    let message = error.to_string();

    assert!(message.contains("tenant_id"));
    assert!(message.contains("atmp_load_id"));
    assert!(message.contains("idempotency_key"));
}

#[test]
fn atmp_contract_normalizes_terminal_actions() {
    for (action, expected) in [
        ("withdraw", AtmpContractAction::Withdraw),
        ("cancel", AtmpContractAction::Cancel),
        ("close", AtmpContractAction::Close),
        ("status", AtmpContractAction::Status),
        ("document", AtmpContractAction::Document),
        ("finance", AtmpContractAction::Finance),
    ] {
        let mut payload = publish_payload(&format!("idem-{action}"));
        payload["action"] = json!(action);

        let envelope = AtmpContractEnvelope::try_from_value(payload).unwrap();

        assert_eq!(envelope.action, expected);
    }
}

#[tokio::test]
#[serial(atmp_contract_db)]
async fn atmp_publish_replay_does_not_duplicate_postings() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let payload = publish_payload("idem-replay");
    let first = db::tms::apply_atmp_contract_event(&pool, payload.clone())
        .await
        .unwrap();
    let second = db::tms::apply_atmp_contract_event(&pool, payload)
        .await
        .unwrap();

    let posting_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stloads_postings WHERE tenant_id = 'tenant-p4' AND source_load_id = 'ATMP-P4-1001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(first.posting_id, second.posting_id);
    assert!(second.replayed);
    assert_eq!(posting_count, 1);
}

#[tokio::test]
#[serial(atmp_contract_db)]
async fn atmp_update_adds_posting_version_and_cancel_hides_listing() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let publish = publish_payload("idem-update-seed");
    let published = db::tms::apply_atmp_contract_event(&pool, publish)
        .await
        .unwrap();

    let mut update = publish_payload("idem-update-change");
    update["action"] = json!("update");
    update["event_id"] = json!(format!("evt-{}", Uuid::new_v4()));
    update["payload"]["board_rate"] = json!(2550.0);
    let updated = db::tms::apply_atmp_contract_event(&pool, update)
        .await
        .unwrap();

    let mut cancel = publish_payload("idem-update-cancel");
    cancel["action"] = json!("cancel");
    cancel["event_id"] = json!(format!("evt-{}", Uuid::new_v4()));
    let canceled = db::tms::apply_atmp_contract_event(&pool, cancel)
        .await
        .unwrap();

    let posting_id = published.posting_id.unwrap();
    let (version_count, status, visibility): (i64, String, String) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM stloads_posting_versions WHERE posting_id = $1),
            status,
            visibility
         FROM stloads_postings
         WHERE id = $1",
    )
    .bind(posting_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(updated.posting_id, Some(posting_id));
    assert_eq!(canceled.posting_id, Some(posting_id));
    assert!(version_count >= 3);
    assert_eq!(status, "canceled");
    assert_eq!(visibility, "private");
}
