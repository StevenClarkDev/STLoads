use serial_test::serial;

#[tokio::test]
#[serial(eligibility_db)]
async fn every_compliance_block_reason_prevents_booking_until_overridden() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let admin = backend::test_support::insert_user_with_role_status(
        &pool,
        "Eligibility Admin",
        "eligibility-admin-p9@example.com",
        domain::auth::UserRole::Admin,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();

    let cases = [
        ("profile_inactive", "carrier_profile_inactive"),
        ("compliance_pending", "carrier_compliance_not_approved"),
        ("missing_packet", "carrier_packet_incomplete"),
        ("missing_w9", "w9_missing"),
        ("insurance_expired", "insurance_expired"),
        ("authority_invalid", "authority_not_verified"),
        ("missing_dot_mc", "authority_identifier_missing"),
        ("missing_equipment", "equipment_not_eligible"),
        ("lane_mismatch", "lane_not_eligible"),
        ("carrier_blocked", "carrier_blocked"),
        ("private_not_approved", "customer_approval_required"),
        ("fraud_flag", "fraud_risk"),
        ("double_broker_flag", "double_broker_risk"),
    ];

    for (suffix, expected_block) in cases {
        let fixture = seed_eligible_fixture(&pool, suffix).await;
        apply_block(&pool, &fixture, suffix).await;

        let decision = db::eligibility::evaluate_carrier_eligibility(
            &pool,
            "tenant-p9",
            fixture.posting_id,
            fixture.carrier_profile_id,
        )
        .await
        .unwrap();

        assert!(
            !decision.eligible,
            "{suffix} should prevent booking before override"
        );
        assert!(
            decision
                .blocks
                .iter()
                .any(|block| block.key == expected_block),
            "{suffix} should include {expected_block}; got {:?}",
            decision.blocks
        );

        let booking_gate = db::eligibility::ensure_carrier_can_book_posting(
            &pool,
            "tenant-p9",
            fixture.posting_id,
            fixture.carrier_profile_id,
        )
        .await;
        assert!(
            booking_gate.is_err(),
            "{suffix} should be rejected by the booking gate"
        );

        db::eligibility::create_compliance_override(
            &pool,
            "tenant-p9",
            fixture.carrier_profile_id,
            Some(fixture.posting_id),
            expected_block,
            "Verified by compliance desk for P9 override regression.",
            None,
            admin.id,
        )
        .await
        .unwrap();

        let overridden = db::eligibility::evaluate_carrier_eligibility(
            &pool,
            "tenant-p9",
            fixture.posting_id,
            fixture.carrier_profile_id,
        )
        .await
        .unwrap();
        assert!(
            overridden.eligible,
            "{suffix} should become eligible after a scoped admin override"
        );
        assert!(
            overridden
                .overridden_blocks
                .iter()
                .any(|block| block.key == expected_block),
            "{suffix} should record the override for audit"
        );
    }
}

struct EligibilityFixture {
    carrier_profile_id: i64,
    posting_id: i64,
}

async fn seed_eligible_fixture(pool: &db::DbPool, suffix: &str) -> EligibilityFixture {
    seed_tenant(pool, "tenant-p9").await;
    let carrier = backend::test_support::insert_user_with_role_status(
        pool,
        &format!("Carrier {suffix}"),
        &format!("carrier-{suffix}-p9@example.com"),
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();

    let carrier_profile_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO carrier_profiles
            (tenant_id, user_id, display_name, carrier_type, dot_number, mc_number,
             insurance_status, authority_status, compliance_status, risk_score, status,
             created_at, updated_at)
         VALUES ('tenant-p9', $1, $2, 'motor_carrier', 'DOT12345', 'MC12345',
             'approved', 'verified', 'approved', 2.5, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(carrier.id)
    .bind(format!("Carrier {suffix}"))
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
                (tenant_id, carrier_profile_id, document_type, document_name, status,
                 storage_key, expires_at, created_at, updated_at)
             VALUES ('tenant-p9', $1, $2, $3, 'approved',
                 $4, CURRENT_TIMESTAMP + INTERVAL '30 days', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(carrier_profile_id)
        .bind(document_type)
        .bind(format!("{document_type} {suffix}"))
        .bind(format!("p9/{suffix}/{document_type}.pdf"))
        .execute(pool)
        .await
        .unwrap();
    }

    sqlx::query(
        "INSERT INTO carrier_equipment
            (tenant_id, carrier_profile_id, equipment_type, quantity, status, created_at, updated_at)
         VALUES ('tenant-p9', $1, 'Dry Van', 2, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(carrier_profile_id)
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO carrier_lanes
            (tenant_id, carrier_profile_id, origin_state, destination_state, status, created_at, updated_at)
         VALUES ('tenant-p9', $1, 'TX', 'TN', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(carrier_profile_id)
    .execute(pool)
    .await
    .unwrap();

    let posting_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_postings
            (tenant_id, source_system, posting_number, title, freight_mode, equipment_type,
             status, visibility, readiness, published_at, created_at, updated_at)
         VALUES ('tenant-p9', 'test', $1, $2, 'road', 'Dry Van',
             'published', 'public', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(format!("P9-{suffix}"))
    .bind(format!("P9 Fixture {suffix}"))
    .fetch_one(pool)
    .await
    .unwrap();

    for (sequence, stop_type, city, state) in [
        (1_i32, "pickup", "Dallas", "TX"),
        (2_i32, "delivery", "Memphis", "TN"),
    ] {
        sqlx::query(
            "INSERT INTO stloads_posting_stops
                (tenant_id, posting_id, stop_sequence, stop_type, city, state_region,
                 created_at, updated_at)
             VALUES ('tenant-p9', $1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
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

    EligibilityFixture {
        carrier_profile_id,
        posting_id,
    }
}

async fn apply_block(pool: &db::DbPool, fixture: &EligibilityFixture, suffix: &str) {
    match suffix {
        "profile_inactive" => {
            sqlx::query("UPDATE carrier_profiles SET status = 'suspended' WHERE id = $1")
                .bind(fixture.carrier_profile_id)
                .execute(pool)
                .await
                .unwrap();
        }
        "compliance_pending" => {
            sqlx::query("UPDATE carrier_profiles SET compliance_status = 'pending' WHERE id = $1")
                .bind(fixture.carrier_profile_id)
                .execute(pool)
                .await
                .unwrap();
        }
        "missing_packet" => delete_document(pool, fixture, "carrier_packet").await,
        "missing_w9" => delete_document(pool, fixture, "w9").await,
        "insurance_expired" => {
            sqlx::query(
                "UPDATE compliance_documents
                 SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 day'
                 WHERE carrier_profile_id = $1 AND document_type = 'insurance_certificate'",
            )
            .bind(fixture.carrier_profile_id)
            .execute(pool)
            .await
            .unwrap();
        }
        "authority_invalid" => {
            sqlx::query("UPDATE carrier_profiles SET authority_status = 'invalid' WHERE id = $1")
                .bind(fixture.carrier_profile_id)
                .execute(pool)
                .await
                .unwrap();
        }
        "missing_dot_mc" => {
            sqlx::query(
                "UPDATE carrier_profiles SET dot_number = NULL, mc_number = NULL WHERE id = $1",
            )
            .bind(fixture.carrier_profile_id)
            .execute(pool)
            .await
            .unwrap();
        }
        "missing_equipment" => {
            sqlx::query("DELETE FROM carrier_equipment WHERE carrier_profile_id = $1")
                .bind(fixture.carrier_profile_id)
                .execute(pool)
                .await
                .unwrap();
        }
        "lane_mismatch" => {
            sqlx::query(
                "UPDATE carrier_lanes
                 SET origin_state = 'CA', destination_state = 'WA'
                 WHERE carrier_profile_id = $1",
            )
            .bind(fixture.carrier_profile_id)
            .execute(pool)
            .await
            .unwrap();
        }
        "carrier_blocked" => {
            sqlx::query(
                "INSERT INTO stloads_posting_visibility_rules
                    (tenant_id, posting_id, rule_type, target_type, target_id, allow, created_at, updated_at)
                 VALUES ('tenant-p9', $1, 'carrier_access', 'carrier_profile', $2, FALSE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            )
            .bind(fixture.posting_id)
            .bind(fixture.carrier_profile_id.to_string())
            .execute(pool)
            .await
            .unwrap();
        }
        "private_not_approved" => {
            sqlx::query("UPDATE stloads_postings SET visibility = 'private' WHERE id = $1")
                .bind(fixture.posting_id)
                .execute(pool)
                .await
                .unwrap();
        }
        "fraud_flag" => insert_risk_flag(pool, fixture, "fraud", "critical").await,
        "double_broker_flag" => insert_risk_flag(pool, fixture, "double_broker", "critical").await,
        _ => unreachable!("unknown P9 fixture suffix {suffix}"),
    }
}

async fn delete_document(pool: &db::DbPool, fixture: &EligibilityFixture, document_type: &str) {
    sqlx::query(
        "DELETE FROM compliance_documents WHERE carrier_profile_id = $1 AND document_type = $2",
    )
    .bind(fixture.carrier_profile_id)
    .bind(document_type)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_risk_flag(
    pool: &db::DbPool,
    fixture: &EligibilityFixture,
    flag_type: &str,
    severity: &str,
) {
    sqlx::query(
        "INSERT INTO carrier_risk_flags
            (tenant_id, carrier_profile_id, flag_type, severity, title, status, created_at, updated_at)
         VALUES ('tenant-p9', $1, $2, $3, $4, 'open', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(fixture.carrier_profile_id)
    .bind(flag_type)
    .bind(severity)
    .bind(format!("{flag_type} risk"))
    .execute(pool)
    .await
    .unwrap();
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
