use chrono::NaiveDateTime;
use domain::payments::{
    EscrowStatus, EscrowStatusDescriptor, PaymentsModuleContract, StripeWebhookEventDescriptor,
    escrow_status_descriptors, payments_module_contract, stripe_webhook_events,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;

use crate::{
    DbPool,
    tms::{EnqueueAtmpOutboundEvent, enqueue_atmp_outbound_event},
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EscrowRecord {
    pub id: i64,
    pub leg_id: i64,
    pub payer_user_id: i64,
    pub payee_user_id: i64,
    pub currency: String,
    pub amount: i64,
    pub platform_fee: i64,
    pub status: String,
    pub transfer_group: Option<String>,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub transfer_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl EscrowRecord {
    pub fn escrow_status(&self) -> Option<EscrowStatus> {
        EscrowStatus::from_legacy_label(&self.status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserConnectStateRecord {
    pub id: i64,
    pub stripe_connect_account_id: Option<String>,
    pub payouts_enabled: bool,
    pub kyc_status: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentWorkflowRecord {
    pub id: i64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct EscrowTransitionParams<'a> {
    pub leg_id: i64,
    pub payer_user_id: i64,
    pub payee_user_id: i64,
    pub amount: i64,
    pub platform_fee: i64,
    pub currency: &'a str,
    pub status: EscrowStatus,
    pub transfer_group: Option<&'a str>,
    pub payment_intent_id: Option<&'a str>,
    pub charge_id: Option<&'a str>,
    pub transfer_id: Option<&'a str>,
    pub actor_user_id: Option<i64>,
    pub note: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct LoadLegFinanceContext {
    pub load_id: i64,
    pub status_id: i16,
}

pub async fn find_escrow_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<EscrowRecord>, sqlx::Error> {
    sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE leg_id = $1
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_escrow_by_payment_intent_id(
    pool: &DbPool,
    payment_intent_id: &str,
) -> Result<Option<EscrowRecord>, sqlx::Error> {
    sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE payment_intent_id = $1
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(payment_intent_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_escrows_for_user(
    pool: &DbPool,
    user_id: i64,
) -> Result<Vec<EscrowRecord>, sqlx::Error> {
    sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE payer_user_id = $1 OR payee_user_id = $2
         ORDER BY id DESC",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn list_funded_escrows(pool: &DbPool) -> Result<Vec<EscrowRecord>, sqlx::Error> {
    sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE status = 'funded'
         ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn apply_escrow_transition(
    pool: &DbPool,
    params: EscrowTransitionParams<'_>,
) -> Result<Option<EscrowRecord>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(finance_context) = sqlx::query_as::<_, LoadLegFinanceContext>(
        "SELECT load_id, status_id
         FROM load_legs
         WHERE deleted_at IS NULL AND id = $1
         LIMIT 1",
    )
    .bind(params.leg_id)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    let existing_escrow = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE leg_id = $1
         LIMIT 1",
    )
    .bind(params.leg_id)
    .fetch_optional(&mut *tx)
    .await?;

    let resolved_transfer_group = params.transfer_group.map(str::to_string).or_else(|| {
        existing_escrow
            .as_ref()
            .and_then(|escrow| escrow.transfer_group.clone())
    });
    let resolved_payment_intent_id = params.payment_intent_id.map(str::to_string).or_else(|| {
        existing_escrow
            .as_ref()
            .and_then(|escrow| escrow.payment_intent_id.clone())
    });
    let resolved_charge_id = params.charge_id.map(str::to_string).or_else(|| {
        existing_escrow
            .as_ref()
            .and_then(|escrow| escrow.charge_id.clone())
    });
    let resolved_transfer_id = params.transfer_id.map(str::to_string).or_else(|| {
        existing_escrow
            .as_ref()
            .and_then(|escrow| escrow.transfer_id.clone())
    });

    if let Some(existing_escrow) = existing_escrow.as_ref() {
        sqlx::query(
            "UPDATE escrows
             SET payer_user_id = $1,
                 payee_user_id = $2,
                 currency = $3,
                 amount = $4,
                 platform_fee = $5,
                 status = $6,
                 transfer_group = $7,
                 payment_intent_id = $8,
                 charge_id = $9,
                 transfer_id = $10,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $11",
        )
        .bind(params.payer_user_id)
        .bind(params.payee_user_id)
        .bind(params.currency)
        .bind(params.amount)
        .bind(params.platform_fee)
        .bind(params.status.as_legacy_label())
        .bind(resolved_transfer_group.as_deref())
        .bind(resolved_payment_intent_id.as_deref())
        .bind(resolved_charge_id.as_deref())
        .bind(resolved_transfer_id.as_deref())
        .bind(existing_escrow.id)
        .execute(&mut *tx)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO escrows (
                leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(params.leg_id)
        .bind(params.payer_user_id)
        .bind(params.payee_user_id)
        .bind(params.currency)
        .bind(params.amount)
        .bind(params.platform_fee)
        .bind(params.status.as_legacy_label())
        .bind(resolved_transfer_group.as_deref())
        .bind(resolved_payment_intent_id.as_deref())
        .bind(resolved_charge_id.as_deref())
        .bind(resolved_transfer_id.as_deref())
        .execute(&mut *tx)
        .await?;
    }

    match params.status {
        EscrowStatus::Funded => {
            sqlx::query(
                "UPDATE load_legs
                 SET status_id = CASE WHEN status_id < 8 THEN 8 ELSE status_id END,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1",
            )
            .bind(params.leg_id)
            .execute(&mut *tx)
            .await?;
        }
        EscrowStatus::Released => {
            sqlx::query(
                "UPDATE load_legs
                 SET status_id = 11,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = $1",
            )
            .bind(params.leg_id)
            .execute(&mut *tx)
            .await?;
        }
        _ => {}
    }

    let history_status = match params.status {
        EscrowStatus::Funded => 8,
        EscrowStatus::Released => 11,
        _ => finance_context.status_id,
    };
    let history_note = params.note.map(str::to_string).unwrap_or_else(|| {
        format!(
            "Rust payments transition recorded escrow as {}.",
            params.status.as_legacy_label()
        )
    });

    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(finance_context.load_id)
    .bind(params.actor_user_id)
    .bind(history_status)
    .bind(history_note)
    .execute(&mut *tx)
    .await?;

    let updated_escrow = sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE leg_id = $1
         LIMIT 1",
    )
    .bind(params.leg_id)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    if let Some(escrow) = updated_escrow.as_ref() {
        match params.status {
            EscrowStatus::Funded => {
                enqueue_finance_event(
                    pool,
                    "escrow_funded",
                    params.leg_id,
                    Some(escrow.id),
                    params.actor_user_id,
                    json!({"amount": params.amount, "currency": params.currency}),
                )
                .await?;
            }
            EscrowStatus::OnHold => {
                enqueue_finance_event(
                    pool,
                    "payment_hold",
                    params.leg_id,
                    Some(escrow.id),
                    params.actor_user_id,
                    json!({"reason": params.note}),
                )
                .await?;
            }
            EscrowStatus::Released => {
                enqueue_finance_event(
                    pool,
                    "payment_released",
                    params.leg_id,
                    Some(escrow.id),
                    params.actor_user_id,
                    json!({"transfer_id": escrow.transfer_id}),
                )
                .await?;
                let _ =
                    ensure_settlement_ready_for_escrow(pool, escrow, params.actor_user_id).await?;
            }
            EscrowStatus::Failed => {
                enqueue_finance_event(
                    pool,
                    "payment_failed",
                    params.leg_id,
                    Some(escrow.id),
                    params.actor_user_id,
                    json!({
                        "payment_intent_id": escrow.payment_intent_id,
                        "charge_id": escrow.charge_id,
                        "reason": params.note,
                    }),
                )
                .await?;
            }
            _ => {}
        }
    }
    Ok(updated_escrow)
}

pub async fn record_stripe_webhook_once(
    pool: &DbPool,
    event_id: &str,
    event_type: &str,
    leg_id: Option<i64>,
    payment_intent_id: Option<&str>,
    stripe_account_id: Option<&str>,
    payload: &Value,
) -> Result<bool, sqlx::Error> {
    ensure_default_tenant(pool).await?;
    let inserted = sqlx::query_scalar::<_, i64>(
        "INSERT INTO stripe_webhook_events (
            tenant_id, event_id, event_type, leg_id, payment_intent_id, stripe_account_id, payload, processed_at
         ) VALUES ('default', $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP)
         ON CONFLICT (tenant_id, event_id) DO NOTHING
         RETURNING id",
    )
    .bind(event_id)
    .bind(event_type)
    .bind(leg_id)
    .bind(payment_intent_id)
    .bind(stripe_account_id)
    .bind(payload)
    .fetch_optional(pool)
    .await?;
    Ok(inserted.is_some())
}

pub async fn create_accessorial_request(
    pool: &DbPool,
    leg_id: i64,
    requested_by: Option<i64>,
    accessorial_type: &str,
    amount_cents: i64,
    currency: &str,
    reason: Option<&str>,
) -> Result<Option<PaymentWorkflowRecord>, sqlx::Error> {
    ensure_default_tenant(pool).await?;
    let Some(context) = load_leg_finance_context(pool, leg_id).await? else {
        return Ok(None);
    };
    let amount = cents_to_currency(amount_cents);
    let record = sqlx::query_as::<_, PaymentWorkflowRecord>(
        "INSERT INTO accessorial_requests (
            tenant_id, leg_id, requested_by, accessorial_type, amount, currency, status, reason, created_at, updated_at
         ) VALUES ('default', $1, $2, $3, $4, $5, 'pending', $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, status",
    )
    .bind(leg_id)
    .bind(requested_by)
    .bind(accessorial_type)
    .bind(amount)
    .bind(currency)
    .bind(reason)
    .fetch_one(pool)
    .await?;
    insert_payment_history(
        pool,
        context.load_id,
        requested_by,
        context.status_id,
        "Accessorial request opened from Rust payments.",
    )
    .await?;
    enqueue_finance_event(pool, "payment_hold", leg_id, None, requested_by, json!({"accessorial_request_id": record.id, "amount_cents": amount_cents, "type": accessorial_type})).await?;
    Ok(Some(record))
}

pub async fn create_payment_dispute(
    pool: &DbPool,
    leg_id: i64,
    opened_by: Option<i64>,
    dispute_type: &str,
    amount_cents: Option<i64>,
    detail: Option<&str>,
) -> Result<Option<PaymentWorkflowRecord>, sqlx::Error> {
    ensure_default_tenant(pool).await?;
    let Some(context) = load_leg_finance_context(pool, leg_id).await? else {
        return Ok(None);
    };
    let amount = amount_cents.map(cents_to_currency);
    let record = sqlx::query_as::<_, PaymentWorkflowRecord>(
        "INSERT INTO payment_disputes (
            tenant_id, leg_id, dispute_type, amount, status, opened_by, detail, created_at, updated_at
         ) VALUES ('default', $1, $2, $3, 'open', $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         RETURNING id, status",
    )
    .bind(leg_id)
    .bind(dispute_type)
    .bind(amount)
    .bind(opened_by)
    .bind(detail)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        "INSERT INTO payment_holds (tenant_id, leg_id, hold_reason, amount, status, created_at, updated_at)
         VALUES ('default', $1, $2, $3, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(leg_id)
    .bind(format!("dispute:{}", dispute_type))
    .bind(amount)
    .execute(pool)
    .await?;
    insert_payment_history(
        pool,
        context.load_id,
        opened_by,
        context.status_id,
        "Payment dispute opened from Rust payments.",
    )
    .await?;
    enqueue_finance_event(
        pool,
        "payment_hold",
        leg_id,
        None,
        opened_by,
        json!({"payment_dispute_id": record.id, "type": dispute_type}),
    )
    .await?;
    Ok(Some(record))
}

pub async fn has_active_finance_blocks(pool: &DbPool, leg_id: i64) -> Result<bool, sqlx::Error> {
    let accessorials = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM accessorial_requests WHERE tenant_id = 'default' AND leg_id = $1 AND status = 'pending'",
    )
    .bind(leg_id)
    .fetch_one(pool)
    .await?;
    let holds = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM payment_holds WHERE tenant_id = 'default' AND leg_id = $1 AND status = 'active'",
    )
    .bind(leg_id)
    .fetch_one(pool)
    .await?;
    let disputes = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM payment_disputes WHERE tenant_id = 'default' AND leg_id = $1 AND status = 'open'",
    )
    .bind(leg_id)
    .fetch_one(pool)
    .await?;
    Ok(accessorials + holds + disputes > 0)
}

pub async fn update_user_connect_state(
    pool: &DbPool,
    stripe_connect_account_id: &str,
    payouts_enabled: bool,
    kyc_status: Option<&str>,
) -> Result<Option<UserConnectStateRecord>, sqlx::Error> {
    sqlx::query(
        "UPDATE users
         SET payouts_enabled = $1,
             kyc_status = $2,
             status = CASE WHEN $1 = TRUE THEN 1 ELSE 3 END,
             updated_at = CURRENT_TIMESTAMP
         WHERE stripe_connect_account_id = $3",
    )
    .bind(payouts_enabled)
    .bind(kyc_status)
    .bind(stripe_connect_account_id)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, UserConnectStateRecord>(
        "SELECT id, stripe_connect_account_id, payouts_enabled, kyc_status, updated_at
         FROM users
         WHERE stripe_connect_account_id = $1
         LIMIT 1",
    )
    .bind(stripe_connect_account_id)
    .fetch_optional(pool)
    .await
}

pub async fn set_user_stripe_connect_account_id(
    pool: &DbPool,
    user_id: i64,
    stripe_connect_account_id: &str,
) -> Result<Option<UserConnectStateRecord>, sqlx::Error> {
    sqlx::query(
        "UPDATE users
         SET stripe_connect_account_id = $1,
             kyc_status = COALESCE(kyc_status, 'pending'),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(stripe_connect_account_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, UserConnectStateRecord>(
        "SELECT id, stripe_connect_account_id, payouts_enabled, kyc_status, updated_at
         FROM users
         WHERE id = $1
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn payments_contract_summary() -> PaymentsModuleContract {
    payments_module_contract()
}

pub async fn escrow_status_catalog() -> &'static [EscrowStatusDescriptor] {
    escrow_status_descriptors()
}

pub async fn stripe_webhook_event_catalog() -> &'static [StripeWebhookEventDescriptor] {
    stripe_webhook_events()
}

async fn ensure_settlement_ready_for_escrow(
    pool: &DbPool,
    escrow: &EscrowRecord,
    actor_user_id: Option<i64>,
) -> Result<Option<PaymentWorkflowRecord>, sqlx::Error> {
    if has_active_finance_blocks(pool, escrow.leg_id).await? {
        return Ok(None);
    }
    ensure_default_tenant(pool).await?;
    let settlement_number = format!("SET-LEG-{}-ESC-{}", escrow.leg_id, escrow.id);
    let gross_amount = cents_to_currency(escrow.amount);
    let deductions = cents_to_currency(escrow.platform_fee);
    let net_amount = cents_to_currency(escrow.amount.saturating_sub(escrow.platform_fee));
    let record = sqlx::query_as::<_, PaymentWorkflowRecord>(
        "INSERT INTO settlements (
            tenant_id, settlement_number, status, currency, gross_amount, deductions_amount,
            net_amount, leg_id, escrow_id, settlement_ready_at, created_at, updated_at
         ) VALUES (
            'default', $1, 'ready', $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (tenant_id, leg_id, escrow_id) WHERE leg_id IS NOT NULL AND escrow_id IS NOT NULL
         DO UPDATE SET
            status = CASE WHEN settlements.status = 'paid' THEN settlements.status ELSE 'ready' END,
            gross_amount = EXCLUDED.gross_amount,
            deductions_amount = EXCLUDED.deductions_amount,
            net_amount = EXCLUDED.net_amount,
            settlement_ready_at = COALESCE(settlements.settlement_ready_at, CURRENT_TIMESTAMP),
            updated_at = CURRENT_TIMESTAMP
         RETURNING id, status",
    )
    .bind(&settlement_number)
    .bind(&escrow.currency)
    .bind(gross_amount)
    .bind(deductions)
    .bind(net_amount)
    .bind(escrow.leg_id)
    .bind(escrow.id)
    .fetch_one(pool)
    .await?;
    enqueue_finance_event(
        pool,
        "settlement_ready",
        escrow.leg_id,
        Some(escrow.id),
        actor_user_id,
        json!({"settlement_id": record.id, "net_amount_cents": escrow.amount.saturating_sub(escrow.platform_fee)}),
    )
    .await?;
    Ok(Some(record))
}

async fn load_leg_finance_context(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<LoadLegFinanceContext>, sqlx::Error> {
    sqlx::query_as::<_, LoadLegFinanceContext>(
        "SELECT load_id, status_id FROM load_legs WHERE deleted_at IS NULL AND id = $1 LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await
}

async fn insert_payment_history(
    pool: &DbPool,
    load_id: i64,
    actor_user_id: Option<i64>,
    status_id: i16,
    note: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO load_history (load_id, admin_id, status, remarks, created_at, updated_at)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(load_id)
    .bind(actor_user_id)
    .bind(status_id)
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_default_tenant(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at, updated_at)
         VALUES ('default', 'Default STLoads Tenant', 'default', 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn enqueue_finance_event(
    pool: &DbPool,
    event_type: &str,
    leg_id: i64,
    escrow_id: Option<i64>,
    actor_user_id: Option<i64>,
    payload: Value,
) -> Result<(), sqlx::Error> {
    enqueue_atmp_outbound_event(
        pool,
        EnqueueAtmpOutboundEvent {
            tenant_id: "default",
            event_type,
            posting_id: None,
            booking_award_id: None,
            target_url: None,
            payload: json!({
                "event_type": event_type,
                "leg_id": leg_id,
                "escrow_id": escrow_id,
                "actor_user_id": actor_user_id,
                "details": payload,
            }),
            correlation_id: None,
        },
    )
    .await
    .map(|_| ())
}

fn cents_to_currency(cents: i64) -> f64 {
    cents as f64 / 100.0
}
