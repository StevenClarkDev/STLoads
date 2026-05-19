use serial_test::serial;

#[tokio::test]
#[serial(marketplace_workflows_db)]
async fn offers_tenders_book_now_and_cancellations_emit_events_and_lock_booking() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let fixture = seed_marketplace_fixture(&pool).await;

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
    assert!(event_types.contains(&"offer_accepted".to_string()));
    assert!(event_types.contains(&"carrier_booked".to_string()));
    assert!(event_types.contains(&"booking_canceled".to_string()));
}

struct MarketplaceFixture {
    shipper_user_id: i64,
    carrier_one_user_id: i64,
    carrier_two_user_id: i64,
    carrier_one_profile_id: i64,
    carrier_two_profile_id: i64,
    posting_id: i64,
}

async fn seed_marketplace_fixture(pool: &db::DbPool) -> MarketplaceFixture {
    seed_tenant(pool, "tenant-p10").await;
    let shipper = backend::test_support::insert_user_with_role_status(
        pool,
        "P10 Shipper",
        "shipper-p10@example.com",
        domain::auth::UserRole::Shipper,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let carrier_one = backend::test_support::insert_user_with_role_status(
        pool,
        "P10 Carrier One",
        "carrier-one-p10@example.com",
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let carrier_two = backend::test_support::insert_user_with_role_status(
        pool,
        "P10 Carrier Two",
        "carrier-two-p10@example.com",
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
         VALUES ('tenant-p10', 'test', $1, $2, 'P10-POSTING', 'P10 lane', 'road', 'Dry Van',
             'published', 'public', 'ready', CURRENT_TIMESTAMP, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(load.load_id.to_string())
    .bind(load.leg_id.to_string())
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
             VALUES ('tenant-p10', $1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(posting_id)
        .bind(sequence)
        .bind(stop_type)
        .bind(city)
        .bind(state)
        .execute(pool)
        .await
        .unwrap();
    }

    let carrier_one_profile_id = seed_eligible_carrier(pool, carrier_one.id, "one").await;
    let carrier_two_profile_id = seed_eligible_carrier(pool, carrier_two.id, "two").await;

    MarketplaceFixture {
        shipper_user_id: shipper.id,
        carrier_one_user_id: carrier_one.id,
        carrier_two_user_id: carrier_two.id,
        carrier_one_profile_id,
        carrier_two_profile_id,
        posting_id,
    }
}

async fn seed_eligible_carrier(pool: &db::DbPool, user_id: i64, suffix: &str) -> i64 {
    let carrier_profile_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO carrier_profiles
            (tenant_id, user_id, display_name, carrier_type, dot_number, mc_number,
             insurance_status, authority_status, compliance_status, risk_score, status,
             created_at, updated_at)
         VALUES ('tenant-p10', $1, $2, 'motor_carrier', $3, $4,
             'approved', 'verified', 'approved', 1.0, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(user_id)
    .bind(format!("Carrier {suffix}"))
    .bind(format!("DOT-P10-{suffix}"))
    .bind(format!("MC-P10-{suffix}"))
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
             VALUES ('tenant-p10', $1, $2, $3, $4, 'approved',
                 CURRENT_TIMESTAMP + INTERVAL '30 days', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(carrier_profile_id)
        .bind(document_type)
        .bind(format!("{document_type} {suffix}"))
        .bind(format!("p10/{suffix}/{document_type}.pdf"))
        .execute(pool)
        .await
        .unwrap();
    }

    sqlx::query(
        "INSERT INTO carrier_equipment
            (tenant_id, carrier_profile_id, equipment_type, quantity, status, created_at, updated_at)
         VALUES ('tenant-p10', $1, 'Dry Van', 1, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(carrier_profile_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO carrier_lanes
            (tenant_id, carrier_profile_id, origin_state, destination_state, status, created_at, updated_at)
         VALUES ('tenant-p10', $1, 'TX', 'TN', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
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
