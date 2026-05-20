use domain::atmp_contract::{AtmpContractAction, AtmpContractEnvelope};
use serde_json::json;
use serial_test::serial;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};
use uuid::Uuid;

fn spawn_atmp_reconcile_endpoint(max_requests: usize) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let thread_count = count.clone();

    thread::spawn(move || {
        for stream in listener.incoming().take(max_requests) {
            let mut stream = stream.unwrap();
            let mut buffer = [0_u8; 8192];
            let _ = stream.read(&mut buffer);
            thread_count.fetch_add(1, Ordering::SeqCst);
            let response = "HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nOK";
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    (url, count)
}

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

#[tokio::test]
#[serial(atmp_contract_db)]
async fn atmp_publish_update_withdraw_close_reconciles_back_to_atmp() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };
    let (atmp_url, delivered_count) = spawn_atmp_reconcile_endpoint(4);
    let load_id = format!("ATMP-E1-{}", Uuid::new_v4().simple());

    let mut publish = publish_payload("idem-e1-publish");
    publish["atmp_load_id"] = json!(load_id);
    let published = db::tms::apply_atmp_contract_event(&pool, publish.clone())
        .await
        .unwrap();

    let replayed = db::tms::apply_atmp_contract_event(&pool, publish)
        .await
        .unwrap();
    assert!(replayed.replayed);
    assert_eq!(replayed.posting_id, published.posting_id);

    let mut update = publish_payload("idem-e1-update");
    update["action"] = json!("update");
    update["event_id"] = json!(format!("evt-{}", Uuid::new_v4()));
    update["correlation_id"] = json!(format!("corr-{}", Uuid::new_v4()));
    update["atmp_load_id"] = json!(load_id);
    update["payload"]["board_rate"] = json!(2650.0);
    let updated = db::tms::apply_atmp_contract_event(&pool, update)
        .await
        .unwrap();

    let mut withdraw = publish_payload("idem-e1-withdraw");
    withdraw["action"] = json!("withdraw");
    withdraw["event_id"] = json!(format!("evt-{}", Uuid::new_v4()));
    withdraw["correlation_id"] = json!(format!("corr-{}", Uuid::new_v4()));
    withdraw["atmp_load_id"] = json!(load_id);
    withdraw["payload"] = json!({});
    let withdrawn = db::tms::apply_atmp_contract_event(&pool, withdraw)
        .await
        .unwrap();

    let mut close = publish_payload("idem-e1-close");
    close["action"] = json!("close");
    close["event_id"] = json!(format!("evt-{}", Uuid::new_v4()));
    close["correlation_id"] = json!(format!("corr-{}", Uuid::new_v4()));
    close["atmp_load_id"] = json!(load_id);
    close["payload"] = json!({});
    let closed = db::tms::apply_atmp_contract_event(&pool, close)
        .await
        .unwrap();

    let posting_id = published.posting_id.unwrap();
    assert_eq!(updated.posting_id, Some(posting_id));
    assert_eq!(withdrawn.posting_id, Some(posting_id));
    assert_eq!(closed.posting_id, Some(posting_id));

    let (status, visibility, inbound_count, inbound_payloads, outbound_payloads, version_count): (
        String,
        String,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT
            p.status,
            p.visibility,
            (SELECT COUNT(*) FROM atmp_inbound_events WHERE tenant_id = p.tenant_id AND atmp_load_id = p.source_load_id),
            (SELECT COUNT(*) FROM atmp_contract_payloads WHERE tenant_id = p.tenant_id AND direction = 'inbound'),
            (SELECT COUNT(*) FROM atmp_contract_payloads WHERE tenant_id = p.tenant_id AND direction = 'outbound'),
            (SELECT COUNT(*) FROM stloads_posting_versions WHERE tenant_id = p.tenant_id AND posting_id = p.id)
         FROM stloads_postings p
         WHERE p.id = $1",
    )
    .bind(posting_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(status, "closed");
    assert_eq!(visibility, "private");
    assert_eq!(inbound_count, 4);
    assert_eq!(inbound_payloads, 4);
    assert_eq!(outbound_payloads, 4);
    assert!(version_count >= 4);

    let outbound_types: Vec<String> = sqlx::query_scalar(
        "SELECT event_type
         FROM atmp_outbound_events
         WHERE tenant_id = 'tenant-p4' AND posting_id = $1
         ORDER BY id",
    )
    .bind(posting_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(
        outbound_types,
        vec![
            "listing_published",
            "listing_updated",
            "listing_withdrawn",
            "listing_closed"
        ]
    );

    let delivered = backend::atmp_outbound::process_due_events(
        &pool,
        backend::atmp_outbound::AtmpOutboundOptions {
            default_base_url: Some(atmp_url),
            batch_size: 10,
            max_attempts: 3,
        },
    )
    .await
    .unwrap();

    assert_eq!(delivered.delivered, 4);
    assert_eq!(delivered_count.load(Ordering::SeqCst), 4);
}
