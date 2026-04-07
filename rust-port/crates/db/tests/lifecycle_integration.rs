use db::{
    DbPool, connect, migrate,
    payments::{EscrowTransitionParams, apply_escrow_transition, find_escrow_for_leg},
    tms::{apply_status_webhook, find_handoff_by_id, push_handoff},
};
use domain::payments::EscrowStatus;
use serial_test::serial;
use shared::{TmsHandoffPayload, TmsStatusWebhookRequest};
use sqlx::Row;

struct LegFixture {
    shipper_user_id: i64,
    carrier_user_id: i64,
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
            stloads_sync_errors,
            stloads_external_refs,
            stloads_handoff_events,
            stloads_handoffs,
            escrows,
            load_history,
            load_legs,
            loads,
            locations,
            load_status_master,
            users
         RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await?;
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
        leg_id,
    })
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

    Ok(())
}

#[tokio::test]
#[serial]
async fn tms_cancel_webhook_withdraws_local_projection() -> Result<(), Box<dyn std::error::Error>> {
    let Some(pool) = prepare_pool().await? else {
        return Ok(());
    };

    let payload = sample_tms_payload("TMS-CANCEL-1001");
    let publish_result = push_handoff(&pool, &payload).await?;
    let load_id = publish_result
        .load_id
        .expect("push should materialize a load");

    let webhook_result = apply_status_webhook(
        &pool,
        &TmsStatusWebhookRequest {
            tms_load_id: payload.tms_load_id.clone(),
            tenant_id: payload.tenant_id.clone(),
            tms_status: "cancelled".into(),
            status_at: Some("2026-04-07T10:45:00Z".into()),
            source_module: Some("integration_test".into()),
            pushed_by: Some("tester".into()),
            detail: Some("Cancelled upstream".into()),
            rate_update: None,
        },
    )
    .await?
    .expect("published handoff should accept a cancellation webhook");

    assert_eq!(webhook_result.action_label, "auto_withdraw");
    let updated_handoff = find_handoff_by_id(&pool, publish_result.handoff.id)
        .await?
        .expect("handoff should still exist");
    assert_eq!(updated_handoff.status, "withdrawn");
    assert!(updated_handoff.withdrawn_at.is_some());

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
    let publish_result = push_handoff(&pool, &payload).await?;
    let load_id = publish_result
        .load_id
        .expect("push should materialize a load");

    let webhook_result = apply_status_webhook(
        &pool,
        &TmsStatusWebhookRequest {
            tms_load_id: payload.tms_load_id.clone(),
            tenant_id: payload.tenant_id.clone(),
            tms_status: "in_transit".into(),
            status_at: Some("2026-04-07T12:15:00Z".into()),
            source_module: Some("integration_test".into()),
            pushed_by: Some("tester".into()),
            detail: Some("Carrier still moving after rate revision".into()),
            rate_update: Some(1260.0),
        },
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
