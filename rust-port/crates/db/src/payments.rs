use chrono::NaiveDateTime;
use domain::payments::{
    EscrowStatus, EscrowStatusDescriptor, PaymentsModuleContract, StripeWebhookEventDescriptor,
    escrow_status_descriptors, payments_module_contract, stripe_webhook_events,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::DbPool;

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
    Ok(updated_escrow)
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
