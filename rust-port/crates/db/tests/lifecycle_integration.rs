use db::{
    DbPool,
    auth::{
        CreateKycDocumentInput, UpdateKycDocumentInput, create_kyc_document,
        list_kyc_documents_by_user_id, update_kyc_document, verify_kyc_document_blockchain,
    },
    compliance::{
        CreateRiskReviewItemParams, ENTERPRISE_COMPLIANCE_DOMAINS,
        UpsertCarrierAuthorityVerificationParams, UpsertComplianceStatusParams,
        UpsertDriverEquipmentSafetyParams, UpsertSanctionsTaxProfileParams,
        carrier_authority_booking_blocker, compliance_eligibility_summary_for_user,
        create_risk_review_item, driver_equipment_booking_blocker,
        find_carrier_authority_verification, list_compliance_statuses_for_user,
        risk_review_booking_blocker, risk_review_payout_blocker, sanctions_tax_booking_blocker,
        sanctions_tax_payout_blocker, upsert_carrier_authority_verification,
        upsert_compliance_status, upsert_driver_equipment_safety_compliance,
        upsert_sanctions_tax_profile,
    },
    connect,
    dispatch::{
        CreateCustomerContractLaneParams, CreateCustomerContractParams, CreateLoadLegParams,
        CreateLoadParams, UpsertLoadDocumentParams, book_load_leg, clone_load_as_draft,
        create_customer_contract, create_customer_contract_lane, create_load_document,
        create_load_with_legs, find_load_by_id, list_active_freight_document_templates,
        list_load_board_saved_filters, load_board_search, load_freight_document_context,
        record_generated_freight_document, update_load_lifecycle, update_load_with_legs,
    },
    document_rules::list_required_document_rules,
    email_outbox::{
        EnqueueEmailParams, claim_due_emails, count_pending_emails, enqueue_email,
        mark_email_delivered, mark_email_retry,
    },
    integrations::{
        EnqueueWebhookDeliveryParams, claim_external_event, complete_external_event,
        enqueue_webhook_delivery, list_webhook_delivery_logs, mark_webhook_delivery_for_replay,
    },
    legal_agreements::{
        AcceptLegalAgreementInput, accept_latest_legal_agreement, acceptance_has_audit_event,
        list_missing_required_agreements,
    },
    marketplace::list_recent_conversation_workspace_records_for_user,
    master_data, migrate,
    operating_authority::{
        list_customer_disclosable_evidence, list_evidence_due_for_review,
        list_operating_authority_decisions, upsert_compliance_evidence_document,
    },
    organizations::list_permission_keys_for_organization_role,
    organizations::{DEFAULT_ORGANIZATION_ID, default_organization, primary_membership_for_user},
    payments::find_escrow_for_leg_in_organization,
    payments::{
        CreatePaymentLedgerEntryParams, EscrowTransitionParams, FinanceApprovalDecisionParams,
        FinanceApprovalRequestParams, apply_escrow_transition, apply_invoice_settlement_adjustment,
        approve_finance_request, claim_stripe_webhook_event, complete_stripe_webhook_event,
        ensure_finance_approval_request, finance_request_has_required_approval,
        find_escrow_for_leg, find_invoice_settlement_for_leg, list_payment_ledger_entries_for_leg,
        record_payment_ledger_entry, release_has_required_finance_approval,
    },
    reliability::{
        claim_background_jobs, list_alert_rules, list_backup_restore_policies,
        list_dead_letter_jobs, list_incident_runbooks, list_observability_signals,
        list_query_performance_controls, mark_background_job_dead_letter,
    },
    reporting::{
        list_business_metric_definitions, list_carrier_scorecards, list_customer_scorecards,
        list_data_quality_rules, list_lane_pricing_recommendations,
        list_open_data_quality_findings, list_reporting_read_models, search_global_documents,
    },
    tms::{
        CreateTmsConflictParams, apply_status_webhook, create_tms_conflict, find_handoff_by_id,
        handoff_belongs_to_organization, list_open_tms_conflicts, list_tms_source_of_truth_rules,
        process_retryable_handoffs, push_handoff, repair_tms_conflict, run_reconciliation_scan,
    },
};
use domain::payments::EscrowStatus;
use serial_test::serial;
use shared::{TmsHandoffPayload, TmsStatusWebhookRequest};
use sqlx::Row;

struct LegFixture {
    shipper_user_id: i64,
    carrier_user_id: i64,
    load_id: i64,
    leg_id: i64,
}

fn test_database_url() -> Option<String> {
    std::env::var("RUST_TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("TEST_DATABASE_URL").ok())
}

async fn prepare_pool() -> Result<Option<DbPool>, Box<dyn std::error::Error>> {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping DB integration test because RUST_TEST_DATABASE_URL is not set");
        return Ok(None);
    };

    let pool = connect(&database_url).await?;
    migrate(&pool).await?;
    reset_database(&pool).await?;
    seed_load_statuses(&pool).await?;
    Ok(Some(pool))
}

async fn reset_database(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "TRUNCATE TABLE
            stloads_reconciliation_log,
            tms_conflict_queue,
            stloads_sync_errors,
            stloads_external_refs,
            stloads_handoff_events,
            stloads_handoffs,
            legal_agreement_acceptances,
            external_event_dedupe_records,
            external_idempotency_records,
            webhook_delivery_logs,
            outbound_webhook_endpoints,
            partner_api_auth_events,
            partner_api_clients,
            compliance_evidence_records,
            compliance_status_records,
            risk_review_items,
            sanctions_tax_profiles,
            driver_equipment_safety_compliance,
            carrier_authority_verifications,
            operating_authority_decisions,
            email_outbox,
            payment_stripe_webhook_events,
            finance_approval_requests,
            payment_ledger_entries,
            stloads_platform_invoices,
            shipper_credit_override_requests,
            payout_destination_change_reviews,
            ar_collection_notes,
            shipper_credit_accounts,
            stloads_usage_events,
            stloads_billing_accounts,
            stloads_subscription_plans,
            carrier_settlement_lines,
            customer_invoice_lines,
            carrier_settlements,
            customer_invoices,
            leg_document_versions,
            kyc_document_versions,
            load_document_versions,
            escrows,
            load_rate_override_audit,
            load_rating_quotes,
            accessorial_catalog,
            fuel_surcharge_rules,
            mileage_calculation_rules,
            facility_appointment_events,
            facility_appointments,
            facility_notes,
            facilities,
            load_mode_validation_events,
            load_localization_snapshots,
            load_cross_border_finance_checks,
            cross_border_finance_policies,
            governed_configuration_changes,
            customer_configuration_rules,
            carrier_capacity_profiles,
            carrier_network_memberships,
            hazmat_class_catalog,
            trailer_type_catalog,
            exception_reason_catalog,
            rejection_reason_catalog,
            service_level_catalog,
            booking_idempotency_keys,
            load_import_rows,
            load_import_batches,
            load_history,
            load_legs,
            loads,
            customer_lane_carriers,
            customer_contract_lanes,
            customer_contracts,
            locations,
            cities,
            countries,
            load_types,
            equipments,
            commodity_types,
            load_status_master,
            users
         RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn reporting_metrics_and_scorecards_are_seeded_and_queryable()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;
    let required_metrics = [
        "posted_loads",
        "booked_loads",
        "acceptance_rate",
        "quote_to_book_time",
        "tracking_compliance",
        "on_time_pickup",
        "on_time_delivery",
        "document_cycle_time",
        "margin",
        "payout_time",
        "dispute_rate",
    ];

    let metric_records = list_business_metric_definitions(&pool).await?;
    for metric_key in required_metrics {
        assert!(
            metric_records
                .iter()
                .any(|metric| metric.metric_key == metric_key),
            "missing seeded metric definition: {metric_key}"
        );
    }

    let read_models = list_reporting_read_models(&pool).await?;
    for model_key in [
        "load_operational_metrics_daily",
        "finance_metrics_daily",
        "customer_scorecards_monthly",
        "carrier_scorecards_monthly",
    ] {
        let model = read_models
            .iter()
            .find(|record| record.model_key == model_key)
            .unwrap_or_else(|| panic!("missing reporting read model: {model_key}"));
        assert!(model.operational_screen_safe);
        assert!(!model.source_tables.is_empty());
    }

    sqlx::query(
        "INSERT INTO customer_scorecards (
             organization_id, period_start, period_end, posted_loads, booked_loads,
             acceptance_rate, quote_to_book_minutes, on_time_pickup_rate,
             on_time_delivery_rate, document_cycle_minutes, gross_margin_cents,
             dispute_rate, score, score_tone, notes
         )
         VALUES ($1, CURRENT_DATE - INTERVAL '30 days', CURRENT_DATE, 42, 37,
                 0.8810, 96, 0.9400, 0.9100, 180, 1250000, 0.0300, 91.5,
                 'success', 'Monthly customer scorecard fixture.')
         ON CONFLICT (organization_id, COALESCE(customer_organization_id, 0), period_start, period_end)
         DO UPDATE SET
             posted_loads = EXCLUDED.posted_loads,
             booked_loads = EXCLUDED.booked_loads,
             acceptance_rate = EXCLUDED.acceptance_rate,
             score = EXCLUDED.score,
             score_tone = EXCLUDED.score_tone",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO carrier_scorecards (
             organization_id, carrier_user_id, period_start, period_end, offered_loads,
             accepted_loads, acceptance_rate, tracking_compliance_rate,
             on_time_pickup_rate, on_time_delivery_rate, claims_rate,
             document_quality_rate, payout_cycle_hours, score, score_tone, notes
         )
         VALUES ($1, $2, CURRENT_DATE - INTERVAL '30 days', CURRENT_DATE, 30, 26,
                 0.8667, 0.9700, 0.9200, 0.9000, 0.0200, 0.9600, 28, 89.25,
                 'success', 'Monthly carrier scorecard fixture.')
         ON CONFLICT (
             organization_id, COALESCE(carrier_user_id, 0), COALESCE(carrier_organization_id, 0),
             period_start, period_end
         )
         DO UPDATE SET
             offered_loads = EXCLUDED.offered_loads,
             accepted_loads = EXCLUDED.accepted_loads,
             tracking_compliance_rate = EXCLUDED.tracking_compliance_rate,
             score = EXCLUDED.score,
             score_tone = EXCLUDED.score_tone",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(fixture.carrier_user_id)
    .execute(&pool)
    .await?;

    let customer_scorecards = list_customer_scorecards(&pool, DEFAULT_ORGANIZATION_ID).await?;
    assert!(
        customer_scorecards
            .iter()
            .any(|scorecard| scorecard.posted_loads == 42
                && scorecard.score_tone == "success"
                && scorecard.score.unwrap_or_default() > 90.0)
    );

    let carrier_scorecards = list_carrier_scorecards(&pool, DEFAULT_ORGANIZATION_ID).await?;
    assert!(
        carrier_scorecards
            .iter()
            .any(
                |scorecard| scorecard.carrier_user_id == Some(fixture.carrier_user_id)
                    && scorecard.tracking_compliance_rate.unwrap_or_default() > 0.95
                    && scorecard.score_tone == "success"
            )
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn pricing_search_and_data_quality_controls_are_queryable()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;

    sqlx::query(
        "INSERT INTO lane_pricing_history (
             organization_id, lane_key, origin_label, destination_label, equipment_type,
             freight_mode, observed_rate_cents, observed_currency, source_type,
             source_entity_id, pickup_date, margin_cents, on_time_delivery
         )
         VALUES ($1, 'dallas-tx:chicago-il:dry-van', 'Dallas, TX', 'Chicago, IL',
                 'dry_van', 'truckload', 245000, 'USD', 'booked_load', $2,
                 CURRENT_DATE, 42000, TRUE)",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(fixture.load_id)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO lane_pricing_recommendations (
             organization_id, lane_key, equipment_type, recommended_rate_cents,
             low_rate_cents, high_rate_cents, currency, confidence_score,
             sample_size, anomaly_status, recommendation_reason
         )
         VALUES ($1, 'dallas-tx:chicago-il:dry-van', 'dry_van', 250000,
                 235000, 268000, 'USD', 87.5, 18, 'normal',
                 'Recent booked dry-van lane history supports this target range.')
         ON CONFLICT (organization_id, lane_key, COALESCE(equipment_type, ''), valid_from)
         DO UPDATE SET
             recommended_rate_cents = EXCLUDED.recommended_rate_cents,
             confidence_score = EXCLUDED.confidence_score,
             sample_size = EXCLUDED.sample_size,
             anomaly_status = EXCLUDED.anomaly_status,
             recommendation_reason = EXCLUDED.recommendation_reason",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .execute(&pool)
    .await?;

    let recommendations = list_lane_pricing_recommendations(&pool, DEFAULT_ORGANIZATION_ID).await?;
    assert!(
        recommendations
            .iter()
            .any(|row| row.lane_key == "dallas-tx:chicago-il:dry-van"
                && row.recommended_rate_cents == 250000
                && row.anomaly_status == "normal")
    );

    sqlx::query(
        "INSERT INTO global_search_documents (
             organization_id, entity_type, entity_id, title, subtitle, searchable_text,
             href, permission_key
         )
         VALUES
             ($1, 'load', $2, 'Dallas to Chicago dry van', 'Booked load',
              'Dallas Chicago dry van booked load customer invoice payment tms',
              $3, 'manage_loads'),
             ($1, 'payment', $4, 'Carrier payout for Dallas to Chicago',
              'Payment ledger', 'carrier payout invoice settlement Dallas Chicago',
              '/admin/payments', 'manage_payments')
         ON CONFLICT (organization_id, entity_type, entity_id)
         DO UPDATE SET
             title = EXCLUDED.title,
             searchable_text = EXCLUDED.searchable_text,
             href = EXCLUDED.href,
             permission_key = EXCLUDED.permission_key,
             last_indexed_at = CURRENT_TIMESTAMP",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(fixture.load_id.to_string())
    .bind(format!("/admin/loads/{}", fixture.load_id))
    .bind(format!("leg-{}", fixture.leg_id))
    .execute(&pool)
    .await?;

    let load_results = search_global_documents(
        &pool,
        DEFAULT_ORGANIZATION_ID,
        "Dallas",
        &["manage_loads".to_string()],
    )
    .await?;
    assert!(
        load_results
            .iter()
            .any(|row| row.entity_type == "load" && row.title.contains("Dallas"))
    );
    assert!(
        load_results
            .iter()
            .all(|row| row.permission_key.as_deref() != Some("manage_payments"))
    );

    let required_rules = [
        "orphan_load_legs",
        "invalid_state_combinations",
        "duplicate_external_references",
        "missing_required_documents",
        "stale_tms_handoffs",
        "unmatched_payments",
        "inconsistent_tenant_ownership",
        "lane_rate_anomaly",
        "carrier_score_change_anomaly",
        "suspicious_tracking_pattern",
        "unusual_document_replacement",
        "sudden_volume_change",
    ];
    let rules = list_data_quality_rules(&pool).await?;
    for rule_key in required_rules {
        assert!(
            rules.iter().any(|rule| rule.rule_key == rule_key),
            "missing data quality rule: {rule_key}"
        );
    }

    let run_id: i64 = sqlx::query_scalar(
        "INSERT INTO data_quality_runs (
             run_status, checked_rule_count, finding_count, triggered_by, finished_at, notes
         )
         VALUES ('success', $1, 1, 'test', CURRENT_TIMESTAMP, 'Phase 14 data quality test run.')
         RETURNING id",
    )
    .bind(required_rules.len() as i32)
    .fetch_one(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO data_quality_findings (
             organization_id, rule_id, run_id, entity_type, entity_id, severity,
             finding_status, owner_team, title, detail, repair_action
         )
         SELECT $1, id, $2, 'lane_pricing_recommendation', 'dallas-tx:chicago-il:dry-van',
                severity, 'open', owner_team, 'Lane pricing recommendation needs review',
                'Recommendation was flagged by anomaly monitoring before publication.',
                repair_playbook
         FROM data_quality_rules
         WHERE rule_key = 'lane_rate_anomaly'",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(run_id)
    .execute(&pool)
    .await?;

    let findings = list_open_data_quality_findings(&pool, Some(DEFAULT_ORGANIZATION_ID)).await?;
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule_key == "lane_rate_anomaly"
                && finding.finding_status == "open"
                && finding.owner_team == "Data/Product")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn reliability_operations_contracts_are_seeded_and_jobs_are_recoverable()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let signals = list_observability_signals(&pool).await?;
    for signal_key in [
        "http.request.latency",
        "http.request.error_rate",
        "db.pool.usage",
        "job.queue.lag",
        "worker.outcome",
        "email.delivery.failure",
        "webhook.delivery.failure",
        "storage.operation.error",
        "payment.operation.failure",
        "tms.drift",
    ] {
        assert!(
            signals.iter().any(|signal| signal.signal_key == signal_key),
            "missing observability signal {signal_key}"
        );
    }

    let alerts = list_alert_rules(&pool).await?;
    assert!(alerts.iter().any(|rule| rule.severity == "P0"));
    assert!(alerts.iter().any(|rule| rule.route_key == "payments"));

    let job_id: i64 = sqlx::query_scalar(
        "INSERT INTO background_jobs (
             organization_id, job_type, queue_name, priority, payload, max_attempts,
             visibility_timeout_seconds
         )
         VALUES ($1, 'reporting.refresh', 'reporting', 10,
                 jsonb_build_object('model_key', 'load_operational_metrics_daily'), 3, 60)
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;

    let claimed = claim_background_jobs(&pool, "reporting", "worker-phase15-test", 5).await?;
    assert!(claimed.iter().any(|job| job.id == job_id));
    let claimed_job = claimed.iter().find(|job| job.id == job_id).unwrap();
    assert_eq!(claimed_job.status, "running");
    assert_eq!(claimed_job.attempt_count, 1);

    let dead_letter =
        mark_background_job_dead_letter(&pool, job_id, "phase 15 test dead letter").await?;
    assert_eq!(dead_letter.status, "dead_letter");
    assert_eq!(
        dead_letter.dead_letter_reason.as_deref(),
        Some("phase 15 test dead letter")
    );

    let dead_letters = list_dead_letter_jobs(&pool, Some("reporting")).await?;
    assert!(dead_letters.iter().any(|job| job.id == job_id));

    let query_controls = list_query_performance_controls(&pool).await?;
    for query_key in [
        "load_board_search",
        "chat_threads",
        "tracking_points",
        "admin_queues",
        "tms_reconciliation",
        "global_search",
        "reporting_scorecards",
    ] {
        assert!(
            query_controls
                .iter()
                .any(|control| control.query_key == query_key
                    && !control.required_indexes.is_empty()),
            "missing query performance control {query_key}"
        );
    }

    let backups = list_backup_restore_policies(&pool).await?;
    assert!(
        backups
            .iter()
            .any(|policy| policy.policy_key == "postgres_primary")
    );
    assert!(
        backups
            .iter()
            .any(|policy| policy.policy_key == "object_storage_documents")
    );

    let runbooks = list_incident_runbooks(&pool).await?;
    for runbook_key in [
        "auth_outage",
        "database_outage",
        "object_storage_outage",
        "payment_incident",
        "duplicate_booking",
        "tms_outage",
        "email_outage",
        "data_exposure",
        "bad_deploy",
    ] {
        assert!(
            runbooks
                .iter()
                .any(|runbook| runbook.policy_key == runbook_key),
            "missing incident runbook {runbook_key}"
        );
    }

    let quotas: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM usage_quota_policies
         WHERE quota_key = ANY($1)",
    )
    .bind([
        "document_uploads_per_tenant",
        "api_calls_per_partner",
        "webhook_deliveries_per_tenant",
        "geocoding_requests_per_tenant",
        "tracking_pings_per_tenant",
        "sandbox_resets_per_tenant",
        "report_exports_per_tenant",
        "notifications_per_tenant",
    ])
    .fetch_one(&pool)
    .await?;
    assert_eq!(quotas, 8);

    let continuity: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM continuity_exercises
         WHERE status = 'planned'",
    )
    .fetch_one(&pool)
    .await?;
    assert!(continuity >= 4);

    Ok(())
}

#[tokio::test]
#[serial]
async fn stripe_webhook_event_claims_are_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    assert!(
        claim_stripe_webhook_event(
            &pool,
            "evt_duplicate_test",
            "payment_intent.succeeded",
            Some("pi_duplicate_test"),
            None,
        )
        .await?
    );
    assert!(
        !claim_stripe_webhook_event(
            &pool,
            "evt_duplicate_test",
            "payment_intent.succeeded",
            Some("pi_duplicate_test"),
            None,
        )
        .await?
    );

    complete_stripe_webhook_event(
        &pool,
        "evt_duplicate_test",
        "processed",
        &serde_json::json!({ "acknowledged": true }),
    )
    .await?;

    let status: String = sqlx::query_scalar(
        "SELECT processing_status FROM payment_stripe_webhook_events WHERE stripe_event_id = $1",
    )
    .bind("evt_duplicate_test")
    .fetch_one(&pool)
    .await?;
    assert_eq!(status, "processed");

    Ok(())
}

#[tokio::test]
#[serial]
async fn finance_release_approval_requires_two_distinct_approvals()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;
    let request = ensure_finance_approval_request(
        &pool,
        FinanceApprovalRequestParams {
            approval_type: "escrow_release",
            entity_type: "load_leg",
            entity_id: fixture.leg_id,
            organization_id: Some(1),
            amount_cents: 750000,
            currency: "USD",
            required_approval_count: 2,
            requested_by_user_id: Some(fixture.shipper_user_id),
            reason: Some("High-value release integration test"),
        },
    )
    .await?;

    assert_eq!(request.status, "pending");
    assert!(
        !release_has_required_finance_approval(&pool, fixture.leg_id, 2).await?,
        "pending approval must not satisfy high-value release approval"
    );

    let first = approve_finance_request(
        &pool,
        FinanceApprovalDecisionParams {
            approval_type: "escrow_release",
            entity_type: "load_leg",
            entity_id: fixture.leg_id,
            approver_user_id: fixture.carrier_user_id,
            decision_note: Some("First finance approval"),
        },
    )
    .await?
    .expect("approval request should exist");
    assert_eq!(first.status, "pending");
    assert_eq!(
        first.first_approved_by_user_id,
        Some(fixture.carrier_user_id)
    );

    let duplicate = approve_finance_request(
        &pool,
        FinanceApprovalDecisionParams {
            approval_type: "escrow_release",
            entity_type: "load_leg",
            entity_id: fixture.leg_id,
            approver_user_id: fixture.carrier_user_id,
            decision_note: Some("Duplicate approval attempt"),
        },
    )
    .await?
    .expect("approval request should still exist");
    assert_eq!(duplicate.status, "pending");
    assert!(duplicate.second_approved_by_user_id.is_none());

    let second = approve_finance_request(
        &pool,
        FinanceApprovalDecisionParams {
            approval_type: "escrow_release",
            entity_type: "load_leg",
            entity_id: fixture.leg_id,
            approver_user_id: fixture.shipper_user_id,
            decision_note: Some("Second finance approval"),
        },
    )
    .await?
    .expect("approval request should be approvable");
    assert_eq!(second.status, "approved");
    assert!(release_has_required_finance_approval(&pool, fixture.leg_id, 2).await?);

    let hold_request = ensure_finance_approval_request(
        &pool,
        FinanceApprovalRequestParams {
            approval_type: "escrow_hold",
            entity_type: "load_leg",
            entity_id: fixture.leg_id,
            organization_id: Some(1),
            amount_cents: 750000,
            currency: "USD",
            required_approval_count: 1,
            requested_by_user_id: Some(fixture.shipper_user_id),
            reason: Some("Manual hold approval integration test"),
        },
    )
    .await?;
    assert_eq!(hold_request.status, "pending");
    assert!(
        !finance_request_has_required_approval(&pool, "escrow_hold", "load_leg", fixture.leg_id, 1)
            .await?,
        "pending hold approval must not satisfy manual hold approval"
    );

    let hold_approved = approve_finance_request(
        &pool,
        FinanceApprovalDecisionParams {
            approval_type: "escrow_hold",
            entity_type: "load_leg",
            entity_id: fixture.leg_id,
            approver_user_id: fixture.carrier_user_id,
            decision_note: Some("Manual hold finance approval"),
        },
    )
    .await?
    .expect("hold approval request should exist");
    assert_eq!(hold_approved.status, "approved");
    assert!(
        finance_request_has_required_approval(&pool, "escrow_hold", "load_leg", fixture.leg_id, 1)
            .await?
    );

    Ok(())
}

async fn seed_load_statuses(pool: &DbPool) -> Result<(), sqlx::Error> {
    for (id, name, slug, sort_order) in [
        (1_i16, "New", "new", 1_i32),
        (4_i16, "Booked", "booked", 4_i32),
        (8_i16, "Escrow Funded", "escrow_funded", 8_i32),
        (11_i16, "Paid Out", "paid_out", 11_i32),
    ] {
        sqlx::query(
            "INSERT INTO load_status_master (id, name, slug, description, sort_order, is_terminal)
             VALUES ($1, $2, $3, $4, $5, FALSE)",
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(format!("{} seeded for integration tests", name))
        .bind(sort_order)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_user(pool: &DbPool, name: &str, email: &str) -> Result<i64, sqlx::Error> {
    let user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (name, email, password, status, created_at, updated_at)
         VALUES ($1, $2, $3, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(name)
    .bind(email)
    .bind("integration-test-password")
    .fetch_one(pool)
    .await?;

    Ok(user_id)
}

async fn insert_organization(pool: &DbPool, slug: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO organizations (name, slug, account_type, status, support_tier, created_at, updated_at)
         VALUES ($1, $2, 'customer', 'active', 'standard', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (slug) DO UPDATE SET
            name = EXCLUDED.name,
            status = 'active',
            updated_at = CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(format!("Tenant {}", slug))
    .bind(slug)
    .fetch_one(pool)
    .await
}

async fn insert_user_in_organization(
    pool: &DbPool,
    organization_id: i64,
    name: &str,
    email: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (organization_id, name, email, password, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(organization_id)
    .bind(name)
    .bind(email)
    .bind("integration-test-password")
    .fetch_one(pool)
    .await
}

async fn seed_valid_carrier_authority(
    pool: &DbPool,
    carrier_user_id: i64,
) -> Result<(), sqlx::Error> {
    upsert_carrier_authority_verification(
        pool,
        UpsertCarrierAuthorityVerificationParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            carrier_user_id,
            dot_number: Some("DOT-IT-1001"),
            mc_number: Some("MC-IT-1001"),
            legal_name: Some("Integration Carrier LLC"),
            authority_status: "active",
            operating_authority_type: Some("motor_carrier_property"),
            safety_rating: Some("satisfactory"),
            insurance_status: "verified",
            insurance_provider: Some("Integration Mutual"),
            insurance_policy_number: Some("POL-IT-1001"),
            cargo_coverage_amount_cents: Some(25_000_000),
            liability_coverage_amount_cents: Some(100_000_000),
            currency: "USD",
            insurance_effective_at: Some(chrono::Utc::now().date_naive()),
            insurance_expires_at: Some(
                chrono::Utc::now().date_naive() + chrono::Duration::days(90),
            ),
            verification_source: "manual_integration_test",
            reviewed_by_user_id: None,
            notes: Some("Seeded valid carrier authority for booking tests"),
        },
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn organization_foundation_assigns_default_membership()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let organization = default_organization(&pool)
        .await?
        .expect("default organization should exist");
    assert_eq!(organization.id, DEFAULT_ORGANIZATION_ID);
    assert_eq!(organization.slug, "stloads-default");

    let user_id = insert_user(&pool, "Tenant User", "tenant-user@example.com").await?;
    let user_organization_id: i64 =
        sqlx::query_scalar("SELECT organization_id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(user_organization_id, DEFAULT_ORGANIZATION_ID);

    let membership = primary_membership_for_user(&pool, user_id)
        .await?
        .expect("insert trigger should create a membership");
    assert_eq!(membership.organization_id, DEFAULT_ORGANIZATION_ID);
    assert_eq!(membership.role_key, "member");

    let owner_permissions = list_permission_keys_for_organization_role(&pool, "owner").await?;
    assert!(
        owner_permissions
            .iter()
            .any(|value| value == "manage_users")
    );
    assert!(
        owner_permissions
            .iter()
            .any(|value| value == "manage_tms_operations")
    );
    let integration_admin_permissions =
        list_permission_keys_for_organization_role(&pool, "integration_admin").await?;
    assert!(
        integration_admin_permissions
            .iter()
            .any(|value| value == "manage_tms_operations")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn split_compliance_models_drive_specific_eligibility()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let organization_id = insert_organization(&pool, "split-compliance").await?;
    let user_id = insert_user_in_organization(
        &pool,
        organization_id,
        "Compliance Carrier",
        "compliance-carrier@example.com",
    )
    .await?;

    for (subject_type, domain, status, blocking) in [
        ("person", "person_kyc", "approved", false),
        ("company", "company_kyb", "approved", false),
        ("carrier", "carrier_compliance", "approved", false),
        ("broker", "broker_compliance", "not_required", false),
        (
            "freight_forwarder",
            "freight_forwarder_compliance",
            "not_required",
            false,
        ),
        ("tax", "tax_compliance", "approved", false),
        ("payout", "payout_compliance", "blocked", true),
    ] {
        upsert_compliance_status(
            &pool,
            UpsertComplianceStatusParams {
                organization_id: Some(organization_id),
                user_id: Some(user_id),
                subject_type,
                compliance_domain: domain,
                status,
                eligibility_blocking: blocking,
                evidence_reference: Some("integration-test"),
                reviewer_user_id: None,
                reason: Some("Split compliance model integration test"),
                effective_at: None,
                expires_at: None,
            },
        )
        .await?;
    }

    let records = list_compliance_statuses_for_user(&pool, user_id).await?;
    assert_eq!(records.len(), ENTERPRISE_COMPLIANCE_DOMAINS.len());
    assert!(records.iter().any(|record| {
        record.compliance_domain == "payout_compliance" && record.eligibility_blocking
    }));

    let blocked_summary = compliance_eligibility_summary_for_user(&pool, user_id).await?;
    assert_eq!(blocked_summary.eligibility_status, "blocked");
    assert_eq!(blocked_summary.blocking_records, 1);
    assert!(blocked_summary.missing_domains.is_empty());

    upsert_compliance_status(
        &pool,
        UpsertComplianceStatusParams {
            organization_id: Some(organization_id),
            user_id: Some(user_id),
            subject_type: "payout",
            compliance_domain: "payout_compliance",
            status: "approved",
            eligibility_blocking: false,
            evidence_reference: Some("stripe-connect-approved"),
            reviewer_user_id: None,
            reason: Some("Payout compliance cleared"),
            effective_at: None,
            expires_at: None,
        },
    )
    .await?;

    let eligible_summary = compliance_eligibility_summary_for_user(&pool, user_id).await?;
    assert_eq!(eligible_summary.eligibility_status, "eligible");
    assert_eq!(
        eligible_summary.approved_records,
        ENTERPRISE_COMPLIANCE_DOMAINS.len()
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn carrier_authority_and_insurance_verification_blocks_booking()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_open_booking_race_fixture(&pool).await?;
    assert_eq!(
        carrier_authority_booking_blocker(&pool, fixture.carrier_user_id).await?,
        Some("Carrier FMCSA/DOT/MC and insurance verification is missing.".into())
    );
    let missing_result = book_load_leg(
        &pool,
        fixture.leg_id,
        fixture.carrier_user_id,
        None,
        Some(fixture.carrier_user_id),
        Some("missing-compliance-booking"),
    )
    .await;
    assert!(missing_result.is_err());

    upsert_carrier_authority_verification(
        &pool,
        UpsertCarrierAuthorityVerificationParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            carrier_user_id: fixture.carrier_user_id,
            dot_number: Some("DOT-EXPIRED"),
            mc_number: Some("MC-EXPIRED"),
            legal_name: Some("Expired Insurance Carrier LLC"),
            authority_status: "active",
            operating_authority_type: Some("motor_carrier_property"),
            safety_rating: Some("conditional"),
            insurance_status: "verified",
            insurance_provider: Some("Expired Mutual"),
            insurance_policy_number: Some("EXP-1001"),
            cargo_coverage_amount_cents: Some(10_000_000),
            liability_coverage_amount_cents: Some(75_000_000),
            currency: "USD",
            insurance_effective_at: Some(
                chrono::Utc::now().date_naive() - chrono::Duration::days(120),
            ),
            insurance_expires_at: Some(chrono::Utc::now().date_naive() - chrono::Duration::days(1)),
            verification_source: "manual_integration_test",
            reviewed_by_user_id: None,
            notes: Some("Expired insurance should block booking"),
        },
    )
    .await?;
    assert_eq!(
        carrier_authority_booking_blocker(&pool, fixture.carrier_user_id).await?,
        Some("Carrier insurance is expired or missing an expiration date.".into())
    );

    seed_valid_carrier_authority(&pool, fixture.carrier_user_id).await?;
    assert!(
        carrier_authority_booking_blocker(&pool, fixture.carrier_user_id)
            .await?
            .is_none()
    );

    let verification = find_carrier_authority_verification(&pool, fixture.carrier_user_id)
        .await?
        .expect("carrier verification should exist");
    assert_eq!(verification.authority_status, "active");
    assert_eq!(verification.insurance_status, "verified");

    let booked = book_load_leg(
        &pool,
        fixture.leg_id,
        fixture.carrier_user_id,
        None,
        Some(fixture.carrier_user_id),
        Some("valid-compliance-booking"),
    )
    .await?
    .expect("valid carrier should be able to book");
    assert_eq!(booked.booked_carrier_id, Some(fixture.carrier_user_id));

    Ok(())
}

#[tokio::test]
#[serial]
async fn driver_equipment_safety_blocks_restricted_booking()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_open_booking_race_fixture(&pool).await?;
    seed_valid_carrier_authority(&pool, fixture.carrier_user_id).await?;

    upsert_driver_equipment_safety_compliance(
        &pool,
        UpsertDriverEquipmentSafetyParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            carrier_user_id: fixture.carrier_user_id,
            driver_compliance_status: "approved",
            cdl_expires_at: Some(chrono::Utc::now().date_naive() + chrono::Duration::days(180)),
            medical_card_expires_at: Some(
                chrono::Utc::now().date_naive() + chrono::Duration::days(180),
            ),
            mvr_status: "approved",
            background_check_status: "approved",
            endorsements: vec!["van".into()],
            equipment_compliance_status: "approved",
            truck_unit_identifier: Some("TRUCK-1002A"),
            trailer_unit_identifier: Some("TRAILER-1002A"),
            vin: Some("VIN1002APHASE10"),
            ownership_status: Some("owned"),
            inspection_expires_at: Some(
                chrono::Utc::now().date_naive() + chrono::Duration::days(90),
            ),
            maintenance_status: "current",
            equipment_insurance_status: "verified",
            safety_rating: Some("satisfactory"),
            csa_alert_level: "none",
            hazmat_eligible: false,
            temperature_control_eligible: false,
            restricted_freight_blocking: true,
            dvir_policy: "deferred_to_carrier_system",
            notes: Some("Restricted freight hold should block booking"),
            reviewed_by_user_id: None,
        },
    )
    .await?;
    assert_eq!(
        driver_equipment_booking_blocker(&pool, fixture.carrier_user_id).await?,
        Some("Carrier driver/equipment safety review blocks restricted freight.".into())
    );
    assert!(
        book_load_leg(
            &pool,
            fixture.leg_id,
            fixture.carrier_user_id,
            None,
            Some(fixture.carrier_user_id),
            Some("driver-equipment-blocked"),
        )
        .await
        .is_err()
    );

    upsert_driver_equipment_safety_compliance(
        &pool,
        UpsertDriverEquipmentSafetyParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            carrier_user_id: fixture.carrier_user_id,
            driver_compliance_status: "approved",
            cdl_expires_at: Some(chrono::Utc::now().date_naive() + chrono::Duration::days(180)),
            medical_card_expires_at: Some(
                chrono::Utc::now().date_naive() + chrono::Duration::days(180),
            ),
            mvr_status: "approved",
            background_check_status: "approved",
            endorsements: vec!["van".into(), "reefer".into()],
            equipment_compliance_status: "approved",
            truck_unit_identifier: Some("TRUCK-1002A"),
            trailer_unit_identifier: Some("TRAILER-1002A"),
            vin: Some("VIN1002APHASE10"),
            ownership_status: Some("owned"),
            inspection_expires_at: Some(
                chrono::Utc::now().date_naive() + chrono::Duration::days(90),
            ),
            maintenance_status: "current",
            equipment_insurance_status: "verified",
            safety_rating: Some("satisfactory"),
            csa_alert_level: "none",
            hazmat_eligible: false,
            temperature_control_eligible: true,
            restricted_freight_blocking: false,
            dvir_policy: "deferred_to_carrier_system",
            notes: Some("Cleared driver/equipment safety review"),
            reviewed_by_user_id: None,
        },
    )
    .await?;
    assert!(
        driver_equipment_booking_blocker(&pool, fixture.carrier_user_id)
            .await?
            .is_none()
    );

    let booked = book_load_leg(
        &pool,
        fixture.leg_id,
        fixture.carrier_user_id,
        None,
        Some(fixture.carrier_user_id),
        Some("driver-equipment-cleared"),
    )
    .await?
    .expect("cleared driver/equipment compliance should allow booking");
    assert_eq!(booked.booked_carrier_id, Some(fixture.carrier_user_id));

    Ok(())
}

#[tokio::test]
#[serial]
async fn sanctions_tax_and_risk_reviews_block_booking_and_payout()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_open_booking_race_fixture(&pool).await?;
    seed_valid_carrier_authority(&pool, fixture.carrier_user_id).await?;

    upsert_sanctions_tax_profile(
        &pool,
        UpsertSanctionsTaxProfileParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            user_id: fixture.carrier_user_id,
            sanctions_status: "possible_match",
            sanctions_provider: "manual_ofac_review",
            sanctions_reference: Some("OFAC-POSSIBLE-1003"),
            beneficial_owner_status: "pending",
            tax_document_status: "pending",
            tax_document_type: Some("W-9"),
            tin_masked: Some("***-**-6789"),
            tax_reporting_owner: "internal_finance",
            tax_year: Some(2026),
            payout_tax_blocking: true,
            notes: Some("Possible sanctions match should block booking and payout"),
            reviewed_by_user_id: None,
        },
    )
    .await?;
    assert_eq!(
        sanctions_tax_booking_blocker(&pool, fixture.carrier_user_id).await?,
        Some("Carrier has an unresolved sanctions screening result.".into())
    );
    assert_eq!(
        sanctions_tax_payout_blocker(&pool, fixture.carrier_user_id).await?,
        Some("Payout blocked by sanctions screening.".into())
    );
    assert!(
        book_load_leg(
            &pool,
            fixture.leg_id,
            fixture.carrier_user_id,
            None,
            Some(fixture.carrier_user_id),
            Some("sanctions-blocked"),
        )
        .await
        .is_err()
    );

    upsert_sanctions_tax_profile(
        &pool,
        UpsertSanctionsTaxProfileParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            user_id: fixture.carrier_user_id,
            sanctions_status: "clear",
            sanctions_provider: "manual_ofac_review",
            sanctions_reference: Some("OFAC-CLEAR-1003"),
            beneficial_owner_status: "clear",
            tax_document_status: "verified",
            tax_document_type: Some("W-9"),
            tin_masked: Some("***-**-6789"),
            tax_reporting_owner: "internal_finance",
            tax_year: Some(2026),
            payout_tax_blocking: true,
            notes: Some("Tax and sanctions cleared"),
            reviewed_by_user_id: None,
        },
    )
    .await?;
    assert!(
        sanctions_tax_booking_blocker(&pool, fixture.carrier_user_id)
            .await?
            .is_none()
    );
    assert!(
        sanctions_tax_payout_blocker(&pool, fixture.carrier_user_id)
            .await?
            .is_none()
    );

    create_risk_review_item(
        &pool,
        CreateRiskReviewItemParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            subject_user_id: Some(fixture.carrier_user_id),
            load_id: Some(fixture.load_id),
            leg_id: Some(fixture.leg_id),
            review_type: "double_brokering".into(),
            severity: "high".into(),
            score: 88,
            reasons: vec![
                "mismatched authority".into(),
                "new payout destination before pickup".into(),
            ],
            evidence: serde_json::json!({
                "authority_mismatch": true,
                "payout_destination_age_hours": 2
            }),
            hold_booking: true,
            hold_payout: true,
            communication_required: true,
            provider_notification_required: false,
        },
    )
    .await?;
    let booking_blocker =
        risk_review_booking_blocker(&pool, fixture.carrier_user_id, fixture.leg_id).await?;
    assert!(
        booking_blocker
            .as_deref()
            .unwrap_or_default()
            .contains("double_brokering")
    );
    assert!(
        book_load_leg(
            &pool,
            fixture.leg_id,
            fixture.carrier_user_id,
            None,
            Some(fixture.carrier_user_id),
            Some("risk-review-blocked"),
        )
        .await
        .is_err()
    );

    create_risk_review_item(
        &pool,
        CreateRiskReviewItemParams {
            organization_id: Some(DEFAULT_ORGANIZATION_ID),
            subject_user_id: Some(fixture.carrier_user_id),
            load_id: Some(fixture.load_id),
            leg_id: Some(fixture.leg_id),
            review_type: "account_takeover".into(),
            severity: "critical".into(),
            score: 96,
            reasons: vec![
                "impossible travel login".into(),
                "failed MFA before payout change".into(),
            ],
            evidence: serde_json::json!({
                "ip_velocity": "impossible_travel",
                "mfa_failures": 4
            }),
            hold_booking: false,
            hold_payout: true,
            communication_required: true,
            provider_notification_required: true,
        },
    )
    .await?;
    let payout_blocker =
        risk_review_payout_blocker(&pool, fixture.carrier_user_id, fixture.leg_id).await?;
    assert!(
        payout_blocker
            .as_deref()
            .unwrap_or_default()
            .contains("account_takeover")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn required_document_rules_seed_enterprise_checklists()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let carrier_onboarding = list_required_document_rules(
        &pool,
        "onboarding",
        "submit_onboarding",
        Some("carrier"),
        None,
    )
    .await?;
    assert!(
        carrier_onboarding
            .iter()
            .any(|rule| rule.rule_key == "carrier_operating_authority")
    );
    assert!(
        carrier_onboarding
            .iter()
            .any(|rule| rule.document_type_key == "insurance_certificate")
    );
    assert!(carrier_onboarding.iter().all(|rule| rule.blocks_transition));

    let execution_closeout =
        list_required_document_rules(&pool, "execution", "complete_delivery", None, None).await?;
    assert_eq!(execution_closeout.len(), 1);
    assert_eq!(execution_closeout[0].document_type_key, "delivery_pod");

    Ok(())
}

#[tokio::test]
#[serial]
async fn freight_document_templates_seed_and_link_generated_documents()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;
    let load_id: i64 = sqlx::query_scalar("SELECT load_id FROM load_legs WHERE id = $1")
        .bind(fixture.leg_id)
        .fetch_one(&pool)
        .await?;
    let templates = list_active_freight_document_templates(&pool).await?;
    assert!(
        templates
            .iter()
            .any(|item| item.template_key == "rate_confirmation")
    );
    assert!(
        templates
            .iter()
            .any(|item| item.template_key == "bill_of_lading")
    );
    assert!(
        templates
            .iter()
            .any(|item| item.template_key == "carrier_packet")
    );

    let context = load_freight_document_context(&pool, load_id)
        .await?
        .expect("load context should render documents");
    assert_eq!(context.load_number.as_deref(), Some("LD-IT-1001"));
    assert_eq!(context.carrier_name.as_deref(), Some("Carrier Ops"));

    let document = create_load_document(
        &pool,
        load_id,
        &UpsertLoadDocumentParams {
            document_name: "Rate Confirmation".into(),
            document_type: "rate_confirmation".into(),
            file_path: "local://loads/LD-IT-1001/rate-confirmation.txt".into(),
            storage_provider: "local".into(),
            original_name: Some("LD-IT-1001-rate-confirmation.txt".into()),
            mime_type: Some("text/plain".into()),
            file_size: Some(128),
        },
        Some(fixture.shipper_user_id),
    )
    .await?
    .expect("generated document should link to load");

    let generated = record_generated_freight_document(
        &pool,
        load_id,
        document.id,
        "rate_confirmation",
        "2026-05-25",
        Some(fixture.shipper_user_id),
    )
    .await?;
    assert_eq!(generated.load_id, load_id);
    assert_eq!(generated.document_id, document.id);
    assert_eq!(generated.generation_status, "generated");

    Ok(())
}

#[tokio::test]
#[serial]
async fn enterprise_load_model_persists_and_updates_operational_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let shipper_user_id = insert_user(
        &pool,
        "Enterprise Shipper",
        "enterprise-shipper@example.test",
    )
    .await?;
    let load_type = master_data::upsert_load_type(&pool, None, "Enterprise Freight").await?;
    let equipment = master_data::upsert_equipment(&pool, None, "Reefer Van").await?;
    let commodity = master_data::upsert_commodity_type(&pool, None, "Fresh Produce").await?;
    let pickup = master_data::upsert_location(&pool, None, "Dallas DC", None, None).await?;
    let delivery = master_data::upsert_location(&pool, None, "Joliet DC", None, None).await?;
    let appointment_start = chrono::NaiveDate::from_ymd_opt(2026, 6, 10)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let appointment_end = chrono::NaiveDate::from_ymd_opt(2026, 6, 10)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let pickup_date = chrono::NaiveDate::from_ymd_opt(2026, 6, 10)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let delivery_date = chrono::NaiveDate::from_ymd_opt(2026, 6, 12)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let params = CreateLoadParams {
        title: "Enterprise produce tender".into(),
        owner_user_id: shipper_user_id,
        load_type_id: load_type.id,
        equipment_id: equipment.id,
        commodity_type_id: commodity.id,
        customer_contract_id: None,
        customer_contract_lane_id: None,
        contract_rate: None,
        contract_rate_currency: None,
        contract_posting_behavior: None,
        contract_service_rules: None,
        freight_mode: "FTL".into(),
        visibility: "contract".into(),
        service_level: Some("expedited".into()),
        customer_reference: Some("CUST-7788".into()),
        po_number: Some("PO-4411".into()),
        pickup_appointment_ref: Some("PU-APT-1".into()),
        delivery_appointment_ref: Some("DL-APT-1".into()),
        facility_contact_name: Some("Dock Lead".into()),
        facility_contact_phone: Some("+1 555 0100".into()),
        facility_contact_email: Some("dock@example.test".into()),
        appointment_window_start: Some(appointment_start),
        appointment_window_end: Some(appointment_end),
        accessorial_flags: Some(serde_json::json!({"lumper": true, "detention": "2h free"})),
        weight_unit: "LBS".into(),
        weight: 42000.0,
        temperature_data: Some(serde_json::json!({"min_f": 34, "max_f": 38})),
        container_data: Some(serde_json::json!({"seal_required": true})),
        securement_data: Some(serde_json::json!({"load_bars": 2})),
        special_instructions: Some("Pulp checks at pickup and delivery.".into()),
        is_hazardous: false,
        is_temperature_controlled: true,
    };

    let created = create_load_with_legs(
        &pool,
        &params,
        &[CreateLoadLegParams {
            pickup_location_id: pickup.id,
            delivery_location_id: delivery.id,
            pickup_date,
            delivery_date,
            bid_status: "Fixed".into(),
            price: 2450.0,
        }],
        Some(shipper_user_id),
    )
    .await?;

    let saved = find_load_by_id(&pool, created.load_id)
        .await?
        .expect("created enterprise load should be readable");
    assert_eq!(saved.visibility, "contract");
    assert_eq!(saved.service_level.as_deref(), Some("expedited"));
    assert_eq!(saved.customer_reference.as_deref(), Some("CUST-7788"));
    assert_eq!(
        saved
            .temperature_data
            .as_ref()
            .and_then(|value| value.get("min_f")),
        Some(&serde_json::json!(34))
    );

    let mut updated_params = params;
    updated_params.visibility = "internal".into();
    updated_params.service_level = Some("team".into());
    updated_params.accessorial_flags = Some(serde_json::json!(["liftgate", "inside_delivery"]));

    update_load_with_legs(
        &pool,
        created.load_id,
        &updated_params,
        &[CreateLoadLegParams {
            pickup_location_id: pickup.id,
            delivery_location_id: delivery.id,
            pickup_date,
            delivery_date,
            bid_status: "Open".into(),
            price: 2600.0,
        }],
        Some(shipper_user_id),
    )
    .await?
    .expect("enterprise load update should succeed");

    let updated = find_load_by_id(&pool, created.load_id)
        .await?
        .expect("updated enterprise load should be readable");
    assert_eq!(updated.visibility, "internal");
    assert_eq!(updated.service_level.as_deref(), Some("team"));
    assert_eq!(
        updated.accessorial_flags,
        Some(serde_json::json!(["liftgate", "inside_delivery"]))
    );

    let published = update_load_lifecycle(
        &pool,
        created.load_id,
        "published",
        Some("ready for carrier visibility"),
        None,
        Some(shipper_user_id),
    )
    .await?
    .expect("publish lifecycle action should update load");
    assert_eq!(published.lifecycle_status, "published");
    assert!(published.published_at.is_some());

    let revised = update_load_lifecycle(
        &pool,
        created.load_id,
        "revised",
        Some("customer changed appointment details"),
        Some("Produce lane template"),
        Some(shipper_user_id),
    )
    .await?
    .expect("revise lifecycle action should update load");
    assert_eq!(revised.lifecycle_status, "revised");
    assert_eq!(revised.revision_number, 2);
    assert!(revised.is_template);
    assert_eq!(
        revised.template_name.as_deref(),
        Some("Produce lane template")
    );

    let cloned = clone_load_as_draft(&pool, created.load_id, Some(shipper_user_id))
        .await?
        .expect("clone should create a draft copy");
    let cloned_load = find_load_by_id(&pool, cloned.load_id)
        .await?
        .expect("cloned draft should be readable");
    assert_eq!(cloned_load.lifecycle_status, "draft");
    assert_eq!(cloned_load.visibility, "private");
    assert_eq!(cloned_load.cloned_from_load_id, Some(created.load_id));

    Ok(())
}

#[tokio::test]
#[serial]
async fn customer_contract_lane_guides_attach_contract_pricing_to_loads()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let shipper_user_id =
        insert_user(&pool, "Contract Shipper", "contract-shipper@example.test").await?;
    let load_type = master_data::upsert_load_type(&pool, None, "Contract Freight").await?;
    let equipment = master_data::upsert_equipment(&pool, None, "Dry Van").await?;
    let commodity = master_data::upsert_commodity_type(&pool, None, "Retail Goods").await?;
    let pickup =
        master_data::upsert_location(&pool, None, "Dallas Contract DC", None, None).await?;
    let delivery =
        master_data::upsert_location(&pool, None, "Chicago Contract DC", None, None).await?;
    let effective_start = chrono::Utc::now().date_naive() - chrono::Days::new(1);
    let pickup_date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let delivery_date = chrono::NaiveDate::from_ymd_opt(2026, 7, 3)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let contract = create_customer_contract(
        &pool,
        &CreateCustomerContractParams {
            organization_id: 1,
            customer_user_id: Some(shipper_user_id),
            contract_number: "CTR-2026-001".into(),
            contract_name: "National Retail Contract".into(),
            effective_start,
            effective_end: None,
            default_currency: "USD".into(),
            service_rules: Some(serde_json::json!({"tender_cutoff_hours": 24})),
        },
    )
    .await?;

    let lane = create_customer_contract_lane(
        &pool,
        &CreateCustomerContractLaneParams {
            organization_id: 1,
            contract_id: contract.id,
            lane_name: "Dallas to Chicago".into(),
            origin_location_id: Some(pickup.id),
            destination_location_id: Some(delivery.id),
            equipment_id: Some(equipment.id),
            commodity_type_id: Some(commodity.id),
            freight_mode: "FTL".into(),
            contracted_rate: 3100.0,
            rate_currency: "USD".into(),
            accessorial_rules: Some(serde_json::json!({"detention": "2h free"})),
            service_level: Some("contract_standard".into()),
            posting_behavior: "contract".into(),
            tender_priority: 10,
            effective_start,
            effective_end: None,
            service_rules: None,
        },
    )
    .await?;

    let active_lane = db::dispatch::find_active_customer_contract_lane(&pool, 1, lane.id)
        .await?
        .expect("active lane should be found");
    assert_eq!(active_lane.contract_id, contract.id);
    assert_eq!(active_lane.contracted_rate, 3100.0);
    assert_eq!(active_lane.posting_behavior, "contract");

    let created = create_load_with_legs(
        &pool,
        &CreateLoadParams {
            title: "Contract retail lane".into(),
            owner_user_id: shipper_user_id,
            load_type_id: load_type.id,
            equipment_id: equipment.id,
            commodity_type_id: commodity.id,
            customer_contract_id: Some(contract.id),
            customer_contract_lane_id: Some(lane.id),
            contract_rate: Some(active_lane.contracted_rate),
            contract_rate_currency: Some(active_lane.rate_currency.clone()),
            contract_posting_behavior: Some(active_lane.posting_behavior.clone()),
            contract_service_rules: active_lane.service_rules.clone(),
            freight_mode: active_lane.freight_mode.clone(),
            visibility: active_lane.posting_behavior.clone(),
            service_level: active_lane.service_level.clone(),
            customer_reference: Some("RET-LOAD-9001".into()),
            po_number: Some("RET-PO-9001".into()),
            pickup_appointment_ref: None,
            delivery_appointment_ref: None,
            facility_contact_name: None,
            facility_contact_phone: None,
            facility_contact_email: None,
            appointment_window_start: None,
            appointment_window_end: None,
            accessorial_flags: active_lane.accessorial_rules.clone(),
            weight_unit: "LBS".into(),
            weight: 38000.0,
            temperature_data: None,
            container_data: None,
            securement_data: None,
            special_instructions: Some("Use contracted carrier routing guide.".into()),
            is_hazardous: false,
            is_temperature_controlled: false,
        },
        &[CreateLoadLegParams {
            pickup_location_id: pickup.id,
            delivery_location_id: delivery.id,
            pickup_date,
            delivery_date,
            bid_status: "Fixed".into(),
            price: active_lane.contracted_rate,
        }],
        Some(shipper_user_id),
    )
    .await?;

    let load = find_load_by_id(&pool, created.load_id)
        .await?
        .expect("contract load should be readable");
    assert_eq!(load.visibility, "contract");
    assert_eq!(load.customer_contract_id, Some(contract.id));
    assert_eq!(load.customer_contract_lane_id, Some(lane.id));
    assert_eq!(load.contract_rate, Some(3100.0));
    assert_eq!(load.service_level.as_deref(), Some("contract_standard"));

    sqlx::query(
        "INSERT INTO load_board_saved_filters (
            organization_id, user_id, role_key, name, filter_payload, is_default, created_at, updated_at
         ) VALUES (1, $1, 'shipper', 'Contract Dallas lanes', $2, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(shipper_user_id)
    .bind(serde_json::json!({
        "origin": "Dallas",
        "destination": "Chicago",
        "visibility": "contract",
        "page": 1,
        "per_page": 10
    }))
    .execute(&pool)
    .await?;

    let filters = shared::LoadBoardFilters {
        origin: Some("Dallas".into()),
        destination: Some("Chicago".into()),
        visibility: Some("contract".into()),
        page: 1,
        per_page: 10,
        ..Default::default()
    };
    let search = load_board_search(
        &pool,
        Some(domain::auth::UserRole::Shipper),
        shipper_user_id,
        &filters,
        Some("all"),
    )
    .await?;
    assert_eq!(search.total, 1);
    assert_eq!(search.rows[0].load_id, created.load_id);

    let saved_filters =
        list_load_board_saved_filters(&pool, 1, shipper_user_id, Some("shipper")).await?;
    assert_eq!(saved_filters.len(), 1);
    assert_eq!(saved_filters[0].name, "Contract Dallas lanes");

    Ok(())
}

#[tokio::test]
#[serial]
async fn private_network_rules_filter_carrier_load_board_visibility()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let shipper_user_id = insert_user(
        &pool,
        "Private Network Shipper",
        "private-network-shipper@example.test",
    )
    .await?;
    let carrier_user_id = insert_user(
        &pool,
        "Private Network Carrier",
        "private-network-carrier@example.test",
    )
    .await?;
    let load_type = master_data::upsert_load_type(&pool, None, "Private Freight").await?;
    let equipment = master_data::upsert_equipment(&pool, None, "Dry Van").await?;
    let commodity = master_data::upsert_commodity_type(&pool, None, "Paper").await?;
    let pickup = master_data::upsert_location(&pool, None, "Dallas Private DC", None, None).await?;
    let delivery =
        master_data::upsert_location(&pool, None, "Chicago Private DC", None, None).await?;
    let pickup_date = chrono::NaiveDate::from_ymd_opt(2026, 8, 10)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let delivery_date = chrono::NaiveDate::from_ymd_opt(2026, 8, 12)
        .unwrap()
        .and_hms_opt(16, 0, 0)
        .unwrap();

    let created = create_load_with_legs(
        &pool,
        &CreateLoadParams {
            title: "Private network only freight".into(),
            owner_user_id: shipper_user_id,
            load_type_id: load_type.id,
            equipment_id: equipment.id,
            commodity_type_id: commodity.id,
            customer_contract_id: None,
            customer_contract_lane_id: None,
            contract_rate: None,
            contract_rate_currency: None,
            contract_posting_behavior: None,
            contract_service_rules: None,
            freight_mode: "FTL".into(),
            visibility: "private".into(),
            service_level: Some("standard".into()),
            customer_reference: None,
            po_number: None,
            pickup_appointment_ref: None,
            delivery_appointment_ref: None,
            facility_contact_name: None,
            facility_contact_phone: None,
            facility_contact_email: None,
            appointment_window_start: None,
            appointment_window_end: None,
            accessorial_flags: None,
            weight_unit: "LBS".into(),
            weight: 40000.0,
            temperature_data: None,
            container_data: None,
            securement_data: None,
            special_instructions: None,
            is_hazardous: false,
            is_temperature_controlled: false,
        },
        &[CreateLoadLegParams {
            pickup_location_id: pickup.id,
            delivery_location_id: delivery.id,
            pickup_date,
            delivery_date,
            bid_status: "Open".into(),
            price: 1800.0,
        }],
        Some(shipper_user_id),
    )
    .await?;

    let filters = shared::LoadBoardFilters {
        page: 1,
        per_page: 25,
        ..Default::default()
    };
    let hidden = load_board_search(
        &pool,
        Some(domain::auth::UserRole::Carrier),
        carrier_user_id,
        &filters,
        Some("all"),
    )
    .await?;
    assert!(hidden.rows.iter().all(|row| row.load_id != created.load_id));

    sqlx::query(
        "INSERT INTO carrier_network_memberships (
            owner_user_id, carrier_user_id, relationship_status, carrier_group_key,
            created_by_user_id, created_at, updated_at
         )
         VALUES ($1, $2, 'preferred', 'primary', $1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(shipper_user_id)
    .bind(carrier_user_id)
    .execute(&pool)
    .await?;

    let visible = load_board_search(
        &pool,
        Some(domain::auth::UserRole::Carrier),
        carrier_user_id,
        &filters,
        Some("all"),
    )
    .await?;
    assert!(
        visible
            .rows
            .iter()
            .any(|row| row.load_id == created.load_id)
    );

    sqlx::query(
        "UPDATE carrier_network_memberships
         SET relationship_status = 'blocked', updated_at = CURRENT_TIMESTAMP
         WHERE owner_user_id = $1 AND carrier_user_id = $2",
    )
    .bind(shipper_user_id)
    .bind(carrier_user_id)
    .execute(&pool)
    .await?;

    let blocked = load_board_search(
        &pool,
        Some(domain::auth::UserRole::Carrier),
        carrier_user_id,
        &filters,
        Some("all"),
    )
    .await?;
    assert!(
        blocked
            .rows
            .iter()
            .all(|row| row.load_id != created.load_id)
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn kyc_document_replacement_preserves_version_history()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let user_id = insert_user(
        &pool,
        "Document Version User",
        "document-version@example.com",
    )
    .await?;

    let created = create_kyc_document(
        &pool,
        &CreateKycDocumentInput {
            user_id,
            document_name: "Operating Authority".into(),
            document_type: "standard".into(),
            file_path: "local://kyc/authority-v1.pdf".into(),
            original_name: Some("authority-v1.pdf".into()),
            mime_type: Some("application/pdf".into()),
            file_size: Some(1000),
        },
    )
    .await?;
    assert_eq!(created.current_version, 1);
    assert_eq!(created.version_count, 1);

    let updated = update_kyc_document(
        &pool,
        created.id,
        user_id,
        &UpdateKycDocumentInput {
            document_name: "Operating Authority".into(),
            document_type: "standard".into(),
            file_path: Some("local://kyc/authority-v2.pdf".into()),
            original_name: Some("authority-v2.pdf".into()),
            mime_type: Some("application/pdf".into()),
            file_size: Some(2000),
            next_status: 2,
        },
    )
    .await?
    .expect("owned document should update");
    assert_eq!(updated.file_path, "local://kyc/authority-v2.pdf");

    let listed = list_kyc_documents_by_user_id(&pool, user_id).await?;
    let document = listed
        .iter()
        .find(|document| document.id == created.id)
        .expect("replaced document should remain visible");
    assert_eq!(document.current_version, 2);
    assert_eq!(document.version_count, 2);

    let version_paths = sqlx::query(
        "SELECT version_number, file_path, original_name, replacement_reason
         FROM kyc_document_versions
         WHERE document_id = $1
         ORDER BY version_number",
    )
    .bind(created.id)
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| {
        (
            row.get::<i32, _>("version_number"),
            row.get::<String, _>("file_path"),
            row.get::<Option<String>, _>("original_name"),
            row.get::<Option<String>, _>("replacement_reason"),
        )
    })
    .collect::<Vec<_>>();

    assert_eq!(version_paths.len(), 2);
    assert_eq!(version_paths[0].0, 1);
    assert_eq!(version_paths[0].1, "local://kyc/authority-v1.pdf");
    assert_eq!(version_paths[0].2.as_deref(), Some("authority-v1.pdf"));
    assert_eq!(version_paths[0].3.as_deref(), Some("initial upload"));
    assert_eq!(version_paths[1].0, 2);
    assert_eq!(version_paths[1].1, "local://kyc/authority-v2.pdf");
    assert_eq!(version_paths[1].2.as_deref(), Some("authority-v2.pdf"));
    assert_eq!(
        version_paths[1].3.as_deref(),
        Some("document replacement or metadata update")
    );

    let verified = verify_kyc_document_blockchain(
        &pool,
        created.id,
        user_id,
        "40f0128ed5f35ecbe4f84744712d90b546e8ddbb4b7dc6304469a750061ca547",
        Some("integration test content hash"),
        2,
    )
    .await?
    .expect("owned document should accept content hash");
    assert_eq!(
        verified.hash.as_deref(),
        Some("40f0128ed5f35ecbe4f84744712d90b546e8ddbb4b7dc6304469a750061ca547")
    );
    assert_eq!(verified.hash_algorithm.as_deref(), Some("sha256"));
    assert!(
        !verified
            .hash
            .as_deref()
            .unwrap_or_default()
            .contains("mock")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn legal_agreements_block_until_acceptance_and_write_audit()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let organization_id = insert_organization(&pool, "legal-agreements").await?;
    let user_id = insert_user_in_organization(
        &pool,
        organization_id,
        "Legal Carrier",
        "legal-carrier@example.com",
    )
    .await?;

    let missing_before =
        list_missing_required_agreements(&pool, user_id, organization_id, Some("carrier")).await?;
    assert!(
        missing_before
            .iter()
            .any(|agreement| agreement.agreement_key == "platform_terms")
    );
    assert!(
        missing_before
            .iter()
            .any(|agreement| agreement.agreement_key == "carrier_operating_agreement")
    );

    let acceptance = accept_latest_legal_agreement(
        &pool,
        &AcceptLegalAgreementInput {
            agreement_key: "platform_terms",
            signer_user_id: user_id,
            organization_id,
            role_key: Some("carrier"),
            signer_name: "Legal Carrier",
            signer_email: "legal-carrier@example.com",
            ip_address: Some("203.0.113.10"),
            user_agent: Some("integration-test"),
            request_id: Some("req_legal_acceptance_test"),
            accept_for_organization: false,
        },
    )
    .await?;

    assert_eq!(acceptance.agreement_key, "platform_terms");
    assert_eq!(acceptance.user_id, Some(user_id));
    assert!(acceptance_has_audit_event(&pool, acceptance.id).await?);

    let missing_after =
        list_missing_required_agreements(&pool, user_id, organization_id, Some("carrier")).await?;
    assert!(
        !missing_after
            .iter()
            .any(|agreement| agreement.agreement_key == "platform_terms")
    );
    assert!(
        missing_after
            .iter()
            .any(|agreement| agreement.agreement_key == "carrier_operating_agreement")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn operating_authority_tracks_customer_evidence_and_renewal_alerts()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let decisions = list_operating_authority_decisions(&pool).await?;
    assert!(
        decisions
            .iter()
            .any(|decision| decision.model_key == "software_tms_extension"
                && decision.scope == "in_scope_first_release")
    );
    assert!(
        decisions
            .iter()
            .any(|decision| decision.model_key == "broker_of_record"
                && decision.scope == "explicitly_out_of_scope")
    );

    let customer_package = list_customer_disclosable_evidence(&pool).await?;
    assert!(
        customer_package
            .iter()
            .any(|record| record.evidence_key == "cyber_liability")
    );
    assert!(
        customer_package
            .iter()
            .any(|record| record.evidence_key == "broker_authority")
    );

    let soon = chrono::Utc::now().date_naive() + chrono::Days::new(15);
    upsert_compliance_evidence_document(
        &pool,
        "cyber_liability",
        "compliance/cyber-liability-coi.pdf",
        Some("Enterprise Mutual"),
        Some("CYBER-2026"),
        Some(chrono::Utc::now().date_naive()),
        Some(soon),
    )
    .await?;

    let due = list_evidence_due_for_review(&pool, 30).await?;
    assert!(
        due.iter()
            .any(|record| record.evidence_key == "cyber_liability"
                && record.document_uri.as_deref() == Some("compliance/cyber-liability-coi.pdf"))
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn tenant_scoped_queries_reject_cross_organization_records()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let tenant_a = insert_organization(&pool, "tenant-isolation-a").await?;
    let tenant_b = insert_organization(&pool, "tenant-isolation-b").await?;
    let shipper_a =
        insert_user_in_organization(&pool, tenant_a, "Tenant A Shipper", "tenant-a@example.com")
            .await?;
    let carrier_a = insert_user_in_organization(
        &pool,
        tenant_a,
        "Tenant A Carrier",
        "tenant-a-carrier@example.com",
    )
    .await?;
    let shipper_b =
        insert_user_in_organization(&pool, tenant_b, "Tenant B Shipper", "tenant-b@example.com")
            .await?;
    let carrier_b = insert_user_in_organization(
        &pool,
        tenant_b,
        "Tenant B Carrier",
        "tenant-b-carrier@example.com",
    )
    .await?;
    let pickup_location_id = insert_location(&pool, "Tenant isolation pickup").await?;
    let delivery_location_id = insert_location(&pool, "Tenant isolation delivery").await?;

    async fn insert_scoped_load_leg(
        pool: &DbPool,
        organization_id: i64,
        owner_user_id: i64,
        carrier_user_id: i64,
        load_number: &str,
        pickup_location_id: i64,
        delivery_location_id: i64,
    ) -> Result<(i64, i64), sqlx::Error> {
        let load_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO loads (
                organization_id, load_number, title, user_id, weight_unit, weight, status, leg_count, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, 'lbs', 42000, 1, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .bind(organization_id)
        .bind(load_number)
        .bind(format!("{} scoped load", load_number))
        .bind(owner_user_id)
        .fetch_one(pool)
        .await?;

        let leg_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO load_legs (
                load_id, leg_no, leg_code, pickup_location_id, delivery_location_id, bid_status,
                price, status_id, booked_carrier_id, booked_amount, created_at, updated_at
             ) VALUES ($1, 1, $2, $3, $4, 'Fixed', 1200, 4, $5, 1200, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .bind(load_id)
        .bind(format!("{}-1", load_number))
        .bind(pickup_location_id)
        .bind(delivery_location_id)
        .bind(carrier_user_id)
        .fetch_one(pool)
        .await?;

        Ok((load_id, leg_id))
    }

    let (_load_a, leg_a) = insert_scoped_load_leg(
        &pool,
        tenant_a,
        shipper_a,
        carrier_a,
        "LD-TENANT-A",
        pickup_location_id,
        delivery_location_id,
    )
    .await?;
    let (load_b, leg_b) = insert_scoped_load_leg(
        &pool,
        tenant_b,
        shipper_b,
        carrier_b,
        "LD-TENANT-B",
        pickup_location_id,
        delivery_location_id,
    )
    .await?;

    let document_b = sqlx::query_scalar::<_, i64>(
        "INSERT INTO load_documents (
            organization_id, load_id, document_name, document_type, file_path, storage_provider, uploaded_by_user_id, created_at, updated_at
         ) VALUES ($1, $2, 'BOL', 'bol', '/secure/tenant-b.pdf', 'local', $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_b)
    .bind(load_b)
    .bind(shipper_b)
    .fetch_one(&pool)
    .await?;

    let conversation_a = sqlx::query_scalar::<_, i64>(
        "INSERT INTO conversations (organization_id, load_leg_id, shipper_id, carrier_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_a)
    .bind(leg_a)
    .bind(shipper_a)
    .bind(carrier_a)
    .fetch_one(&pool)
    .await?;
    let conversation_b = sqlx::query_scalar::<_, i64>(
        "INSERT INTO conversations (organization_id, load_leg_id, shipper_id, carrier_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_b)
    .bind(leg_b)
    .bind(shipper_b)
    .bind(carrier_b)
    .fetch_one(&pool)
    .await?;
    sqlx::query(
        "INSERT INTO messages (conversation_id, user_id, body, created_at, updated_at)
         VALUES ($1, $2, 'tenant A message', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ($3, $4, 'tenant B message', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(conversation_a)
    .bind(shipper_a)
    .bind(conversation_b)
    .bind(shipper_b)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO escrows (
            organization_id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, 'USD', 120000, 0, 'funded', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_b)
    .bind(leg_b)
    .bind(shipper_b)
    .bind(carrier_b)
    .execute(&pool)
    .await?;

    let handoff_b = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_handoffs (
            organization_id, tms_load_id, tenant_id, load_id, status, queued_at, created_at, updated_at
         ) VALUES ($1, 'TMS-TENANT-B', 'tenant-b', $2, 'queued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_b)
    .bind(load_b)
    .fetch_one(&pool)
    .await?;

    let leg_scope_b = db::dispatch::find_load_leg_scope(&pool, leg_b)
        .await?
        .expect("tenant B leg scope should exist");
    assert_eq!(leg_scope_b.organization_id, tenant_b);
    assert_ne!(leg_scope_b.organization_id, tenant_a);

    let document_scope_b = db::dispatch::find_load_document_scope(&pool, document_b)
        .await?
        .expect("tenant B document scope should exist");
    assert_eq!(document_scope_b.organization_id, tenant_b);
    assert_ne!(document_scope_b.organization_id, tenant_a);

    assert!(
        find_escrow_for_leg_in_organization(&pool, leg_b, tenant_a)
            .await?
            .is_none()
    );
    assert!(
        find_escrow_for_leg_in_organization(&pool, leg_b, tenant_b)
            .await?
            .is_some()
    );

    let tenant_a_conversations = list_recent_conversation_workspace_records_for_user(
        &pool,
        shipper_a,
        Some(domain::auth::UserRole::Admin),
        Some(tenant_a),
        25,
    )
    .await?;
    assert!(
        tenant_a_conversations
            .iter()
            .any(|row| row.id == conversation_a)
    );
    assert!(
        !tenant_a_conversations
            .iter()
            .any(|row| row.id == conversation_b)
    );

    assert!(!handoff_belongs_to_organization(&pool, handoff_b, tenant_a).await?);
    assert!(handoff_belongs_to_organization(&pool, handoff_b, tenant_b).await?);

    Ok(())
}

async fn insert_location(pool: &DbPool, name: &str) -> Result<i64, sqlx::Error> {
    let location_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO locations (name, city_id, country_id, created_at, updated_at)
         VALUES ($1, NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(name)
    .fetch_one(pool)
    .await?;

    Ok(location_id)
}

async fn insert_booked_leg_fixture(pool: &DbPool) -> Result<LegFixture, sqlx::Error> {
    let shipper_user_id =
        insert_user(pool, "Shipper Ops", "shipper.integration@example.com").await?;
    let carrier_user_id =
        insert_user(pool, "Carrier Ops", "carrier.integration@example.com").await?;
    let pickup_location_id = insert_location(pool, "Dallas pickup").await?;
    let delivery_location_id = insert_location(pool, "Memphis delivery").await?;

    let load_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO loads (
            load_number, title, user_id, weight_unit, weight, status, leg_count, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, 1, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind("LD-IT-1001")
    .bind("Integration test load")
    .bind(shipper_user_id)
    .bind("lbs")
    .bind(42000.0_f64)
    .fetch_one(pool)
    .await?;

    let leg_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO load_legs (
            load_id, leg_no, leg_code, pickup_location_id, delivery_location_id,
            pickup_date, delivery_date, bid_status, price, status_id, booked_carrier_id,
            booked_at, booked_amount, created_at, updated_at
         ) VALUES (
            $1, 1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '1 day',
            $5, $6, 4, $7, CURRENT_TIMESTAMP, $8, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(load_id)
    .bind("LD-IT-1001-1")
    .bind(pickup_location_id)
    .bind(delivery_location_id)
    .bind("Fixed")
    .bind(2450.0_f64)
    .bind(carrier_user_id)
    .bind(2450.0_f64)
    .fetch_one(pool)
    .await?;

    Ok(LegFixture {
        shipper_user_id,
        carrier_user_id,
        load_id,
        leg_id,
    })
}

async fn insert_open_booking_race_fixture(pool: &DbPool) -> Result<LegFixture, sqlx::Error> {
    let shipper_user_id = insert_user(
        pool,
        "Race Shipper Ops",
        "race.shipper.integration@example.com",
    )
    .await?;
    let carrier_user_id = insert_user(
        pool,
        "Race Carrier Ops",
        "race.carrier.integration@example.com",
    )
    .await?;
    let pickup_location_id = insert_location(pool, "Race Dallas pickup").await?;
    let delivery_location_id = insert_location(pool, "Race Memphis delivery").await?;

    let load_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO loads (
            load_number, title, user_id, weight_unit, weight, status, leg_count, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, 1, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind("LD-RACE-1001")
    .bind("Booking race integration load")
    .bind(shipper_user_id)
    .bind("lbs")
    .bind(42000.0_f64)
    .fetch_one(pool)
    .await?;

    let leg_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO load_legs (
            load_id, leg_no, leg_code, pickup_location_id, delivery_location_id,
            pickup_date, delivery_date, bid_status, price, status_id, created_at, updated_at
         ) VALUES (
            $1, 1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '1 day',
            $5, $6, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(load_id)
    .bind("LD-RACE-1001-1")
    .bind(pickup_location_id)
    .bind(delivery_location_id)
    .bind("Fixed")
    .bind(2450.0_f64)
    .fetch_one(pool)
    .await?;

    Ok(LegFixture {
        shipper_user_id,
        carrier_user_id,
        load_id,
        leg_id,
    })
}

#[tokio::test]
#[serial]
async fn booking_race_allows_only_one_carrier_and_replays_idempotency()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };
    reset_database(&pool).await?;
    seed_load_statuses(&pool).await?;
    let fixture = insert_open_booking_race_fixture(&pool).await?;
    let second_carrier_id = insert_user(
        &pool,
        "Race Carrier Two",
        "race.carrier.two.integration@example.com",
    )
    .await?;
    seed_valid_carrier_authority(&pool, fixture.carrier_user_id).await?;
    seed_valid_carrier_authority(&pool, second_carrier_id).await?;

    let first = book_load_leg(
        &pool,
        fixture.leg_id,
        fixture.carrier_user_id,
        None,
        Some(fixture.carrier_user_id),
        Some("race-key-1"),
    )
    .await?;
    let second = book_load_leg(
        &pool,
        fixture.leg_id,
        second_carrier_id,
        None,
        Some(second_carrier_id),
        Some("race-key-2"),
    )
    .await;
    let replay = book_load_leg(
        &pool,
        fixture.leg_id,
        fixture.carrier_user_id,
        None,
        Some(fixture.carrier_user_id),
        Some("race-key-1"),
    )
    .await?;

    assert_eq!(
        first.as_ref().and_then(|leg| leg.booked_carrier_id),
        Some(fixture.carrier_user_id)
    );
    assert!(second.is_err());
    assert_eq!(
        replay.as_ref().and_then(|leg| leg.booked_carrier_id),
        Some(fixture.carrier_user_id)
    );

    Ok(())
}

fn sample_tms_payload(tms_load_id: &str) -> TmsHandoffPayload {
    TmsHandoffPayload {
        tms_load_id: tms_load_id.into(),
        tenant_id: "tenant-it".into(),
        external_handoff_id: Some(format!("{}-external", tms_load_id)),
        party_type: "shipper".into(),
        freight_mode: "FTL".into(),
        equipment_type: "Dry Van".into(),
        commodity_description: Some("Paper goods".into()),
        weight: 42000.0,
        weight_unit: "lbs".into(),
        piece_count: Some(20),
        is_hazardous: Some(false),
        temperature_data: None,
        container_data: None,
        securement_data: None,
        pickup_city: "Dallas".into(),
        pickup_state: Some("TX".into()),
        pickup_zip: Some("75201".into()),
        pickup_country: "US".into(),
        pickup_address: "100 Market St, Dallas, TX".into(),
        pickup_window_start: "2026-04-07T09:00:00Z".into(),
        pickup_window_end: Some("2026-04-07T12:00:00Z".into()),
        pickup_instructions: Some("Check in at dock 4".into()),
        pickup_appointment_ref: Some("PU-IT-1001".into()),
        dropoff_city: "Memphis".into(),
        dropoff_state: Some("TN".into()),
        dropoff_zip: Some("38103".into()),
        dropoff_country: "US".into(),
        dropoff_address: "200 Carrier Ave, Memphis, TN".into(),
        dropoff_window_start: "2026-04-08T15:00:00Z".into(),
        dropoff_window_end: Some("2026-04-08T18:00:00Z".into()),
        dropoff_instructions: Some("Call receiver before arrival".into()),
        dropoff_appointment_ref: Some("DO-IT-1001".into()),
        board_rate: 1140.0,
        rate_currency: Some("USD".into()),
        accessorial_flags: None,
        bid_type: "Fixed".into(),
        quote_status: None,
        tender_posture: None,
        compliance_passed: Some(true),
        compliance_summary: None,
        required_documents_status: None,
        readiness: Some("ready".into()),
        pushed_by: Some("integration@test".into()),
        push_reason: Some("Integration test publish".into()),
        source_module: Some("integration_test".into()),
        payload_version: Some("1.0".into()),
        external_refs: None,
    }
}

#[tokio::test]
#[serial]
async fn escrow_transition_updates_leg_status_and_history() -> Result<(), Box<dyn std::error::Error>>
{
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;
    let funded = apply_escrow_transition(
        &pool,
        EscrowTransitionParams {
            leg_id: fixture.leg_id,
            payer_user_id: fixture.shipper_user_id,
            payee_user_id: fixture.carrier_user_id,
            amount: 245000,
            platform_fee: 2500,
            currency: "USD",
            status: EscrowStatus::Funded,
            transfer_group: Some("tg_it_1001"),
            payment_intent_id: Some("pi_it_1001"),
            charge_id: Some("ch_it_1001"),
            transfer_id: None,
            stripe_refund_id: None,
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(fixture.shipper_user_id),
            note: Some("Funding from integration test"),
        },
    )
    .await?
    .expect("booked leg should exist");

    assert_eq!(funded.status, "funded");
    assert_eq!(funded.amount, 245000);
    assert_eq!(funded.transfer_group.as_deref(), Some("tg_it_1001"));

    let funded_leg_status: i16 =
        sqlx::query_scalar("SELECT status_id FROM load_legs WHERE id = $1")
            .bind(fixture.leg_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(funded_leg_status, 8);

    let released = apply_escrow_transition(
        &pool,
        EscrowTransitionParams {
            leg_id: fixture.leg_id,
            payer_user_id: fixture.shipper_user_id,
            payee_user_id: fixture.carrier_user_id,
            amount: 245000,
            platform_fee: 2500,
            currency: "USD",
            status: EscrowStatus::Released,
            transfer_group: Some("tg_it_1001"),
            payment_intent_id: Some("pi_it_1001"),
            charge_id: Some("ch_it_1001"),
            transfer_id: Some("tr_it_1001"),
            stripe_refund_id: None,
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(fixture.shipper_user_id),
            note: Some("Release from integration test"),
        },
    )
    .await?
    .expect("escrowed leg should exist");

    assert_eq!(released.status, "released");
    assert_eq!(released.transfer_id.as_deref(), Some("tr_it_1001"));

    let released_leg_status: i16 =
        sqlx::query_scalar("SELECT status_id FROM load_legs WHERE id = $1")
            .bind(fixture.leg_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(released_leg_status, 11);

    let history_rows = sqlx::query("SELECT status, remarks FROM load_history ORDER BY id")
        .fetch_all(&pool)
        .await?;
    assert_eq!(history_rows.len(), 2);
    assert_eq!(history_rows[0].get::<i16, _>("status"), 8);
    assert_eq!(history_rows[1].get::<i16, _>("status"), 11);

    let escrow = find_escrow_for_leg(&pool, fixture.leg_id)
        .await?
        .expect("escrow should exist");
    assert_eq!(escrow.status, "released");

    let ledger_entries = list_payment_ledger_entries_for_leg(&pool, fixture.leg_id).await?;
    assert_eq!(ledger_entries.len(), 3);
    assert_eq!(ledger_entries[0].entry_type, "escrow_funded");
    assert_eq!(ledger_entries[0].direction, "credit");
    assert_eq!(ledger_entries[0].amount_cents, 245000);
    assert!(ledger_entries[0].audit_event_id.is_some());
    assert_eq!(
        ledger_entries[0].payment_intent_id.as_deref(),
        Some("pi_it_1001")
    );
    assert_eq!(ledger_entries[1].entry_type, "carrier_transfer");
    assert_eq!(ledger_entries[1].direction, "debit");
    assert_eq!(ledger_entries[1].amount_cents, 242500);
    assert!(ledger_entries[1].audit_event_id.is_some());
    assert_eq!(ledger_entries[1].transfer_id.as_deref(), Some("tr_it_1001"));
    assert_eq!(ledger_entries[2].entry_type, "fee_earned");
    assert_eq!(ledger_entries[2].amount_cents, 2500);

    let package = find_invoice_settlement_for_leg(&pool, fixture.leg_id)
        .await?
        .expect("released escrow should generate invoice and settlement");
    assert_eq!(package.invoice.status, "issued");
    assert_eq!(package.invoice.total_amount_cents, 245000);
    assert_eq!(package.settlement.status, "released");
    assert_eq!(package.settlement.gross_amount_cents, 245000);
    assert_eq!(package.settlement.platform_fee_cents, 2500);
    assert_eq!(package.settlement.net_amount_cents, 242500);

    let refunded = apply_escrow_transition(
        &pool,
        EscrowTransitionParams {
            leg_id: fixture.leg_id,
            payer_user_id: fixture.shipper_user_id,
            payee_user_id: fixture.carrier_user_id,
            amount: 245000,
            platform_fee: 2500,
            currency: "USD",
            status: EscrowStatus::Refunded,
            transfer_group: Some("tg_it_1001"),
            payment_intent_id: Some("pi_it_1001"),
            charge_id: Some("ch_it_1001"),
            transfer_id: Some("tr_it_1001"),
            stripe_refund_id: Some("re_it_1001"),
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(fixture.shipper_user_id),
            note: Some("Refund from integration test"),
        },
    )
    .await?
    .expect("escrowed leg should allow refund status");
    assert_eq!(refunded.status, "refunded");

    record_payment_ledger_entry(
        &pool,
        CreatePaymentLedgerEntryParams {
            source_event_key: "it-adjustment-1001",
            entry_type: "adjustment",
            direction: "credit",
            currency: "USD",
            amount_cents: 1250,
            platform_fee_cents: 0,
            load_id: Some(fixture.load_id),
            leg_id: Some(fixture.leg_id),
            escrow_id: Some(refunded.id),
            payer_user_id: Some(fixture.shipper_user_id),
            payee_user_id: Some(fixture.carrier_user_id),
            actor_user_id: Some(fixture.shipper_user_id),
            audit_event_id: None,
            transfer_group: Some("tg_it_1001"),
            payment_intent_id: Some("pi_it_1001"),
            charge_id: Some("ch_it_1001"),
            transfer_id: Some("tr_it_1001"),
            stripe_refund_id: None,
            stripe_dispute_id: None,
            adjustment_reference: Some("ADJ-IT-1001"),
            description: Some("Integration adjustment"),
            metadata: serde_json::json!({"source": "integration_test"}),
        },
    )
    .await?;
    let adjusted_package = apply_invoice_settlement_adjustment(
        &pool,
        fixture.leg_id,
        1250,
        Some("Integration adjustment"),
    )
    .await?
    .expect("invoice and settlement should be adjustable");
    assert_eq!(adjusted_package.invoice.adjustment_amount_cents, 1250);
    assert_eq!(adjusted_package.invoice.total_amount_cents, 246250);
    assert_eq!(adjusted_package.settlement.adjustment_amount_cents, 1250);
    assert_eq!(adjusted_package.settlement.net_amount_cents, 243750);

    record_payment_ledger_entry(
        &pool,
        CreatePaymentLedgerEntryParams {
            source_event_key: "it-dispute-1001",
            entry_type: "dispute",
            direction: "hold",
            currency: "USD",
            amount_cents: 3000,
            platform_fee_cents: 0,
            load_id: Some(fixture.load_id),
            leg_id: Some(fixture.leg_id),
            escrow_id: Some(refunded.id),
            payer_user_id: Some(fixture.shipper_user_id),
            payee_user_id: Some(fixture.carrier_user_id),
            actor_user_id: Some(fixture.shipper_user_id),
            audit_event_id: None,
            transfer_group: Some("tg_it_1001"),
            payment_intent_id: Some("pi_it_1001"),
            charge_id: Some("ch_it_1001"),
            transfer_id: Some("tr_it_1001"),
            stripe_refund_id: None,
            stripe_dispute_id: Some("dp_it_1001"),
            adjustment_reference: None,
            description: Some("Integration dispute"),
            metadata: serde_json::json!({"source": "integration_test"}),
        },
    )
    .await?;

    let ledger_entries = list_payment_ledger_entries_for_leg(&pool, fixture.leg_id).await?;
    assert!(ledger_entries.iter().any(|entry| {
        entry.entry_type == "refund"
            && entry.stripe_refund_id.as_deref() == Some("re_it_1001")
            && entry.audit_event_id.is_some()
    }));
    assert!(ledger_entries.iter().any(|entry| {
        entry.entry_type == "adjustment"
            && entry.adjustment_reference.as_deref() == Some("ADJ-IT-1001")
    }));
    assert!(ledger_entries.iter().any(|entry| {
        entry.entry_type == "dispute" && entry.stripe_dispute_id.as_deref() == Some("dp_it_1001")
    }));

    Ok(())
}

#[tokio::test]
#[serial]
async fn tms_cancel_webhook_withdraws_local_projection() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let payload = sample_tms_payload("TMS-CANCEL-1001");
    let publish_result = push_handoff(&pool, &payload, Some("req_tms_test")).await?;
    let load_id = publish_result
        .load_id
        .expect("push should materialize a load");

    let webhook_result = apply_status_webhook(
        &pool,
        &TmsStatusWebhookRequest {
            event_id: Some("evt_tms_cancel_1001".into()),
            tms_load_id: payload.tms_load_id.clone(),
            tenant_id: payload.tenant_id.clone(),
            tms_status: "cancelled".into(),
            status_at: Some("2026-04-07T10:45:00Z".into()),
            source_module: Some("integration_test".into()),
            pushed_by: Some("tester".into()),
            detail: Some("Cancelled upstream".into()),
            rate_update: None,
        },
        Some("req_tms_webhook"),
    )
    .await?
    .expect("published handoff should accept a cancellation webhook");

    assert_eq!(webhook_result.action_label, "auto_withdraw");
    let updated_handoff = find_handoff_by_id(&pool, publish_result.handoff.id)
        .await?
        .expect("handoff should still exist");
    assert_eq!(updated_handoff.status, "withdrawn");
    assert!(updated_handoff.withdrawn_at.is_some());

    let handoff_request_id: Option<String> =
        sqlx::query_scalar("SELECT request_id FROM stloads_handoffs WHERE id = $1")
            .bind(publish_result.handoff.id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(handoff_request_id.as_deref(), Some("req_tms_test"));

    let load_deleted_at = sqlx::query("SELECT deleted_at FROM loads WHERE id = $1")
        .bind(load_id)
        .fetch_one(&pool)
        .await?
        .try_get::<Option<sqlx::types::chrono::NaiveDateTime>, _>("deleted_at")?;
    assert!(load_deleted_at.is_some());

    let reconcile_action: String = sqlx::query_scalar(
        "SELECT action FROM stloads_reconciliation_log WHERE handoff_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(publish_result.handoff.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(reconcile_action, "auto_withdraw");
    let reconcile_request_id: Option<String> = sqlx::query_scalar(
        "SELECT request_id FROM stloads_reconciliation_log WHERE handoff_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(publish_result.handoff.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(reconcile_request_id.as_deref(), Some("req_tms_webhook"));

    Ok(())
}

#[tokio::test]
#[serial]
async fn tms_rate_update_marks_requeue_required_and_updates_leg_price()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let payload = sample_tms_payload("TMS-RATE-1001");
    let publish_result = push_handoff(&pool, &payload, Some("req_tms_test")).await?;
    let load_id = publish_result
        .load_id
        .expect("push should materialize a load");

    let webhook_result = apply_status_webhook(
        &pool,
        &TmsStatusWebhookRequest {
            event_id: Some("evt_tms_rate_1001".into()),
            tms_load_id: payload.tms_load_id.clone(),
            tenant_id: payload.tenant_id.clone(),
            tms_status: "in_transit".into(),
            status_at: Some("2026-04-07T12:15:00Z".into()),
            source_module: Some("integration_test".into()),
            pushed_by: Some("tester".into()),
            detail: Some("Carrier still moving after rate revision".into()),
            rate_update: Some(1260.0),
        },
        Some("req_tms_webhook"),
    )
    .await?
    .expect("published handoff should accept a rate update webhook");

    assert_eq!(webhook_result.action_label, "rate_update");
    let updated_handoff = find_handoff_by_id(&pool, publish_result.handoff.id)
        .await?
        .expect("handoff should still exist");
    assert_eq!(updated_handoff.status, "requeue_required");
    assert_eq!(updated_handoff.board_rate, Some(1260.0));

    let leg_price: Option<f64> =
        sqlx::query_scalar("SELECT price FROM load_legs WHERE load_id = $1 LIMIT 1")
            .bind(load_id)
            .fetch_one(&pool)
            .await?;
    assert_eq!(leg_price, Some(1260.0));

    let reconcile_action: String = sqlx::query_scalar(
        "SELECT action FROM stloads_reconciliation_log WHERE handoff_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(publish_result.handoff.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(reconcile_action, "rate_update");

    Ok(())
}

#[tokio::test]
#[serial]
async fn external_event_claims_reject_duplicate_tms_webhooks()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let payload = serde_json::json!({
        "event_id": "evt_tms_status_duplicate",
        "tms_load_id": "TMS-DEDUPE-1001",
        "tenant_id": "tenant-it",
        "tms_status": "in_transit"
    });

    assert!(
        claim_external_event(
            &pool,
            None,
            "tms",
            "status",
            "evt_tms_status_duplicate",
            Some("req_event_first"),
            Some(&payload),
        )
        .await?
    );
    complete_external_event(
        &pool,
        None,
        "tms",
        "status",
        "evt_tms_status_duplicate",
        "status webhook applied",
    )
    .await?;
    assert!(
        !claim_external_event(
            &pool,
            None,
            "tms",
            "status",
            "evt_tms_status_duplicate",
            Some("req_event_duplicate"),
            Some(&payload),
        )
        .await?
    );

    let status: String = sqlx::query_scalar(
        "SELECT processing_status
         FROM external_event_dedupe_records
         WHERE source_system = 'tms' AND event_type = 'status' AND external_event_id = $1",
    )
    .bind("evt_tms_status_duplicate")
    .fetch_one(&pool)
    .await?;
    assert_eq!(status, "ignored_duplicate");

    Ok(())
}

#[tokio::test]
#[serial]
async fn webhook_delivery_logs_can_be_replayed() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let delivery = enqueue_webhook_delivery(
        &pool,
        EnqueueWebhookDeliveryParams {
            organization_id: DEFAULT_ORGANIZATION_ID,
            endpoint_id: None,
            event_type: "load.booked",
            event_id: "evt_load_booked_1104",
            request_payload: serde_json::json!({ "load_id": 42, "status": "booked" }),
            request_headers: serde_json::json!({ "x-stloads-event": "load.booked" }),
        },
    )
    .await?;
    assert_eq!(delivery.delivery_status, "queued");

    sqlx::query(
        "UPDATE webhook_delivery_logs
         SET delivery_status = 'dead_letter',
             attempt_count = 5,
             response_status_code = 500,
             response_latency_ms = 2450,
             response_body_excerpt = 'server error',
             dead_letter_reason = 'max attempts exceeded',
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(delivery.id)
    .execute(&pool)
    .await?;

    let failed_logs =
        list_webhook_delivery_logs(&pool, DEFAULT_ORGANIZATION_ID, Some("dead_letter"), 25).await?;
    assert_eq!(failed_logs.len(), 1);
    assert_eq!(failed_logs[0].response_status_code, Some(500));

    let replay = mark_webhook_delivery_for_replay(&pool, DEFAULT_ORGANIZATION_ID, delivery.id)
        .await?
        .expect("dead-letter delivery should be replayable");
    assert_eq!(replay.delivery_status, "replay_queued");
    assert_eq!(replay.replay_of_delivery_id, Some(delivery.id));

    Ok(())
}

#[tokio::test]
#[serial]
async fn email_outbox_records_retry_and_delivery_state() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let record = enqueue_email(
        &pool,
        EnqueueEmailParams {
            request_id: Some("req_email_test"),
            template_name: "integration_notice",
            to_email: "ops@example.test",
            to_name: Some("Ops User"),
            subject: "Integration notice",
            html_body: "<p>Hello from test</p>",
            max_attempts: 3,
        },
    )
    .await?;
    assert_eq!(record.request_id.as_deref(), Some("req_email_test"));

    assert_eq!(record.status, "processing");
    assert_eq!(record.attempts, 1);

    mark_email_retry(&pool, record.id, "temporary SMTP timeout").await?;
    let pending = count_pending_emails(&pool).await?;
    assert_eq!(pending, 1);

    let claimed = claim_due_emails(&pool, 10).await?;
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].id, record.id);
    assert_eq!(claimed[0].attempts, 2);

    mark_email_delivered(&pool, record.id, "sent").await?;
    let pending_after_delivery = count_pending_emails(&pool).await?;
    assert_eq!(pending_after_delivery, 0);

    Ok(())
}

#[tokio::test]
#[serial]
async fn master_data_crud_covers_location_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let country = master_data::upsert_country(&pool, None, "Testland", Some("TL")).await?;
    assert_eq!(country.iso_code.as_deref(), Some("TL"));

    let city = master_data::upsert_city(&pool, None, "Test City", country.id).await?;
    assert_eq!(city.country_id, country.id);

    let location =
        master_data::upsert_location(&pool, None, "Test Dock", Some(city.id), Some(country.id))
            .await?;
    assert_eq!(location.city_id, Some(city.id));

    let load_type = master_data::upsert_load_type(&pool, None, "Integration Freight").await?;
    let equipment = master_data::upsert_equipment(&pool, None, "Integration Trailer").await?;
    let commodity =
        master_data::upsert_commodity_type(&pool, None, "Integration Commodity").await?;

    assert_eq!(
        master_data::soft_delete_simple_catalog(&pool, "load_types", load_type.id).await?,
        1
    );
    assert_eq!(
        master_data::soft_delete_simple_catalog(&pool, "equipments", equipment.id).await?,
        1
    );
    assert_eq!(
        master_data::soft_delete_simple_catalog(&pool, "commodity_types", commodity.id).await?,
        1
    );
    assert_eq!(
        master_data::soft_delete_simple_catalog(&pool, "locations", location.id).await?,
        1
    );

    assert_eq!(master_data::delete_city(&pool, city.id).await?, 1);
    assert_eq!(master_data::delete_country(&pool, country.id).await?, 1);

    Ok(())
}

#[tokio::test]
#[serial]
async fn tms_reconciliation_scan_archives_and_flags_drift() -> Result<(), Box<dyn std::error::Error>>
{
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let terminal_payload = sample_tms_payload("TMS-SCAN-ARCHIVE-1001");
    let terminal = push_handoff(&pool, &terminal_payload, None).await?;
    sqlx::query(
        "UPDATE stloads_handoffs
         SET tms_status = 'invoiced',
             tms_status_at = CURRENT_TIMESTAMP,
             last_webhook_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(terminal.handoff.id)
    .execute(&pool)
    .await?;

    let stale_payload = sample_tms_payload("TMS-SCAN-STALE-1001");
    let stale = push_handoff(&pool, &stale_payload, None).await?;
    sqlx::query(
        "UPDATE stloads_handoffs
         SET published_at = CURRENT_TIMESTAMP - INTERVAL '40 days',
             last_webhook_at = NULL
         WHERE id = $1",
    )
    .bind(stale.handoff.id)
    .execute(&pool)
    .await?;

    let summary = run_reconciliation_scan(&pool, 30).await?;
    assert_eq!(summary.auto_archived, 1);
    assert_eq!(summary.stale_handoffs, 1);

    let archived = find_handoff_by_id(&pool, terminal.handoff.id)
        .await?
        .expect("terminal handoff should remain");
    assert_eq!(archived.status, "closed");

    let stale_errors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM stloads_sync_errors WHERE handoff_id = $1 AND error_class = 'stale_handoff' AND resolved = FALSE",
    )
    .bind(stale.handoff.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(stale_errors, 1);

    Ok(())
}

#[tokio::test]
#[serial]
async fn tms_conflict_queue_repairs_requeue_without_database_access()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let rules = list_tms_source_of_truth_rules(&pool).await?;
    assert!(rules.iter().any(|rule| {
        rule.field_key == "rate"
            && rule.owning_system == "stloads"
            && rule.default_repair_action == "requeue_tms_push"
    }));

    let payload = sample_tms_payload("TMS-CONFLICT-1001");
    let publish_result = push_handoff(&pool, &payload, Some("req_tms_conflict")).await?;
    let handoff_id = publish_result.handoff.id;

    let conflict = create_tms_conflict(
        &pool,
        CreateTmsConflictParams {
            handoff_id,
            field_key: "rate",
            stloads_value: Some("2450.00"),
            tms_value: Some("2600.00"),
            severity: "high",
            detected_by: "integration_test",
        },
    )
    .await?;
    assert_eq!(conflict.source_of_truth, "stloads");
    assert_eq!(conflict.repair_action, "requeue_tms_push");

    let open_conflicts = list_open_tms_conflicts(&pool, 25).await?;
    assert_eq!(open_conflicts.len(), 1);
    assert_eq!(open_conflicts[0].id, conflict.id);

    let repaired = repair_tms_conflict(
        &pool,
        conflict.id,
        "integration-test",
        Some("Requeue TMS push with STLoads-owned rate."),
    )
    .await?
    .expect("conflict should be repairable");
    assert_eq!(repaired.conflict_status, "repaired");

    let handoff = find_handoff_by_id(&pool, handoff_id)
        .await?
        .expect("handoff should exist");
    assert_eq!(handoff.status, "requeue_required");
    assert_eq!(
        handoff.last_push_result.as_deref(),
        Some("Queued by TMS conflict repair action.")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn tms_retry_worker_publishes_queued_handoff() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let payload = sample_tms_payload("TMS-RETRY-QUEUE-1001");
    db::tms::queue_handoff(&pool, &payload, None).await?;

    let summary = process_retryable_handoffs(&pool, 10, 5).await?;
    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.published, 1);
    assert_eq!(summary.failed, 0);

    let status: String = sqlx::query_scalar(
        "SELECT status FROM stloads_handoffs WHERE tms_load_id = $1 AND tenant_id = $2 LIMIT 1",
    )
    .bind(payload.tms_load_id)
    .bind(payload.tenant_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(status, "published");

    Ok(())
}

#[tokio::test]
#[serial]
async fn edi_integration_track_maps_validates_and_replays_messages()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let supported: Vec<String> = sqlx::query_scalar(
        "SELECT transaction_code
         FROM edi_transaction_mappings
         WHERE organization_id IS NULL
         ORDER BY transaction_code",
    )
    .fetch_all(&pool)
    .await?;
    for expected in ["204", "210", "214", "990", "997"] {
        assert!(supported.iter().any(|code| code == expected));
    }

    let partner_id: i64 = sqlx::query_scalar(
        "INSERT INTO edi_partner_profiles (
             organization_id, partner_name, isa_id, gs_id, transport_type, status,
             supported_transactions, validation_mode
         )
         VALUES ($1, 'Acme EDI', 'ACMEISA', 'ACMEGS', 'sftp', 'active',
                 ARRAY['204','990','214','210','997']::TEXT[], 'strict')
         ON CONFLICT (organization_id, partner_name)
         DO UPDATE SET
             status = EXCLUDED.status,
             supported_transactions = EXCLUDED.supported_transactions,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;

    let message_id: i64 = sqlx::query_scalar(
        "INSERT INTO edi_message_logs (
             organization_id, partner_profile_id, transaction_code, direction, control_number,
             business_key, message_status, ack_status, error_summary, payload_excerpt
         )
         VALUES ($1, $2, '204', 'inbound', 'CTRL-204-1001', 'LOAD-EDI-1001',
                 'failed', 'rejected_997', 'Missing pickup_stop', 'shipper_reference equipment rate_or_terms')
         ON CONFLICT (organization_id, transaction_code, direction, control_number)
         WHERE control_number IS NOT NULL
         DO UPDATE SET
             message_status = EXCLUDED.message_status,
             ack_status = EXCLUDED.ack_status,
             error_summary = EXCLUDED.error_summary
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(partner_id)
    .fetch_one(&pool)
    .await?;

    let replay_id: i64 = sqlx::query_scalar(
        "INSERT INTO edi_message_logs (
             organization_id, partner_profile_id, transaction_code, direction, control_number,
             business_key, message_status, ack_status, retry_count, payload_excerpt,
             replay_of_message_id
         )
         SELECT organization_id, partner_profile_id, transaction_code, direction, NULL,
                business_key, 'replay_queued', 'pending_997', 0, payload_excerpt, id
         FROM edi_message_logs
         WHERE id = $1 AND organization_id = $2
         RETURNING id",
    )
    .bind(message_id)
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;

    let replay = sqlx::query(
        "SELECT message_status, ack_status, replay_of_message_id
         FROM edi_message_logs
         WHERE id = $1",
    )
    .bind(replay_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(replay.get::<String, _>("message_status"), "replay_queued");
    assert_eq!(replay.get::<String, _>("ack_status"), "pending_997");
    assert_eq!(
        replay.get::<Option<i64>, _>("replay_of_message_id"),
        Some(message_id)
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn sandbox_and_api_lifecycle_governance_blocks_production_side_effects()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let sandbox_id: i64 = sqlx::query_scalar(
        "INSERT INTO sandbox_tenant_environments (
             organization_id, environment_key, display_name, base_url, data_classification,
             pii_allowed, production_payment_blocked, production_tms_push_blocked,
             production_notification_blocked, seeded_dataset_version, reset_status
         )
         VALUES (
             $1, 'integration-test-sandbox', 'Integration Test Sandbox',
             'https://sandbox-api.stloads.com', 'synthetic',
             FALSE, TRUE, TRUE, TRUE, 'demo-v1', 'ready'
         )
         ON CONFLICT (organization_id, environment_key)
         DO UPDATE SET
             pii_allowed = FALSE,
             production_payment_blocked = TRUE,
             production_tms_push_blocked = TRUE,
             production_notification_blocked = TRUE,
             reset_status = 'ready'
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;

    let safety = sqlx::query(
        "SELECT pii_allowed, production_payment_blocked, production_tms_push_blocked,
                production_notification_blocked
         FROM sandbox_tenant_environments
         WHERE id = $1",
    )
    .bind(sandbox_id)
    .fetch_one(&pool)
    .await?;
    assert!(!safety.get::<bool, _>("pii_allowed"));
    assert!(safety.get::<bool, _>("production_payment_blocked"));
    assert!(safety.get::<bool, _>("production_tms_push_blocked"));
    assert!(safety.get::<bool, _>("production_notification_blocked"));

    let reset_job_id: i64 = sqlx::query_scalar(
        "INSERT INTO sandbox_reset_jobs (
             organization_id, sandbox_environment_id, reset_reason, job_status, safety_checks
         )
         VALUES (
             $1, $2, 'integration test reset', 'queued',
             '{\"pii_allowed\":false,\"production_payment_blocked\":true,\"production_tms_push_blocked\":true,\"production_notification_blocked\":true}'::JSONB
         )
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(sandbox_id)
    .fetch_one(&pool)
    .await?;
    assert!(reset_job_id > 0);

    let policy = sqlx::query(
        "SELECT release_status, minimum_notice_days, compatibility_test_status,
                postman_collection_url
         FROM api_lifecycle_policies
         WHERE api_version = '2026-05-26'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(policy.get::<String, _>("release_status"), "active");
    assert!(policy.get::<i32, _>("minimum_notice_days") >= 180);
    assert_eq!(
        policy.get::<String, _>("compatibility_test_status"),
        "passing"
    );
    assert_eq!(
        policy
            .get::<Option<String>, _>("postman_collection_url")
            .as_deref(),
        Some("/docs/STLOADS_POSTMAN_COLLECTION.json")
    );

    let runnable_examples: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM api_partner_examples
         WHERE api_version = '2026-05-26' AND sandbox_runnable = TRUE",
    )
    .fetch_one(&pool)
    .await?;
    assert!(runnable_examples >= 4);

    Ok(())
}

#[tokio::test]
#[serial]
async fn notification_center_preferences_and_coverage_are_auditable()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;
    let required_events = [
        "booking.created",
        "offer.countered",
        "tender.response_required",
        "tracking.stale",
        "pickup.completed",
        "delivery.completed",
        "pod.missing",
        "payment.hold",
        "payment.released",
        "compliance.expiring",
        "tms.drift_detected",
        "document.rejected",
    ];
    let coverage_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM notification_coverage_rules
         WHERE event_key = ANY($1) AND active = TRUE",
    )
    .bind(required_events.as_slice())
    .fetch_one(&pool)
    .await?;
    assert_eq!(coverage_count, required_events.len() as i64);

    let provider_decisions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM notification_provider_decisions
         WHERE channel IN ('in_app', 'email', 'sms', 'push')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(provider_decisions, 4);

    let notification_id: i64 = sqlx::query_scalar(
        "INSERT INTO notification_events (
             organization_id, recipient_user_id, event_key, category, priority,
             subject, body, entity_type, entity_id, action_href, channels, delivery_status
         )
         VALUES ($1, $2, 'pod.missing', 'documents', 'urgent',
                 'POD missing for test load', 'Upload POD before closeout and payment release.',
                 'load_leg', $3, $4, ARRAY['in_app','email']::TEXT[], 'delivered')
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(fixture.carrier_user_id)
    .bind(fixture.leg_id)
    .bind(format!("/execution/legs/{}", fixture.leg_id))
    .fetch_one(&pool)
    .await?;
    assert!(notification_id > 0);

    sqlx::query(
        "INSERT INTO notification_preferences (
             organization_id, user_id, event_key, email_enabled, in_app_enabled,
             sms_enabled, push_enabled, quiet_hours_start, quiet_hours_end, timezone,
             escalation_minutes
         )
         VALUES ($1, $2, 'pod.missing', TRUE, TRUE, FALSE, FALSE, '22:00'::TIME, '06:00'::TIME,
                 'America/Chicago', 30)
         ON CONFLICT (COALESCE(organization_id, 0), COALESCE(user_id, 0), event_key)
         DO UPDATE SET
             email_enabled = EXCLUDED.email_enabled,
             in_app_enabled = EXCLUDED.in_app_enabled,
             quiet_hours_start = EXCLUDED.quiet_hours_start,
             quiet_hours_end = EXCLUDED.quiet_hours_end,
             timezone = EXCLUDED.timezone,
             escalation_minutes = EXCLUDED.escalation_minutes",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(fixture.carrier_user_id)
    .execute(&pool)
    .await?;

    let unread: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM notification_events
         WHERE recipient_user_id = $1 AND read_at IS NULL",
    )
    .bind(fixture.carrier_user_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(unread, 1);

    sqlx::query(
        "UPDATE notification_events
         SET read_at = CURRENT_TIMESTAMP
         WHERE id = $1 AND recipient_user_id = $2",
    )
    .bind(notification_id)
    .bind(fixture.carrier_user_id)
    .execute(&pool)
    .await?;

    let read_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT read_at FROM notification_events WHERE id = $1")
            .bind(notification_id)
            .fetch_one(&pool)
            .await?;
    assert!(read_at.is_some());

    Ok(())
}

#[tokio::test]
#[serial]
async fn message_deliverability_and_branding_governance_are_controlled()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let fixture = insert_booked_leg_fixture(&pool).await?;

    let sender_identity_id: i64 = sqlx::query_scalar(
        "INSERT INTO message_sender_identities (
             organization_id, environment_key, sender_domain, from_email, from_name,
             spf_status, dkim_status, dmarc_status, identity_status, verified_at, notes
         )
         VALUES ($1, 'staging', 'broker.example.com', 'dispatch@broker.example.com',
                 'Broker Dispatch', 'verified', 'verified', 'verified', 'verified',
                 CURRENT_TIMESTAMP, 'Tenant sender verified for staging test traffic.')
         ON CONFLICT (COALESCE(organization_id, 0), environment_key, from_email)
         DO UPDATE SET
             spf_status = EXCLUDED.spf_status,
             dkim_status = EXCLUDED.dkim_status,
             dmarc_status = EXCLUDED.dmarc_status,
             identity_status = EXCLUDED.identity_status,
             verified_at = EXCLUDED.verified_at
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;
    assert!(sender_identity_id > 0);

    let high_risk_rules: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM message_monitoring_rules
         WHERE rule_key = ANY($1) AND active = TRUE",
    )
    .bind([
        "otp.delivery",
        "password_reset.delivery",
        "tender.delivery",
        "pickup.delivery",
        "delivery.delivery",
        "pod_rejection.delivery",
        "payment_hold.delivery",
        "payout_release.delivery",
    ])
    .fetch_one(&pool)
    .await?;
    assert_eq!(high_risk_rules, 8);

    let governed_templates: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM message_template_governance
         WHERE high_risk = TRUE
           AND test_send_required = TRUE
           AND approval_status = 'pending_approval'",
    )
    .fetch_one(&pool)
    .await?;
    assert!(governed_templates >= 6);

    let outbox_id: i64 = sqlx::query_scalar(
        "INSERT INTO email_outbox (template_name, to_email, to_name, subject, html_body)
         VALUES ('payment.hold', 'carrier@example.com', 'Carrier Ops',
                 'Payment hold requires review', '<p>Payment hold</p>')
         RETURNING id",
    )
    .fetch_one(&pool)
    .await?;

    let bounce_event_id: i64 = sqlx::query_scalar(
        "INSERT INTO message_delivery_events (
             organization_id, email_outbox_id, channel, event_type, provider_message_id,
             recipient, reason, metadata
         )
         VALUES ($1, $2, 'email', 'bounce', 'provider-message-1',
                 'carrier@example.com', 'mailbox unavailable',
                 jsonb_build_object('template_key', 'payment.hold'))
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(outbox_id)
    .fetch_one(&pool)
    .await?;
    assert!(bounce_event_id > 0);

    sqlx::query(
        "INSERT INTO message_suppression_entries (
             organization_id, channel, recipient, suppression_reason, source_event_id, status
         )
         VALUES ($1, 'email', 'carrier@example.com', 'hard_bounce', $2, 'active')
         ON CONFLICT (COALESCE(organization_id, 0), channel, lower(recipient), status)
         DO UPDATE SET
             suppression_reason = EXCLUDED.suppression_reason,
             source_event_id = EXCLUDED.source_event_id,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(bounce_event_id)
    .execute(&pool)
    .await?;

    let active_suppressions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM message_suppression_entries
         WHERE organization_id = $1 AND recipient = 'carrier@example.com' AND status = 'active'",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;
    assert_eq!(active_suppressions, 1);

    sqlx::query(
        "INSERT INTO tenant_branding_policies (
             organization_id, portal_branding_enabled, document_branding_enabled,
             email_branding_enabled, custom_domain_enabled, white_label_status,
             unsupported_message, fallback_brand_name, cache_version
         )
         VALUES ($1, TRUE, TRUE, FALSE, FALSE, 'deferred',
                 'Email branding and custom domains require product approval.',
                 'Broker Logistics', 2)
         ON CONFLICT (organization_id)
         DO UPDATE SET
             portal_branding_enabled = EXCLUDED.portal_branding_enabled,
             document_branding_enabled = EXCLUDED.document_branding_enabled,
             email_branding_enabled = EXCLUDED.email_branding_enabled,
             custom_domain_enabled = EXCLUDED.custom_domain_enabled,
             white_label_status = EXCLUDED.white_label_status,
             unsupported_message = EXCLUDED.unsupported_message,
             fallback_brand_name = EXCLUDED.fallback_brand_name,
             cache_version = EXCLUDED.cache_version,
             updated_at = CURRENT_TIMESTAMP",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .execute(&pool)
    .await?;

    let asset_id: i64 = sqlx::query_scalar(
        "INSERT INTO tenant_brand_assets (
             organization_id, asset_type, asset_url, mime_type, file_size_bytes,
             width_px, height_px, review_status, reviewer_user_id, cache_key, notes
         )
         VALUES ($1, 'logo', 's3://tenant-assets/logo.png', 'image/png', 51200,
                 600, 160, 'approved', $2, 'brand-v2-logo', 'Approved logo within size and type constraints.')
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .bind(fixture.shipper_user_id)
    .fetch_one(&pool)
    .await?;
    assert!(asset_id > 0);

    let custom_domain_id: i64 = sqlx::query_scalar(
        "INSERT INTO tenant_custom_domains (
             organization_id, domain, purpose, verification_status, dns_txt_name,
             dns_txt_value, tls_status, rollback_status, notes
         )
         VALUES ($1, 'portal.broker.example.com', 'portal', 'pending',
                 '_stloads.portal.broker.example.com', 'stloads-domain-verification-test',
                 'not_requested', 'ready', 'Pending DNS ownership validation.')
         ON CONFLICT (organization_id, lower(domain), purpose)
         DO UPDATE SET
             verification_status = EXCLUDED.verification_status,
             dns_txt_name = EXCLUDED.dns_txt_name,
             dns_txt_value = EXCLUDED.dns_txt_value,
             tls_status = EXCLUDED.tls_status,
             rollback_status = EXCLUDED.rollback_status,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;
    assert!(custom_domain_id > 0);

    for (surface, key) in [
        ("rate_confirmation", "rate_confirmation.default"),
        ("bol", "bol.default"),
        ("pod_package", "pod_package.default"),
        ("invoice", "invoice.default"),
        ("settlement_packet", "settlement_packet.default"),
        ("notification_email", "payment.hold"),
    ] {
        sqlx::query(
            "INSERT INTO tenant_branded_template_rules (
                 organization_id, template_key, template_surface, branding_status,
                 fallback_allowed, notes
             )
             VALUES ($1, $2, $3, 'fallback', TRUE, 'Tenant-specific branding awaits product approval.')
             ON CONFLICT (organization_id, template_key, template_surface)
             DO UPDATE SET
                 branding_status = EXCLUDED.branding_status,
                 fallback_allowed = EXCLUDED.fallback_allowed,
                 notes = EXCLUDED.notes,
                 updated_at = CURRENT_TIMESTAMP",
        )
        .bind(DEFAULT_ORGANIZATION_ID)
        .bind(key)
        .bind(surface)
        .execute(&pool)
        .await?;
    }

    let fallback_template_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM tenant_branded_template_rules
         WHERE organization_id = $1 AND fallback_allowed = TRUE",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;
    assert_eq!(fallback_template_count, 6);

    let unsupported_message: String = sqlx::query_scalar(
        "SELECT unsupported_message
         FROM tenant_branding_policies
         WHERE organization_id = $1",
    )
    .bind(DEFAULT_ORGANIZATION_ID)
    .fetch_one(&pool)
    .await?;
    assert!(unsupported_message.contains("product approval"));

    Ok(())
}
