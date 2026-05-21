use chrono::Utc;
use serial_test::serial;

#[tokio::test]
#[serial(load_board_search_db)]
async fn carrier_search_excludes_hidden_closed_withdrawn_and_ineligible_freight() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let carrier = backend::test_support::insert_user_with_role_status(
        &pool,
        "Carrier Search",
        "carrier-search-p8@example.com",
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let carrier_profile_id = seed_carrier_profile(&pool, carrier.id).await;

    let visible = seed_search_fixture(&pool, "VISIBLE-P8", Some("published"), None).await;
    let withdrawn = seed_search_fixture(&pool, "WITHDRAWN-P8", Some("withdrawn"), None).await;
    let closed = seed_search_fixture(&pool, "CLOSED-P8", Some("closed"), None).await;
    let hidden = seed_search_fixture(&pool, "HIDDEN-P8", Some("hidden"), None).await;
    let ineligible = seed_search_fixture(
        &pool,
        "INELIGIBLE-P8",
        Some("published"),
        Some(carrier_profile_id),
    )
    .await;

    let filters = db::dispatch::LoadBoardSearchFilters {
        origin: Some("Dallas".into()),
        destination: Some("Memphis".into()),
        load_type: None,
        equipment: Some("Dry Van".into()),
        mode: Some("road".into()),
        status: None,
        date_from: None,
        date_to: None,
        min_rate: None,
        max_rate: None,
        min_rpm: None,
        max_rpm: None,
        min_weight: None,
        max_weight: None,
        hazmat: None,
        temperature_controlled: None,
        service_level: Some("standard".into()),
        visibility: Some("public".into()),
        sort: Some("pickup_date".into()),
        page: 1,
        per_page: 20,
    };

    let rows = db::dispatch::search_load_board_for_carrier(&pool, carrier.id, &filters)
        .await
        .unwrap();
    let ids = rows
        .iter()
        .map(|row| row.leg_id)
        .collect::<std::collections::HashSet<_>>();

    assert!(ids.contains(&visible.leg_id));
    assert!(!ids.contains(&withdrawn.leg_id));
    assert!(!ids.contains(&closed.leg_id));
    assert!(!ids.contains(&hidden.leg_id));
    assert!(!ids.contains(&ineligible.leg_id));
}

#[tokio::test]
#[serial(load_board_search_db)]
async fn saved_search_and_alert_rules_are_tenant_scoped() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let user = backend::test_support::insert_user_with_role_status(
        &pool,
        "Carrier Saved Search",
        "carrier-saved-search-p8@example.com",
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    seed_tenant(&pool, "tenant-p8").await;

    let filters = db::dispatch::LoadBoardSearchFilters {
        origin: Some("Dallas".into()),
        destination: Some("Memphis".into()),
        load_type: None,
        equipment: None,
        mode: None,
        status: None,
        date_from: None,
        date_to: None,
        min_rate: Some(1000.0),
        max_rate: None,
        min_rpm: None,
        max_rpm: None,
        min_weight: None,
        max_weight: None,
        hazmat: Some(false),
        temperature_controlled: Some(false),
        service_level: None,
        visibility: Some("public".into()),
        sort: Some("rate_desc".into()),
        page: 1,
        per_page: 20,
    };

    let saved = db::dispatch::upsert_load_board_saved_search(
        &pool,
        "tenant-p8",
        user.id,
        "Dallas to Memphis",
        &filters,
        true,
    )
    .await
    .unwrap();
    let alert = db::dispatch::upsert_load_board_alert_rule(
        &pool,
        "tenant-p8",
        saved.id,
        user.id,
        "email",
        true,
    )
    .await
    .unwrap();

    let searches = db::dispatch::list_load_board_saved_searches(&pool, "tenant-p8", user.id)
        .await
        .unwrap();
    assert_eq!(searches.len(), 1);
    assert_eq!(searches[0].id, saved.id);
    assert_eq!(alert.saved_search_id, saved.id);
    assert!(
        db::dispatch::list_load_board_saved_searches(&pool, "other-tenant", user.id)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
#[serial(load_board_search_db)]
async fn carrier_search_filters_and_paginates_production_sized_results() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let carrier = backend::test_support::insert_user_with_role_status(
        &pool,
        "Carrier Pagination",
        "carrier-pagination-p8@example.com",
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    seed_carrier_profile(&pool, carrier.id).await;

    let mut expected_ids = Vec::new();
    for index in 0..35 {
        let fixture = seed_search_fixture(
            &pool,
            &format!("PAGE-P8-{index:02}"),
            Some("published"),
            None,
        )
        .await;
        expected_ids.push(fixture.leg_id);
    }

    let booked_by_other =
        seed_search_fixture(&pool, "BOOKED-OTHER-P8", Some("published"), None).await;
    sqlx::query(
        "UPDATE load_legs SET booked_carrier_id = $1, booked_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(carrier.id + 10_000)
    .bind(booked_by_other.leg_id)
    .execute(&pool)
    .await
    .unwrap();

    let filters = db::dispatch::LoadBoardSearchFilters {
        origin: Some("Dallas".into()),
        destination: Some("Memphis".into()),
        load_type: Some("Full Truckload".into()),
        equipment: Some("Dry Van".into()),
        mode: Some("road".into()),
        status: Some("new".into()),
        date_from: Some(chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()),
        date_to: Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
        min_rate: Some(2000.0),
        max_rate: Some(3000.0),
        min_rpm: Some(5.0),
        max_rpm: Some(6.0),
        min_weight: Some(40000.0),
        max_weight: Some(43000.0),
        hazmat: Some(false),
        temperature_controlled: Some(false),
        service_level: Some("standard".into()),
        visibility: Some("public".into()),
        sort: Some("pickup_date".into()),
        page: 2,
        per_page: 10,
    };

    let total = db::dispatch::count_load_board_for_carrier(&pool, carrier.id, &filters)
        .await
        .unwrap();
    let page_two = db::dispatch::search_load_board_for_carrier(&pool, carrier.id, &filters)
        .await
        .unwrap();

    assert_eq!(total, 35);
    assert_eq!(page_two.len(), 10);
    assert!(
        page_two
            .iter()
            .all(|row| expected_ids.contains(&row.leg_id))
    );
    assert!(
        !page_two
            .iter()
            .any(|row| row.leg_id == booked_by_other.leg_id)
    );
}

#[tokio::test]
#[serial(load_board_search_db)]
async fn carrier_load_board_rows_expose_transport_mode() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let carrier = backend::test_support::insert_user_with_role_status(
        &pool,
        "Carrier Mode",
        "carrier-mode-p8@example.com",
        domain::auth::UserRole::Carrier,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    seed_carrier_profile(&pool, carrier.id).await;

    let visible = seed_search_fixture(&pool, "MODE-RAIL-P8", Some("published"), None).await;
    sqlx::query(
        "UPDATE stloads_handoffs
         SET freight_mode = 'rail',
             raw_payload = COALESCE(raw_payload, '{}'::jsonb) || jsonb_build_object('transport_mode', 'rail')
         WHERE load_id = (SELECT load_id FROM load_legs WHERE id = $1)",
    )
    .bind(visible.leg_id)
    .execute(&pool)
    .await
    .unwrap();

    let rows =
        db::dispatch::list_load_board_legs_for_carrier_filtered(&pool, carrier.id, Some("all"), 20)
            .await
            .unwrap();
    let row = rows
        .into_iter()
        .find(|row| row.leg_id == visible.leg_id)
        .expect("seeded rail posting should be visible");

    assert_eq!(row.transport_mode.as_deref(), Some("rail"));
}

#[derive(Debug)]
struct SearchFixture {
    leg_id: i64,
}

async fn seed_carrier_profile(pool: &db::DbPool, user_id: i64) -> i64 {
    seed_tenant(pool, "tenant-p8").await;
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO carrier_profiles
            (tenant_id, user_id, display_name, carrier_type, compliance_status, status, created_at, updated_at)
         VALUES ('tenant-p8', $1, 'Carrier Search P8', 'motor_carrier', 'approved', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_search_fixture(
    pool: &db::DbPool,
    suffix: &str,
    handoff_status: Option<&str>,
    ineligible_carrier_profile_id: Option<i64>,
) -> SearchFixture {
    let fixture = backend::test_support::insert_load_fixture(pool, 1)
        .await
        .unwrap();
    sqlx::query("UPDATE load_legs SET booked_carrier_id = NULL, booked_at = NULL, booked_amount = NULL WHERE id = $1")
        .bind(fixture.leg_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE loads
         SET title = $1,
             load_type_id = $3,
             equipment_id = $4,
             weight = 42000,
             is_hazardous = FALSE,
             is_temperature_controlled = FALSE,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(format!("Search fixture {suffix}"))
    .bind(fixture.load_id)
    .bind(seed_load_type(pool).await)
    .bind(seed_equipment(pool).await)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE load_legs
         SET pickup_date = '2026-05-21 09:00:00',
             delivery_date = '2026-05-22 17:00:00',
             bid_status = 'Fixed',
             price = 2500
         WHERE id = $1",
    )
    .bind(fixture.leg_id)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE locations
         SET name = CASE WHEN id = $1 THEN 'Dallas, TX' ELSE 'Memphis, TN' END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id IN ($1, $2)",
    )
    .bind(
        sqlx::query_scalar::<_, i64>("SELECT pickup_location_id FROM load_legs WHERE id = $1")
            .bind(fixture.leg_id)
            .fetch_one(pool)
            .await
            .unwrap(),
    )
    .bind(
        sqlx::query_scalar::<_, i64>("SELECT delivery_location_id FROM load_legs WHERE id = $1")
            .bind(fixture.leg_id)
            .fetch_one(pool)
            .await
            .unwrap(),
    )
    .execute(pool)
    .await
    .unwrap();

    if let Some(status) = handoff_status {
        sqlx::query(
            "INSERT INTO stloads_handoffs
                (tms_load_id, tenant_id, load_id, status, freight_mode, equipment_type, readiness,
                 board_rate, rate_currency, raw_payload, queued_at, published_at, created_at, updated_at)
             VALUES ($1, 'tenant-p8', $2, $3, 'road', 'Dry Van', 'ready', 2500, 'USD',
                 $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(format!("TMS-{suffix}"))
        .bind(fixture.load_id)
        .bind(status)
        .bind(serde_json::json!({
            "visibility": if status == "hidden" { "private" } else { "public" },
            "service_level": "standard",
            "miles": 450
        }))
        .execute(pool)
        .await
        .unwrap();
    }

    if let Some(carrier_profile_id) = ineligible_carrier_profile_id {
        let posting_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO stloads_postings
                (tenant_id, source_system, source_load_id, source_leg_id, posting_number, title,
                 freight_mode, equipment_type, status, visibility, readiness, published_at, created_at, updated_at)
             VALUES ('tenant-p8', 'test', $1, $2, $3, $4, 'road', 'Dry Van', 'published', 'public', 'ready',
                 CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
             RETURNING id",
        )
        .bind(fixture.load_id.to_string())
        .bind(fixture.leg_id.to_string())
        .bind(format!("POST-{suffix}"))
        .bind(format!("Posting {suffix}"))
        .fetch_one(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO eligibility_results
                (tenant_id, posting_id, carrier_profile_id, eligible, result_code, result_detail, evaluated_at)
             VALUES ('tenant-p8', $1, $2, FALSE, 'blocked', 'Carrier blocked for this lane', CURRENT_TIMESTAMP)",
        )
        .bind(posting_id)
        .bind(carrier_profile_id)
        .execute(pool)
        .await
        .unwrap();
    }

    let _ = Utc::now();
    SearchFixture {
        leg_id: fixture.leg_id,
    }
}

async fn seed_load_type(pool: &db::DbPool) -> i64 {
    if let Some(id) = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM load_types WHERE name = 'Full Truckload' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .unwrap()
    {
        return id;
    }

    sqlx::query_scalar::<_, i64>(
        "INSERT INTO load_types (name, created_at, updated_at)
         VALUES ('Full Truckload', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_equipment(pool: &db::DbPool) -> i64 {
    if let Some(id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM equipments WHERE name = 'Dry Van' LIMIT 1")
            .fetch_optional(pool)
            .await
            .unwrap()
    {
        return id;
    }

    sqlx::query_scalar::<_, i64>(
        "INSERT INTO equipments (name, created_at, updated_at)
         VALUES ('Dry Van', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap()
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
