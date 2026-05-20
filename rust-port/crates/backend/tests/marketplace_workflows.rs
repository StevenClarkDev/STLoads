use serial_test::serial;

#[tokio::test]
#[serial(marketplace_workflows_db)]
async fn offers_tenders_book_now_and_cancellations_emit_events_and_lock_booking() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let fixture = seed_marketplace_fixture(&pool, "tenant-p10", "p10").await;

    let offer = db::marketplace::submit_carrier_offer(
        &pool,
        db::marketplace::SubmitCarrierOfferInput {
            tenant_id: "tenant-p10",
            posting_id: fixture.posting_id,
            carrier_profile_id: fixture.carrier_one_profile_id,
            carrier_user_id: fixture.carrier_one_user_id,
            amount: 2450.0,
            currency: "USD",
            message: Some("Carrier can cover this lane today."),
            idempotency_key: Some("offer-p10-1"),
            created_by: fixture.carrier_one_user_id,
        },
    )
    .await
    .unwrap();
    assert_eq!(offer.status_id, 1);
    assert_eq!(offer.version_count, 1);

    let counter = db::marketplace::create_counteroffer(
        &pool,
        db::marketplace::CreateCounterofferInput {
            tenant_id: "tenant-p10",
            offer_id: offer.id,
            from_party_type: "shipper",
            to_party_type: "carrier",
            amount: 2525.0,
            currency: "USD",
            message: Some("Can you include detention terms?"),
            created_by: fixture.shipper_user_id,
        },
    )
    .await
    .unwrap();
    assert_eq!(counter.status, "pending");

    let accepted_counter = db::marketplace::respond_to_counteroffer(
        &pool,
        "tenant-p10",
        counter.id,
        "accept",
        Some("Accepted with detention terms."),
        fixture.carrier_one_user_id,
    )
    .await
    .unwrap();
    assert_eq!(accepted_counter.status, "accepted");

    for (party, amount) in [("broker", 2535.0), ("forwarder", 2545.0)] {
        let counter = db::marketplace::create_counteroffer(
            &pool,
            db::marketplace::CreateCounterofferInput {
                tenant_id: "tenant-p10",
                offer_id: offer.id,
                from_party_type: party,
                to_party_type: "carrier",
                amount,
                currency: "USD",
                message: Some("Alternate commercial desk counter."),
                created_by: fixture.shipper_user_id,
            },
        )
        .await
        .unwrap();
        assert_eq!(counter.status, "pending");

        let rejected_counter = db::marketplace::respond_to_counteroffer(
            &pool,
            "tenant-p10",
            counter.id,
            "reject",
            Some("Rejected to preserve the accepted shipper terms."),
            fixture.carrier_one_user_id,
        )
        .await
        .unwrap();
        assert_eq!(rejected_counter.status, "rejected");
    }

    let counter_parties = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT from_party_type FROM counteroffers WHERE tenant_id = 'tenant-p10' AND offer_id = $1 ORDER BY from_party_type",
    )
    .bind(offer.id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(counter_parties, vec!["broker", "forwarder", "shipper"]);

    let offer_versions = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM offer_versions WHERE tenant_id = 'tenant-p10' AND offer_id = $1",
    )
    .bind(offer.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(offer_versions, 2);

    let tender = db::marketplace::create_tender_invite(
        &pool,
        db::marketplace::CreateTenderInviteInput {
            tenant_id: "tenant-p10",
            posting_id: fixture.posting_id,
            carrier_profile_id: fixture.carrier_one_profile_id,
            tender_type: "direct",
            expires_minutes: Some(90),
            created_by: fixture.shipper_user_id,
        },
    )
    .await
    .unwrap();
    assert_eq!(tender.invite_status, "sent");

    let tender_response = db::marketplace::respond_to_tender_invite(
        &pool,
        "tenant-p10",
        tender.invite_id,
        "accept",
        Some("Tender accepted."),
        fixture.carrier_one_user_id,
    )
    .await
    .unwrap();
    assert_eq!(tender_response.invite_status, "accepted");

    let carrier_one_book = db::marketplace::book_now_posting(
        &pool,
        db::marketplace::BookNowInput {
            tenant_id: "tenant-p10",
            posting_id: fixture.posting_id,
            carrier_profile_id: fixture.carrier_one_profile_id,
            carrier_user_id: fixture.carrier_one_user_id,
            offer_id: Some(offer.id),
            tender_id: Some(tender.tender_id),
            amount: Some(2525.0),
            currency: "USD",
            terms_accepted: true,
            idempotency_key: Some("book-p10-carrier-one"),
            created_by: fixture.carrier_one_user_id,
        },
    );
    let carrier_two_book = db::marketplace::book_now_posting(
        &pool,
        db::marketplace::BookNowInput {
            tenant_id: "tenant-p10",
            posting_id: fixture.posting_id,
            carrier_profile_id: fixture.carrier_two_profile_id,
            carrier_user_id: fixture.carrier_two_user_id,
            offer_id: None,
            tender_id: None,
            amount: Some(2500.0),
            currency: "USD",
            terms_accepted: true,
            idempotency_key: Some("book-p10-carrier-two"),
            created_by: fixture.carrier_two_user_id,
        },
    );
    let (first, second) = tokio::join!(carrier_one_book, carrier_two_book);
    let successes = [first.as_ref().ok(), second.as_ref().ok()]
        .into_iter()
        .flatten()
        .count();
    let failures = [first.as_ref().err(), second.as_ref().err()]
        .into_iter()
        .flatten()
        .count();
    assert_eq!(successes, 1);
    assert_eq!(failures, 1);

    let award = first.or(second).unwrap();
    let active_awards = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM booking_awards WHERE tenant_id = 'tenant-p10' AND posting_id = $1 AND status IN ('awarded', 'accepted', 'in_transit')",
    )
    .bind(fixture.posting_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_awards, 1);

    let booking_lock_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM booking_locks WHERE tenant_id = 'tenant-p10' AND posting_id = $1 AND released_at IS NULL",
    )
    .bind(fixture.posting_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(booking_lock_count, 1);

    let booked_leg_history = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM load_history WHERE remarks = 'STLoads book-now awarded this posting.'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(booked_leg_history >= 1);

    let cancellation = db::marketplace::request_carrier_cancellation(
        &pool,
        db::marketplace::CarrierCancellationInput {
            tenant_id: "tenant-p10",
            posting_id: fixture.posting_id,
            booking_award_id: Some(award.id),
            requested_by: fixture.carrier_one_user_id,
            reason_code: "carrier_request",
            reason_detail: Some("Driver issue, requesting operator review."),
        },
    )
    .await
    .unwrap();
    assert_eq!(cancellation.status, "pending");

    let event_types = sqlx::query_scalar::<_, String>(
        "SELECT event_type FROM atmp_outbound_events WHERE tenant_id = 'tenant-p10' ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(event_types.contains(&"offer_submitted".to_string()));
    assert!(event_types.contains(&"counteroffer_submitted".to_string()));
    assert!(event_types.contains(&"counteroffer_accepted".to_string()));
    assert!(event_types.contains(&"counteroffer_rejected".to_string()));
    assert!(event_types.contains(&"tender_sent".to_string()));
    assert!(event_types.contains(&"tender_accepted".to_string()));
    if award.offer_id.is_some() {
        assert!(event_types.contains(&"offer_accepted".to_string()));
    }
    assert!(event_types.contains(&"carrier_booked".to_string()));
    assert!(event_types.contains(&"booking_canceled".to_string()));

    let transition_event_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM atmp_outbound_events WHERE tenant_id = 'tenant-p10' AND posting_id = $1",
    )
    .bind(fixture.posting_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(transition_event_count >= 10);
}

#[tokio::test]
#[serial(marketplace_workflows_db)]
async fn chat_notifications_and_marketplace_messages_are_scoped_and_retryable() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let fixture = seed_marketplace_fixture(&pool, "tenant-p11", "p11").await;
    let offer = db::marketplace::submit_carrier_offer(
        &pool,
        db::marketplace::SubmitCarrierOfferInput {
            tenant_id: "tenant-p11",
            posting_id: fixture.posting_id,
            carrier_profile_id: fixture.carrier_one_profile_id,
            carrier_user_id: fixture.carrier_one_user_id,
            amount: 2310.0,
            currency: "USD",
            message: Some("P11 carrier offer."),
            idempotency_key: Some("offer-p11-1"),
            created_by: fixture.carrier_one_user_id,
        },
    )
    .await
    .unwrap();
    let conversation_id = offer.conversation_id.unwrap();

    let scope = db::marketplace::find_authorized_conversation_scope(
        &pool,
        "tenant-p11",
        conversation_id,
        fixture.shipper_user_id,
        false,
    )
    .await
    .unwrap()
    .expect("shipper should be an authorized conversation participant");
    assert_eq!(scope.posting_id, Some(fixture.posting_id));
    assert_eq!(scope.offer_id, Some(offer.id));

    let outsider = backend::test_support::insert_user_with_role_status(
        &pool,
        "P11 Outsider",
        "outsider-p11@example.com",
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    assert!(
        db::marketplace::find_authorized_conversation_scope(
            &pool,
            "tenant-p11",
            conversation_id,
            outsider.id,
            false,
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(
        db::marketplace::find_authorized_conversation_scope(
            &pool,
            "wrong-tenant",
            conversation_id,
            fixture.shipper_user_id,
            false,
        )
        .await
        .unwrap()
        .is_none()
    );

    let system_message = db::marketplace::create_marketplace_system_message(
        &pool,
        db::marketplace::MarketplaceSystemMessageInput {
            tenant_id: "tenant-p11",
            conversation_id,
            actor_user_id: fixture.shipper_user_id,
            event_type: "sync_failure",
            reference_type: Some("posting"),
            reference_id: Some(fixture.posting_id),
            body: "ATMP sync failed for this posting and requires operator review.",
        },
    )
    .await
    .unwrap()
    .expect("system message should be inserted");
    let message_meta =
        sqlx::query_scalar::<_, serde_json::Value>("SELECT meta FROM messages WHERE id = $1")
            .bind(system_message.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(message_meta["system"], true);
    assert_eq!(message_meta["event_type"], "sync_failure");
    assert_eq!(message_meta["reference_type"], "posting");

    let preference = db::marketplace::upsert_marketplace_notification_preference(
        &pool,
        "tenant-p11",
        fixture.carrier_one_user_id,
        false,
        true,
        true,
    )
    .await
    .unwrap();
    assert!(!preference.email_enabled);
    assert!(preference.critical_email_enabled);

    let queued = db::email_outbox::enqueue_critical_marketplace_email(
        &pool,
        db::email_outbox::CriticalMarketplaceEmailParams {
            template_name: "sync_failure",
            to_email: "ops-p11@example.com",
            to_name: Some("Ops P11"),
            subject: "Critical STLoads marketplace sync failure",
            html_body: "<p>Sync failed.</p>",
            max_attempts: 12,
        },
    )
    .await
    .unwrap();
    assert_eq!(queued.status, "pending");
    assert_eq!(queued.max_attempts, 12);

    db::email_outbox::mark_email_retry(&pool, queued.id, "transient smtp outage")
        .await
        .unwrap();
    let retry_record = sqlx::query_as::<_, db::email_outbox::EmailOutboxRecord>(
        "SELECT * FROM email_outbox WHERE id = $1",
    )
    .bind(queued.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(retry_record.status, "retry");
    assert_eq!(
        retry_record.last_error.as_deref(),
        Some("transient smtp outage")
    );
}

struct MarketplaceFixture {
    shipper_user_id: i64,
    carrier_one_user_id: i64,
    carrier_two_user_id: i64,
    carrier_one_profile_id: i64,
    carrier_two_profile_id: i64,
    posting_id: i64,
}

async fn seed_marketplace_fixture(
    pool: &db::DbPool,
    tenant_id: &str,
    suffix: &str,
) -> MarketplaceFixture {
    seed_tenant(pool, tenant_id).await;
    let shipper = backend::test_support::insert_user_with_role_status(
        pool,
        &format!("{} Shipper", suffix.to_uppercase()),
        &format!("shipper-{suffix}@example.com"),
        domain::auth::UserRole::Shipper,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let carrier_one = backend::test_support::insert_user_with_role_status(
        pool,
        &format!("{} Carrier One", suffix.to_uppercase()),
        &format!("carrier-one-{suffix}@example.com"),
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let carrier_two = backend::test_support::insert_user_with_role_status(
        pool,
        &format!("{} Carrier Two", suffix.to_uppercase()),
        &format!("carrier-two-{suffix}@example.com"),
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();

    let load = backend::test_support::insert_load_fixture(pool, 1)
        .await
        .unwrap();
    sqlx::query("UPDATE loads SET user_id = $1 WHERE id = $2")
        .bind(shipper.id)
        .bind(load.load_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE load_legs SET booked_carrier_id = NULL, booked_at = NULL, booked_amount = NULL, accepted_offer_id = NULL, status_id = 2 WHERE id = $1")
        .bind(load.leg_id)
        .execute(pool)
        .await
        .unwrap();

    let posting_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_postings
            (tenant_id, source_system, source_load_id, source_leg_id, posting_number, title,
             freight_mode, equipment_type, status, visibility, readiness, published_at, created_by,
             created_at, updated_at)
         VALUES ($1, 'test', $2, $3, $4, $5, 'road', 'Dry Van',
             'published', 'public', 'ready', CURRENT_TIMESTAMP, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(load.load_id.to_string())
    .bind(load.leg_id.to_string())
    .bind(format!("{}-POSTING", suffix.to_uppercase()))
    .bind(format!("{} lane", suffix.to_uppercase()))
    .bind(shipper.id)
    .fetch_one(pool)
    .await
    .unwrap();

    for (sequence, stop_type, city, state) in [
        (1_i32, "pickup", "Dallas", "TX"),
        (2_i32, "delivery", "Memphis", "TN"),
    ] {
        sqlx::query(
            "INSERT INTO stloads_posting_stops
                (tenant_id, posting_id, stop_sequence, stop_type, city, state_region, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(tenant_id)
        .bind(posting_id)
        .bind(sequence)
        .bind(stop_type)
        .bind(city)
        .bind(state)
        .execute(pool)
        .await
        .unwrap();
    }

    let carrier_one_profile_id =
        seed_eligible_carrier(pool, tenant_id, carrier_one.id, "one").await;
    let carrier_two_profile_id =
        seed_eligible_carrier(pool, tenant_id, carrier_two.id, "two").await;

    MarketplaceFixture {
        shipper_user_id: shipper.id,
        carrier_one_user_id: carrier_one.id,
        carrier_two_user_id: carrier_two.id,
        carrier_one_profile_id,
        carrier_two_profile_id,
        posting_id,
    }
}

async fn seed_eligible_carrier(
    pool: &db::DbPool,
    tenant_id: &str,
    user_id: i64,
    suffix: &str,
) -> i64 {
    let carrier_profile_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO carrier_profiles
            (tenant_id, user_id, display_name, carrier_type, dot_number, mc_number,
             insurance_status, authority_status, compliance_status, risk_score, status,
             created_at, updated_at)
         VALUES ($1, $2, $3, 'motor_carrier', $4, $5,
             'approved', 'verified', 'approved', 1.0, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(format!("Carrier {suffix}"))
    .bind(format!("DOT-{tenant_id}-{suffix}"))
    .bind(format!("MC-{tenant_id}-{suffix}"))
    .fetch_one(pool)
    .await
    .unwrap();

    for document_type in [
        "carrier_packet",
        "w9",
        "insurance_certificate",
        "authority_verification",
    ] {
        sqlx::query(
            "INSERT INTO compliance_documents
                (tenant_id, carrier_profile_id, document_type, document_name, storage_key, status,
                 expires_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'approved',
                 CURRENT_TIMESTAMP + INTERVAL '30 days', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(tenant_id)
        .bind(carrier_profile_id)
        .bind(document_type)
        .bind(format!("{document_type} {suffix}"))
        .bind(format!("{tenant_id}/{suffix}/{document_type}.pdf"))
        .execute(pool)
        .await
        .unwrap();
    }

    sqlx::query(
        "INSERT INTO carrier_equipment
            (tenant_id, carrier_profile_id, equipment_type, quantity, status, created_at, updated_at)
         VALUES ($1, $2, 'Dry Van', 1, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO carrier_lanes
            (tenant_id, carrier_profile_id, origin_state, destination_state, status, created_at, updated_at)
         VALUES ($1, $2, 'TX', 'TN', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(carrier_profile_id)
    .execute(pool)
    .await
    .unwrap();

    carrier_profile_id
}

async fn seed_tenant(pool: &db::DbPool, tenant_id: &str) {
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at, updated_at)
         VALUES ($1, $2, $3, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(format!("Tenant {tenant_id}"))
    .bind(tenant_id)
    .execute(pool)
    .await
    .unwrap();
}
