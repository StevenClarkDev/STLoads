use serial_test::serial;

#[tokio::test]
#[serial(tenant_scope_db)]
async fn tenant_scope_blocks_cross_tenant_resources() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    let tenant_a = "tenant-p7-a";
    let tenant_b = "tenant-p7-b";
    seed_tenant(&pool, tenant_a).await;
    seed_tenant(&pool, tenant_b).await;

    let posting_a = seed_posting(&pool, tenant_a, "P7-A").await;
    let posting_b = seed_posting(&pool, tenant_b, "P7-B").await;
    let offer_a = seed_offer(&pool, tenant_a, posting_a).await;
    let document_a = seed_document(&pool, tenant_a, "posting", posting_a).await;
    let settlement_a = seed_settlement(&pool, tenant_a).await;
    let conversation_a = seed_conversation_for_offer(&pool, tenant_a, posting_a).await;

    for (kind, id) in [
        (db::auth::TenantResourceKind::Posting, posting_a),
        (db::auth::TenantResourceKind::Offer, offer_a),
        (db::auth::TenantResourceKind::Document, document_a),
        (db::auth::TenantResourceKind::Payment, settlement_a),
        (db::auth::TenantResourceKind::Chat, conversation_a),
    ] {
        assert!(
            db::auth::tenant_can_access_resource(&pool, tenant_a, kind, id)
                .await
                .unwrap(),
            "{kind:?} should be visible inside its tenant"
        );
        assert!(
            !db::auth::tenant_can_access_resource(&pool, tenant_b, kind, id)
                .await
                .unwrap(),
            "{kind:?} should be hidden from a different tenant"
        );
    }

    assert!(
        db::auth::tenant_can_access_resource(
            &pool,
            tenant_b,
            db::auth::TenantResourceKind::Posting,
            posting_b
        )
        .await
        .unwrap()
    );
}

#[tokio::test]
#[serial(tenant_scope_db)]
async fn platform_admin_can_cross_tenants_but_tenant_admin_cannot() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    seed_tenant(&pool, "tenant-p7-a").await;
    seed_tenant(&pool, "tenant-p7-b").await;

    let platform = db::auth::TenantScopeRecord {
        tenant_id: "tenant-p7-a".into(),
        organization_id: Some(1),
        scoped_role_key: "platform_admin".into(),
        organization_type: Some("platform".into()),
    };
    let tenant_admin = db::auth::TenantScopeRecord {
        tenant_id: "tenant-p7-a".into(),
        organization_id: Some(1),
        scoped_role_key: "tenant_admin".into(),
        organization_type: Some("shipper".into()),
    };

    assert!(db::auth::tenant_scope_can_admin_tenant(
        &platform,
        "tenant-p7-b"
    ));
    assert!(db::auth::tenant_scope_can_admin_tenant(
        &tenant_admin,
        "tenant-p7-a"
    ));
    assert!(!db::auth::tenant_scope_can_admin_tenant(
        &tenant_admin,
        "tenant-p7-b"
    ));
}

#[tokio::test]
#[serial(tenant_scope_db)]
async fn support_impersonation_requires_reason_and_expiry() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    seed_tenant(&pool, "tenant-p7-a").await;
    let actor = backend::test_support::insert_user_with_role_status(
        &pool,
        "Support Actor",
        "support-p7@example.com",
        domain::auth::UserRole::Admin,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let target = backend::test_support::insert_user_with_role_status(
        &pool,
        "Tenant User",
        "tenant-user-p7@example.com",
        domain::auth::UserRole::Shipper,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();

    assert!(
        db::auth::create_support_impersonation_audit(
            &pool,
            actor.id,
            target.id,
            "tenant-p7-a",
            " ",
            15
        )
        .await
        .is_err()
    );
    assert!(
        db::auth::create_support_impersonation_audit(
            &pool,
            actor.id,
            target.id,
            "tenant-p7-a",
            "billing support ticket ST-100",
            0
        )
        .await
        .is_err()
    );

    let audit = db::auth::create_support_impersonation_audit(
        &pool,
        actor.id,
        target.id,
        "tenant-p7-a",
        "billing support ticket ST-100",
        15,
    )
    .await
    .unwrap();

    assert_eq!(audit.actor_user_id, actor.id);
    assert_eq!(audit.target_user_id, target.id);
    assert_eq!(audit.tenant_id, "tenant-p7-a");
    assert_eq!(audit.reason, "billing support ticket ST-100");
    assert!(audit.expires_at > audit.started_at);
}

#[tokio::test]
#[serial(tenant_scope_db)]
async fn session_includes_tenant_scope_and_scoped_permissions() {
    let Some(pool) = backend::test_support::prepare_pool().await.unwrap() else {
        return;
    };

    seed_tenant(&pool, "tenant-p7-session").await;
    let user = backend::test_support::insert_user_with_role_status(
        &pool,
        "Tenant Admin",
        "tenant-admin-p7@example.com",
        domain::auth::UserRole::Admin,
        domain::auth::AccountStatus::Approved,
    )
    .await
    .unwrap();
    let organization_id = seed_organization(&pool, "tenant-p7-session", "shipper").await;
    seed_membership(
        &pool,
        "tenant-p7-session",
        organization_id,
        user.id,
        "tenant_admin",
    )
    .await;

    let state = backend::test_support::test_state(pool);
    let session = backend::auth_session::build_session_state(&state, &user).await;

    assert_eq!(
        session
            .tenant_scope
            .as_ref()
            .map(|scope| scope.tenant_id.as_str()),
        Some("tenant-p7-session")
    );
    assert_eq!(
        session
            .tenant_scope
            .as_ref()
            .map(|scope| scope.organization_id),
        Some(Some(organization_id as u64))
    );
    assert!(
        session
            .permissions
            .iter()
            .any(|permission| permission == "manage_integration_actions")
    );
}

async fn seed_tenant(pool: &db::DbPool, tenant_id: &str) {
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at, updated_at)
         VALUES ($1, $2, $3, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(format!("Tenant {tenant_id}"))
    .bind(tenant_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_organization(pool: &db::DbPool, tenant_id: &str, organization_type: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO organizations
            (tenant_id, legal_name, organization_type, status, kyc_status, created_at, updated_at)
         VALUES ($1, $2, $3, 'active', 'approved', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(format!("{tenant_id} Organization"))
    .bind(organization_type)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_membership(
    pool: &db::DbPool,
    tenant_id: &str,
    organization_id: i64,
    user_id: i64,
    role_key: &str,
) {
    sqlx::query(
        "INSERT INTO organization_memberships
            (tenant_id, organization_id, user_id, role_key, status, accepted_at, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(user_id)
    .bind(role_key)
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_posting(pool: &db::DbPool, tenant_id: &str, suffix: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO stloads_postings
            (tenant_id, posting_number, title, freight_mode, status, visibility, readiness, created_at, updated_at)
         VALUES ($1, $2, $3, 'road', 'published', 'public', 'ready', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(format!("POST-{suffix}"))
    .bind(format!("Posting {suffix}"))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_offer(pool: &db::DbPool, tenant_id: &str, posting_id: i64) -> i64 {
    let fixture = backend::test_support::insert_load_fixture(pool, 1)
        .await
        .unwrap();
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO offers
            (load_leg_id, carrier_id, amount, status_id, tenant_id, posting_id, created_at, updated_at)
         VALUES ($1, $2, 120000, 1, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(fixture.leg_id)
    .bind(fixture.carrier_user.id)
    .bind(tenant_id)
    .bind(posting_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_document(
    pool: &db::DbPool,
    tenant_id: &str,
    document_scope: &str,
    document_scope_id: i64,
) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO document_versions
            (tenant_id, document_scope, document_scope_id, version_number, storage_key, file_name, file_hash, created_at)
         VALUES ($1, $2, $3, 1, 'p7/doc.pdf', 'doc.pdf', 'hash-p7', CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(document_scope)
    .bind(document_scope_id.to_string())
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_settlement(pool: &db::DbPool, tenant_id: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO settlements
            (tenant_id, settlement_number, status, currency, gross_amount, deductions_amount, net_amount, created_at, updated_at)
         VALUES ($1, $2, 'draft', 'USD', 1000, 0, 1000, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(format!("SET-{tenant_id}"))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn seed_conversation_for_offer(pool: &db::DbPool, tenant_id: &str, posting_id: i64) -> i64 {
    let fixture = backend::test_support::insert_load_fixture(pool, 1)
        .await
        .unwrap();
    let conversation_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO conversations
            (load_leg_id, shipper_id, carrier_id, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id",
    )
    .bind(fixture.leg_id)
    .bind(fixture.owner_user.id)
    .bind(fixture.carrier_user.id)
    .fetch_one(pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO offers
            (load_leg_id, carrier_id, conversation_id, amount, status_id, tenant_id, posting_id, created_at, updated_at)
         VALUES ($1, $2, $3, 1000, 1, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(fixture.leg_id)
    .bind(fixture.carrier_user.id)
    .bind(conversation_id)
    .bind(tenant_id)
    .bind(posting_id)
    .execute(pool)
    .await
    .unwrap();

    conversation_id
}
