#![recursion_limit = "256"]

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use backend::{
    app,
    test_support::{auth_headers_for_user, insert_user_with_role_status, prepare_pool, test_state},
};
use domain::auth::{AccountStatus, UserRole};
use serde_json::{Value, json};
use serial_test::serial;
use tower::ServiceExt;

fn compliant_payload_with_missing_document_status() -> Value {
    json!({
        "tms_load_id": "ATMP-COMPLIANCE-BACKEND-1001",
        "tenant_id": "tenant-compliance",
        "external_handoff_id": "dispatch-handoff-backend-1001",
        "party_type": "shipper",
        "freight_mode": "road",
        "equipment_type": "Dry Van",
        "commodity_description": "Compliance controlled freight",
        "weight": 42000.0,
        "weight_unit": "lbs",
        "piece_count": 20,
        "is_hazardous": false,
        "pickup_city": "Dallas",
        "pickup_state": "TX",
        "pickup_zip": "75201",
        "pickup_country": "US",
        "pickup_address": "100 Market St, Dallas, TX",
        "pickup_window_start": "2026-06-01T09:00:00Z",
        "pickup_window_end": "2026-06-01T12:00:00Z",
        "dropoff_city": "Memphis",
        "dropoff_state": "TN",
        "dropoff_zip": "38103",
        "dropoff_country": "US",
        "dropoff_address": "200 Carrier Ave, Memphis, TN",
        "dropoff_window_start": "2026-06-02T15:00:00Z",
        "dropoff_window_end": "2026-06-02T18:00:00Z",
        "board_rate": 2450.0,
        "rate_currency": "USD",
        "bid_type": "Fixed",
        "compliance_passed": true,
        "required_documents_status": {
            "bol": "generated",
            "freight_bill": "pending",
            "rate_confirmation": "generated",
            "dispatch_sheet": "generated"
        },
        "readiness": "publishable",
        "pushed_by": "dispatch-api",
        "push_reason": "compliance contract test",
        "source_module": "atmp-dispatch",
        "payload_version": "dispatch-stloads-v2",
        "paperwork_packet_id": "packet-1001",
        "document_packet_url": "https://dispatch.example.test/packet-1001.pdf",
        "document_packet_hash": "sha256:packet1001",
        "bol_number": "BOL-1001",
        "freight_bill_number": "FB-1001",
        "atmp_operating_role": "broker",
        "carrier_authority_snapshot": {
            "usdot_number": "123456",
            "authority_status": "active"
        },
        "insurance_snapshot": {
            "cargo_insurance_status": "active"
        },
        "compliance_blockers": [],
        "retention_metadata": {
            "retention_category": "freight_load_file",
            "retention_period_years": 3
        },
        "audit_event_ids": ["audit-1001"],
        "external_refs": [
            {
                "ref_type": "dispatch_load_id",
                "ref_value": "load-1001",
                "ref_source": "atmp-dispatch"
            },
            {
                "ref_type": "paperwork_packet_id",
                "ref_value": "packet-1001",
                "ref_source": "atmp-dispatch"
            }
        ]
    })
}

#[tokio::test]
#[serial]
async fn tms_push_quarantines_or_rejects_non_compliant_handoff()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let state = test_state(pool.clone());
    let admin = insert_user_with_role_status(
        &pool,
        "Compliance Admin",
        "compliance-admin@example.test",
        UserRole::Admin,
        AccountStatus::Approved,
    )
    .await?;
    let headers = auth_headers_for_user(&state, &admin).await?;
    let body = serde_json::to_vec(&compliant_payload_with_missing_document_status())?;

    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/tms/push")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))?;
    *request.headers_mut() = headers;
    request
        .headers_mut()
        .insert(header::CONTENT_TYPE, "application/json".parse()?);

    let response = app::router(state).oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await?;
    let envelope: Value = serde_json::from_slice(&body)?;
    let data = envelope
        .get("data")
        .expect("TMS push response should use the API envelope");

    assert_eq!(
        data.get("success").and_then(Value::as_bool),
        Some(false),
        "STLOADS must reject or quarantine non-compliant handoffs instead of publishing them"
    );
    assert!(
        data.get("status_label")
            .and_then(Value::as_str)
            .is_some_and(|label| matches!(label, "Quarantined" | "Blocked" | "Validation Error")),
        "non-compliant handoff should return a compliance-specific non-active status"
    );

    Ok(())
}
