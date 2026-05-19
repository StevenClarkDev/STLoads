use std::{error::Error, path::PathBuf};

use axum::http::{HeaderMap, HeaderValue, header};
use bcrypt::hash;
use db::{
    DbPool,
    auth::{UserRecord, find_user_by_id},
    connect, migrate,
};
use domain::auth::{AccountStatus, UserRole};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    auth_session, config::RuntimeConfig, document_storage::DocumentStorageService,
    email::EmailService, realtime_bus::RoutedRealtimeEvent, state::AppState, stripe::StripeService,
};

pub struct LoadFixture {
    pub load_id: i64,
    pub owner_user: UserRecord,
    pub carrier_user: UserRecord,
    pub leg_id: i64,
}

pub fn test_database_url() -> Option<String> {
    std::env::var("RUST_TEST_DATABASE_URL")
        .ok()
        .or_else(|| std::env::var("TEST_DATABASE_URL").ok())
}

pub async fn prepare_pool() -> Result<Option<DbPool>, Box<dyn Error>> {
    let Some(database_url) = test_database_url() else {
        eprintln!("skipping backend DB acceptance test because RUST_TEST_DATABASE_URL is not set");
        return Ok(None);
    };

    let pool = connect(&database_url).await?;
    migrate(&pool).await?;
    reset_database(&pool).await?;
    seed_load_statuses(&pool).await?;
    Ok(Some(pool))
}

pub fn test_state(pool: DbPool) -> AppState {
    let config = RuntimeConfig {
        bind_addr: "127.0.0.1".into(),
        port: 3001,
        deployment_target: "backend-test".into(),
        environment: "development".into(),
        public_base_url: Some("https://rust.test".into()),
        cors_allowed_origins: vec!["https://rust.test".into()],
        run_migrations: false,
        database_url: None,
        database_schema: None,
        document_storage_backend: "local".into(),
        document_storage_root: temp_storage_root().display().to_string(),
        object_storage_bucket: None,
        object_storage_region: "us-south".into(),
        object_storage_endpoint: None,
        object_storage_access_key_id: None,
        object_storage_secret_access_key: None,
        object_storage_session_token: None,
        object_storage_force_path_style: false,
        object_storage_prefix: "tests".into(),
        stripe_webhook_shared_secret: None,
        stripe_webhook_connect_secret: None,
        stripe_secret_key: None,
        stripe_api_base_url: "https://api.stripe.com/v1".into(),
        stripe_connect_refresh_url: None,
        stripe_connect_return_url: None,
        stripe_live_transfers_required: false,
        atmp_outbound_base_url: None,
        atmp_integration_shared_secret: None,
        atmp_integration_require_signature: false,
        atmp_integration_replay_window_seconds: 300,
        atmp_integration_rate_limit_per_minute: 120,
        atmp_outbound_worker_enabled: false,
        atmp_outbound_interval_seconds: 30,
        atmp_outbound_batch_size: 25,
        atmp_outbound_max_attempts: 8,
        tms_shared_secret: None,
        tms_reconciliation_worker_enabled: false,
        tms_reconciliation_interval_seconds: 21_600,
        tms_retry_worker_enabled: false,
        tms_retry_interval_seconds: 300,
        tms_retry_batch_size: 10,
        tms_retry_max_attempts: 5,
        tms_stale_handoff_days: 30,
        mail_mailer: "log".into(),
        mail_host: None,
        mail_port: 587,
        mail_username: None,
        mail_password: None,
        mail_encryption: None,
        mail_from_address: "noreply@stloads.test".into(),
        mail_from_name: "STLoads Tests".into(),
        mail_fail_open: true,
        mail_outbox_enabled: false,
        mail_outbox_worker_enabled: false,
        mail_outbox_batch_size: 25,
        mail_outbox_retry_interval_seconds: 30,
        mail_outbox_max_attempts: 8,
        portal_url: "https://portal.stloads.test".into(),
    };
    let (realtime_tx, _) = broadcast::channel::<RoutedRealtimeEvent>(32);
    let document_storage = DocumentStorageService::from_config(&config);
    let email = EmailService::from_config_with_pool(&config, Some(pool.clone()));
    let stripe = StripeService::from_config(&config);

    AppState {
        config,
        pool: Some(pool),
        document_storage,
        email,
        stripe,
        integration_auth: crate::integration_auth::IntegrationAuthState::default(),
        realtime_tx,
    }
}

pub async fn auth_headers_for_user(
    state: &AppState,
    user: &UserRecord,
) -> Result<HeaderMap, Box<dyn Error>> {
    let token = auth_session::issue_session_token(state, user).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    Ok(headers)
}

pub async fn insert_user_with_role_status(
    pool: &DbPool,
    name: &str,
    email: &str,
    role: UserRole,
    status: AccountStatus,
) -> Result<UserRecord, Box<dyn Error>> {
    let password_hash = hash("Password123!", 4)?;
    let user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users
            (name, email, password, role_id, phone_no, address, email_verified_at, status, created_at, updated_at)
         VALUES (
            $1, $2, $3, $4, $5, $6,
            CASE WHEN $7 = 4 THEN NULL ELSE CURRENT_TIMESTAMP END,
            $7,
            CURRENT_TIMESTAMP,
            CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(name)
    .bind(email.to_ascii_lowercase())
    .bind(password_hash)
    .bind(role.legacy_id())
    .bind(Some("555-0100"))
    .bind(Some("101 Test Ave"))
    .bind(status.legacy_code())
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO model_has_roles (role_id, model_type, model_id)
         VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(i64::from(role.legacy_id()))
    .bind("App\\Models\\User")
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(find_user_by_id(pool, user_id)
        .await?
        .ok_or("user missing after insert")?)
}

pub async fn insert_load_fixture(
    pool: &DbPool,
    status_id: i16,
) -> Result<LoadFixture, Box<dyn Error>> {
    let owner_user = insert_user_with_role_status(
        pool,
        "Shipper Fixture",
        &format!("shipper-{}@example.com", Uuid::new_v4()),
        UserRole::Shipper,
        AccountStatus::Approved,
    )
    .await?;
    let carrier_user = insert_user_with_role_status(
        pool,
        "Carrier Fixture",
        &format!("carrier-{}@example.com", Uuid::new_v4()),
        UserRole::Carrier,
        AccountStatus::Approved,
    )
    .await?;
    let pickup_location_id = insert_location(pool, "Dallas pickup").await?;
    let delivery_location_id = insert_location(pool, "Memphis delivery").await?;

    let load_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO loads (
            load_number, title, user_id, weight_unit, weight, status, leg_count, created_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, 1, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(format!("LD-T-{}", Uuid::new_v4().simple()))
    .bind("Backend route fixture load")
    .bind(owner_user.id)
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
            $5, $6, $7, $8, CURRENT_TIMESTAMP, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(load_id)
    .bind(format!("LEG-{}", Uuid::new_v4().simple()))
    .bind(pickup_location_id)
    .bind(delivery_location_id)
    .bind("Fixed")
    .bind(2450.0_f64)
    .bind(status_id)
    .bind(carrier_user.id)
    .bind(2450.0_f64)
    .fetch_one(pool)
    .await?;

    Ok(LoadFixture {
        load_id,
        owner_user,
        carrier_user,
        leg_id,
    })
}

pub async fn fetch_password_reset_token(
    pool: &DbPool,
    email: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT token
         FROM password_reset_tokens
         WHERE email = $1
         LIMIT 1",
    )
    .bind(email.to_ascii_lowercase())
    .fetch_optional(pool)
    .await
}

pub async fn read_leg_status(pool: &DbPool, leg_id: i64) -> Result<i16, sqlx::Error> {
    sqlx::query_scalar::<_, i16>("SELECT status_id FROM load_legs WHERE id = $1")
        .bind(leg_id)
        .fetch_one(pool)
        .await
}

async fn reset_database(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "TRUNCATE TABLE
            dead_letter_events,
            support_impersonation_audits,
            atmp_contract_payloads,
            atmp_sync_errors,
            atmp_outbound_events,
            atmp_inbound_events,
            quickpay_terms,
            factoring_profiles,
            payment_disputes,
            payment_holds,
            accessorial_requests,
            settlement_lines,
            settlements,
            required_document_rules,
            document_reviews,
            document_versions,
            carrier_risk_flags,
            compliance_overrides,
            eligibility_results,
            carrier_compliance_status,
            compliance_rules,
            compliance_documents,
            cancellation_requests,
            booking_locks,
            booking_awards,
            tender_invites,
            tenders,
            counteroffers,
            offer_versions,
            stloads_posting_visibility_rules,
            stloads_posting_requirements,
            stloads_posting_rates,
            stloads_posting_stops,
            stloads_posting_versions,
            stloads_postings,
            carrier_group_members,
            carrier_groups,
            carrier_availability,
            carrier_lanes,
            carrier_equipment,
            carrier_profiles,
            tenant_external_mappings,
            organization_memberships,
            organization_branches,
            organizations,
            tenants,
            stloads_reconciliation_log,
            stloads_sync_errors,
            stloads_external_refs,
            stloads_handoff_events,
            stloads_handoffs,
            email_outbox,
            personal_access_tokens,
            password_reset_tokens,
            leg_events,
            leg_locations,
            leg_documents,
            escrows,
            load_history,
            load_legs,
            loads,
            kyc_documents,
            user_history,
            user_details,
            model_has_roles,
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

async fn seed_load_statuses(pool: &DbPool) -> Result<(), sqlx::Error> {
    for (id, name, slug, sort_order, is_terminal) in [
        (0_i16, "Rejected", "rejected", 0_i32, true),
        (1_i16, "New", "new", 1_i32, false),
        (2_i16, "Reviewed", "reviewed", 2_i32, false),
        (4_i16, "Booked", "booked", 4_i32, false),
        (5_i16, "Pickup Started", "pickup_started", 5_i32, false),
        (6_i16, "At Pickup", "at_pickup", 6_i32, false),
        (7_i16, "In Transit", "in_transit", 7_i32, false),
        (8_i16, "Escrow Funded", "escrow_funded", 8_i32, false),
        (9_i16, "At Delivery", "at_delivery", 9_i32, false),
        (10_i16, "Delivered", "delivered", 10_i32, false),
        (11_i16, "Paid Out", "paid_out", 11_i32, true),
    ] {
        sqlx::query(
            "INSERT INTO load_status_master (id, name, slug, description, sort_order, is_terminal)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(format!("{} seeded for backend acceptance tests", name))
        .bind(sort_order)
        .bind(is_terminal)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_location(pool: &DbPool, name: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO locations (name, city_id, country_id, created_at, updated_at)
         VALUES ($1, NULL, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(name)
    .fetch_one(pool)
    .await
}

fn temp_storage_root() -> PathBuf {
    std::env::temp_dir().join(format!("stloads-backend-tests-{}", Uuid::new_v4()))
}
