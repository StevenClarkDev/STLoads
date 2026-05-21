#![recursion_limit = "256"]

use serde_json::Value;
use shared::TmsHandoffPayload;

fn sample_payload() -> TmsHandoffPayload {
    serde_json::from_value(serde_json::json!({
        "tms_load_id": "ATMP-COMPLIANCE-1001",
        "tenant_id": "tenant-compliance",
        "external_handoff_id": "dispatch-handoff-1001",
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
            "freight_bill": "generated"
        },
        "readiness": "publishable",
        "pushed_by": "dispatch-api",
        "push_reason": "compliance contract test",
        "source_module": "atmp-dispatch",
        "payload_version": "dispatch-stloads-v2",
        "external_refs": [
            {
                "ref_type": "dispatch_load_id",
                "ref_value": "load-1001",
                "ref_source": "atmp-dispatch"
            }
        ],
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
        "audit_event_ids": ["audit-1001"]
    }))
    .expect("sample payload should deserialize")
}

#[test]
fn tms_handoff_payload_preserves_compliance_contract_markers() {
    let value = serde_json::to_value(sample_payload()).expect("payload should serialize");
    let object = value
        .as_object()
        .expect("payload should serialize as object");

    for required_key in [
        "paperwork_packet_id",
        "document_packet_url",
        "document_packet_hash",
        "bol_number",
        "freight_bill_number",
        "atmp_operating_role",
        "carrier_authority_snapshot",
        "insurance_snapshot",
        "compliance_blockers",
        "retention_metadata",
        "audit_event_ids",
    ] {
        assert!(
            object.contains_key(required_key),
            "TmsHandoffPayload must preserve compliance field `{required_key}`"
        );
    }
}

#[test]
fn tms_handoff_payload_preserves_external_refs_for_dispatch_packet_identity() {
    let value = serde_json::to_value(sample_payload()).expect("payload should serialize");
    let refs = value
        .get("external_refs")
        .and_then(Value::as_array)
        .expect("external_refs should be serialized as array");

    for required_ref in [
        "dispatch_load_id",
        "dispatch_handoff_id",
        "paperwork_packet_id",
        "bol_number",
        "freight_bill_number",
    ] {
        assert!(
            refs.iter().any(|reference| {
                reference
                    .get("ref_type")
                    .and_then(Value::as_str)
                    .is_some_and(|value| value == required_ref)
            }),
            "external_refs must include `{required_ref}` for STLOADS reconciliation"
        );
    }
}
