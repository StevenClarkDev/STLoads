use chrono::NaiveDateTime;
use domain::payments::{
    EscrowStatus, EscrowStatusDescriptor, PaymentsModuleContract, StripeWebhookEventDescriptor,
    escrow_status_descriptors, payments_module_contract, stripe_webhook_events,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentIdempotencyRecord {
    pub id: i64,
    pub idempotency_key: String,
    pub flow: String,
    pub leg_id: Option<i64>,
    pub actor_user_id: Option<i64>,
    pub request_fingerprint: String,
    pub response_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentLedgerEntryRecord {
    pub id: i64,
    pub source_event_key: String,
    pub entry_type: String,
    pub direction: String,
    pub currency: String,
    pub amount_cents: i64,
    pub platform_fee_cents: i64,
    pub load_id: Option<i64>,
    pub leg_id: Option<i64>,
    pub escrow_id: Option<i64>,
    pub payer_user_id: Option<i64>,
    pub payee_user_id: Option<i64>,
    pub actor_user_id: Option<i64>,
    pub audit_event_id: Option<i64>,
    pub transfer_group: Option<String>,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub transfer_id: Option<String>,
    pub stripe_refund_id: Option<String>,
    pub stripe_dispute_id: Option<String>,
    pub adjustment_reference: Option<String>,
    pub description: Option<String>,
    pub metadata: Value,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreatePaymentLedgerEntryParams<'a> {
    pub source_event_key: &'a str,
    pub entry_type: &'a str,
    pub direction: &'a str,
    pub currency: &'a str,
    pub amount_cents: i64,
    pub platform_fee_cents: i64,
    pub load_id: Option<i64>,
    pub leg_id: Option<i64>,
    pub escrow_id: Option<i64>,
    pub payer_user_id: Option<i64>,
    pub payee_user_id: Option<i64>,
    pub actor_user_id: Option<i64>,
    pub audit_event_id: Option<i64>,
    pub transfer_group: Option<&'a str>,
    pub payment_intent_id: Option<&'a str>,
    pub charge_id: Option<&'a str>,
    pub transfer_id: Option<&'a str>,
    pub stripe_refund_id: Option<&'a str>,
    pub stripe_dispute_id: Option<&'a str>,
    pub adjustment_reference: Option<&'a str>,
    pub description: Option<&'a str>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FinanceApprovalRequestRecord {
    pub id: i64,
    pub approval_type: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub organization_id: Option<i64>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub required_approval_count: i32,
    pub requested_by_user_id: Option<i64>,
    pub first_approved_by_user_id: Option<i64>,
    pub second_approved_by_user_id: Option<i64>,
    pub rejected_by_user_id: Option<i64>,
    pub reason: Option<String>,
    pub decision_note: Option<String>,
    pub requested_at: NaiveDateTime,
    pub first_approved_at: Option<NaiveDateTime>,
    pub second_approved_at: Option<NaiveDateTime>,
    pub rejected_at: Option<NaiveDateTime>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FinanceApprovalQueueRecord {
    pub id: i64,
    pub approval_type: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub organization_id: Option<i64>,
    pub load_id: Option<i64>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub required_approval_count: i32,
    pub requested_by_user_id: Option<i64>,
    pub first_approved_by_user_id: Option<i64>,
    pub second_approved_by_user_id: Option<i64>,
    pub reason: Option<String>,
    pub requested_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct FinanceApprovalRequestParams<'a> {
    pub approval_type: &'a str,
    pub entity_type: &'a str,
    pub entity_id: i64,
    pub organization_id: Option<i64>,
    pub amount_cents: i64,
    pub currency: &'a str,
    pub required_approval_count: i32,
    pub requested_by_user_id: Option<i64>,
    pub reason: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct FinanceApprovalDecisionParams<'a> {
    pub approval_type: &'a str,
    pub entity_type: &'a str,
    pub entity_id: i64,
    pub approver_user_id: i64,
    pub decision_note: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerInvoiceRecord {
    pub id: i64,
    pub invoice_number: String,
    pub load_id: i64,
    pub leg_id: i64,
    pub customer_user_id: Option<i64>,
    pub currency: String,
    pub freight_amount_cents: i64,
    pub accessorial_amount_cents: i64,
    pub tax_amount_cents: i64,
    pub adjustment_amount_cents: i64,
    pub total_amount_cents: i64,
    pub status: String,
    pub payment_terms_days: i32,
    pub issued_at: Option<NaiveDateTime>,
    pub due_at: Option<NaiveDateTime>,
    pub paid_at: Option<NaiveDateTime>,
    pub created_by_user_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarrierSettlementRecord {
    pub id: i64,
    pub settlement_number: String,
    pub load_id: i64,
    pub leg_id: i64,
    pub carrier_user_id: Option<i64>,
    pub currency: String,
    pub gross_amount_cents: i64,
    pub platform_fee_cents: i64,
    pub accessorial_amount_cents: i64,
    pub tax_withholding_cents: i64,
    pub adjustment_amount_cents: i64,
    pub net_amount_cents: i64,
    pub status: String,
    pub payment_terms_days: i32,
    pub approved_at: Option<NaiveDateTime>,
    pub released_at: Option<NaiveDateTime>,
    pub paid_at: Option<NaiveDateTime>,
    pub created_by_user_id: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSettlementPackage {
    pub invoice: CustomerInvoiceRecord,
    pub settlement: CarrierSettlementRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvoiceSettlementQueueRecord {
    pub invoice_id: i64,
    pub invoice_number: String,
    pub load_id: i64,
    pub leg_id: i64,
    pub customer_user_id: Option<i64>,
    pub invoice_currency: String,
    pub invoice_total_amount_cents: i64,
    pub invoice_adjustment_amount_cents: i64,
    pub invoice_status: String,
    pub invoice_due_at: Option<NaiveDateTime>,
    pub settlement_id: i64,
    pub settlement_number: String,
    pub carrier_user_id: Option<i64>,
    pub settlement_currency: String,
    pub settlement_gross_amount_cents: i64,
    pub settlement_platform_fee_cents: i64,
    pub settlement_adjustment_amount_cents: i64,
    pub settlement_net_amount_cents: i64,
    pub settlement_status: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountingExportRow {
    pub ledger_id: i64,
    pub created_at: NaiveDateTime,
    pub entry_type: String,
    pub direction: String,
    pub currency: String,
    pub amount_cents: i64,
    pub platform_fee_cents: i64,
    pub load_id: Option<i64>,
    pub leg_id: Option<i64>,
    pub escrow_id: Option<i64>,
    pub payment_intent_id: Option<String>,
    pub charge_id: Option<String>,
    pub transfer_id: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_status: Option<String>,
    pub settlement_number: Option<String>,
    pub settlement_status: Option<String>,
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
    pub stripe_refund_id: Option<&'a str>,
    pub stripe_dispute_id: Option<&'a str>,
    pub adjustment_reference: Option<&'a str>,
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

pub async fn find_escrow_for_leg_in_organization(
    pool: &DbPool,
    leg_id: i64,
    organization_id: i64,
) -> Result<Option<EscrowRecord>, sqlx::Error> {
    sqlx::query_as::<_, EscrowRecord>(
        "SELECT id, leg_id, payer_user_id, payee_user_id, currency, amount, platform_fee, status,
                transfer_group, payment_intent_id, charge_id, transfer_id, created_at, updated_at
         FROM escrows
         WHERE leg_id = $1 AND organization_id = $2
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(leg_id)
    .bind(organization_id)
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
    .bind(&history_note)
    .execute(&mut *tx)
    .await?;

    let audit_entity_id = params.leg_id.to_string();
    let audit_event_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO audit_events (
            actor_user_id, organization_id, target_organization_id, entity_type, entity_id,
            action, reason, ticket_ref, request_id, ip_address, user_agent, source, metadata,
            before_state, after_state, created_at
         ) VALUES (
            $1, NULL, NULL, 'load_leg', $2, $3, $4, NULL, NULL, NULL, NULL, 'rust-payments',
            $5, $6, $7, CURRENT_TIMESTAMP
         )
         RETURNING id",
    )
    .bind(params.actor_user_id)
    .bind(&audit_entity_id)
    .bind(format!(
        "payment_escrow_{}",
        params.status.as_legacy_label()
    ))
    .bind(&history_note)
    .bind(serde_json::json!({
        "load_id": finance_context.load_id,
        "leg_id": params.leg_id,
        "amount": params.amount,
        "platform_fee": params.platform_fee,
        "currency": params.currency,
        "payment_intent_id": resolved_payment_intent_id,
        "charge_id": resolved_charge_id,
        "transfer_id": resolved_transfer_id,
        "stripe_refund_id": params.stripe_refund_id,
        "stripe_dispute_id": params.stripe_dispute_id,
        "adjustment_reference": params.adjustment_reference,
    }))
    .bind(serde_json::json!({
        "escrow_status": existing_escrow.as_ref().map(|escrow| escrow.status.as_str()),
    }))
    .bind(serde_json::json!({
        "escrow_status": params.status.as_legacy_label(),
    }))
    .fetch_one(&mut *tx)
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

    if let Some(updated_escrow) = updated_escrow.as_ref() {
        record_escrow_transition_ledger_entries(
            &mut tx,
            &finance_context,
            updated_escrow,
            params.actor_user_id,
            params.status,
            Some(audit_event_id),
            params.stripe_refund_id,
            params.stripe_dispute_id,
            params.adjustment_reference,
        )
        .await?;
        if params.status == EscrowStatus::Released {
            upsert_invoice_and_settlement_tx(
                &mut tx,
                &finance_context,
                updated_escrow,
                params.actor_user_id,
            )
            .await?;
        }
    }

    tx.commit().await?;
    Ok(updated_escrow)
}

// Ledger rows must carry the full transition context; splitting these arguments
// would obscure the money-movement evidence this helper records.
#[allow(clippy::too_many_arguments)]
async fn record_escrow_transition_ledger_entries(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    finance_context: &LoadLegFinanceContext,
    escrow: &EscrowRecord,
    actor_user_id: Option<i64>,
    status: EscrowStatus,
    audit_event_id: Option<i64>,
    stripe_refund_id: Option<&str>,
    stripe_dispute_id: Option<&str>,
    adjustment_reference: Option<&str>,
) -> Result<(), sqlx::Error> {
    let event_anchor = escrow
        .transfer_id
        .as_deref()
        .or(stripe_refund_id)
        .or(stripe_dispute_id)
        .or(adjustment_reference)
        .or(escrow.payment_intent_id.as_deref())
        .or(escrow.charge_id.as_deref())
        .or(escrow.transfer_group.as_deref())
        .unwrap_or("manual");
    let metadata = serde_json::json!({
        "escrow_status": status.as_legacy_label(),
        "load_leg_status_id": finance_context.status_id,
    });

    match status {
        EscrowStatus::Funded => {
            insert_payment_ledger_entry_tx(
                tx,
                &CreatePaymentLedgerEntryParams {
                    source_event_key: &format!("escrow:{}:funded:{event_anchor}", escrow.id),
                    entry_type: "escrow_funded",
                    direction: "credit",
                    currency: &escrow.currency,
                    amount_cents: escrow.amount,
                    platform_fee_cents: escrow.platform_fee,
                    load_id: Some(finance_context.load_id),
                    leg_id: Some(escrow.leg_id),
                    escrow_id: Some(escrow.id),
                    payer_user_id: Some(escrow.payer_user_id),
                    payee_user_id: Some(escrow.payee_user_id),
                    actor_user_id,
                    audit_event_id,
                    transfer_group: escrow.transfer_group.as_deref(),
                    payment_intent_id: escrow.payment_intent_id.as_deref(),
                    charge_id: escrow.charge_id.as_deref(),
                    transfer_id: escrow.transfer_id.as_deref(),
                    stripe_refund_id,
                    stripe_dispute_id,
                    adjustment_reference,
                    description: Some("Escrow funded"),
                    metadata,
                },
            )
            .await?;
        }
        EscrowStatus::OnHold => {
            insert_payment_ledger_entry_tx(
                tx,
                &CreatePaymentLedgerEntryParams {
                    source_event_key: &format!("escrow:{}:hold:{event_anchor}", escrow.id),
                    entry_type: "escrow_hold",
                    direction: "hold",
                    currency: &escrow.currency,
                    amount_cents: escrow.amount,
                    platform_fee_cents: escrow.platform_fee,
                    load_id: Some(finance_context.load_id),
                    leg_id: Some(escrow.leg_id),
                    escrow_id: Some(escrow.id),
                    payer_user_id: Some(escrow.payer_user_id),
                    payee_user_id: Some(escrow.payee_user_id),
                    actor_user_id,
                    audit_event_id,
                    transfer_group: escrow.transfer_group.as_deref(),
                    payment_intent_id: escrow.payment_intent_id.as_deref(),
                    charge_id: escrow.charge_id.as_deref(),
                    transfer_id: escrow.transfer_id.as_deref(),
                    stripe_refund_id,
                    stripe_dispute_id,
                    adjustment_reference,
                    description: Some("Escrow placed on hold"),
                    metadata,
                },
            )
            .await?;
        }
        EscrowStatus::Released => {
            let carrier_amount = escrow.amount.saturating_sub(escrow.platform_fee);
            insert_payment_ledger_entry_tx(
                tx,
                &CreatePaymentLedgerEntryParams {
                    source_event_key: &format!("escrow:{}:transfer:{event_anchor}", escrow.id),
                    entry_type: "carrier_transfer",
                    direction: "debit",
                    currency: &escrow.currency,
                    amount_cents: carrier_amount,
                    platform_fee_cents: escrow.platform_fee,
                    load_id: Some(finance_context.load_id),
                    leg_id: Some(escrow.leg_id),
                    escrow_id: Some(escrow.id),
                    payer_user_id: Some(escrow.payer_user_id),
                    payee_user_id: Some(escrow.payee_user_id),
                    actor_user_id,
                    audit_event_id,
                    transfer_group: escrow.transfer_group.as_deref(),
                    payment_intent_id: escrow.payment_intent_id.as_deref(),
                    charge_id: escrow.charge_id.as_deref(),
                    transfer_id: escrow.transfer_id.as_deref(),
                    stripe_refund_id,
                    stripe_dispute_id,
                    adjustment_reference,
                    description: Some("Carrier payout transfer released"),
                    metadata: metadata.clone(),
                },
            )
            .await?;

            if escrow.platform_fee > 0 {
                insert_payment_ledger_entry_tx(
                    tx,
                    &CreatePaymentLedgerEntryParams {
                        source_event_key: &format!("escrow:{}:fee:{event_anchor}", escrow.id),
                        entry_type: "fee_earned",
                        direction: "credit",
                        currency: &escrow.currency,
                        amount_cents: escrow.platform_fee,
                        platform_fee_cents: escrow.platform_fee,
                        load_id: Some(finance_context.load_id),
                        leg_id: Some(escrow.leg_id),
                        escrow_id: Some(escrow.id),
                        payer_user_id: Some(escrow.payer_user_id),
                        payee_user_id: Some(escrow.payee_user_id),
                        actor_user_id,
                        audit_event_id,
                        transfer_group: escrow.transfer_group.as_deref(),
                        payment_intent_id: escrow.payment_intent_id.as_deref(),
                        charge_id: escrow.charge_id.as_deref(),
                        transfer_id: escrow.transfer_id.as_deref(),
                        stripe_refund_id,
                        stripe_dispute_id,
                        adjustment_reference,
                        description: Some("Platform fee earned"),
                        metadata,
                    },
                )
                .await?;
            }
        }
        EscrowStatus::Refunded => {
            insert_payment_ledger_entry_tx(
                tx,
                &CreatePaymentLedgerEntryParams {
                    source_event_key: &format!("escrow:{}:refund:{event_anchor}", escrow.id),
                    entry_type: "refund",
                    direction: "debit",
                    currency: &escrow.currency,
                    amount_cents: escrow.amount,
                    platform_fee_cents: escrow.platform_fee,
                    load_id: Some(finance_context.load_id),
                    leg_id: Some(escrow.leg_id),
                    escrow_id: Some(escrow.id),
                    payer_user_id: Some(escrow.payer_user_id),
                    payee_user_id: Some(escrow.payee_user_id),
                    actor_user_id,
                    audit_event_id,
                    transfer_group: escrow.transfer_group.as_deref(),
                    payment_intent_id: escrow.payment_intent_id.as_deref(),
                    charge_id: escrow.charge_id.as_deref(),
                    transfer_id: escrow.transfer_id.as_deref(),
                    stripe_refund_id,
                    stripe_dispute_id,
                    adjustment_reference,
                    description: Some("Escrow refunded"),
                    metadata,
                },
            )
            .await?;
        }
        EscrowStatus::Failed => {
            insert_payment_ledger_entry_tx(
                tx,
                &CreatePaymentLedgerEntryParams {
                    source_event_key: &format!(
                        "escrow:{}:payout_failure:{event_anchor}",
                        escrow.id
                    ),
                    entry_type: "payout_failure",
                    direction: "info",
                    currency: &escrow.currency,
                    amount_cents: escrow.amount,
                    platform_fee_cents: escrow.platform_fee,
                    load_id: Some(finance_context.load_id),
                    leg_id: Some(escrow.leg_id),
                    escrow_id: Some(escrow.id),
                    payer_user_id: Some(escrow.payer_user_id),
                    payee_user_id: Some(escrow.payee_user_id),
                    actor_user_id,
                    audit_event_id,
                    transfer_group: escrow.transfer_group.as_deref(),
                    payment_intent_id: escrow.payment_intent_id.as_deref(),
                    charge_id: escrow.charge_id.as_deref(),
                    transfer_id: escrow.transfer_id.as_deref(),
                    stripe_refund_id,
                    stripe_dispute_id,
                    adjustment_reference,
                    description: Some("Payment or payout failure recorded"),
                    metadata,
                },
            )
            .await?;
        }
        EscrowStatus::Unfunded => {}
    }

    Ok(())
}

async fn insert_payment_ledger_entry_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    params: &CreatePaymentLedgerEntryParams<'_>,
) -> Result<PaymentLedgerEntryRecord, sqlx::Error> {
    sqlx::query_as::<_, PaymentLedgerEntryRecord>(
        "INSERT INTO payment_ledger_entries (
             source_event_key, entry_type, direction, currency, amount_cents, platform_fee_cents,
             load_id, leg_id, escrow_id, payer_user_id, payee_user_id, actor_user_id,
             audit_event_id, transfer_group, payment_intent_id, charge_id, transfer_id,
             stripe_refund_id, stripe_dispute_id, adjustment_reference, description, metadata,
             created_at
         )
         VALUES (
             $1, $2, $3, $4, $5, $6,
             $7, $8, $9, $10, $11, $12,
             $13, $14, $15, $16, $17,
             $18, $19, $20, $21, $22,
             CURRENT_TIMESTAMP
         )
         ON CONFLICT (source_event_key) DO UPDATE SET source_event_key = payment_ledger_entries.source_event_key
         RETURNING id, source_event_key, entry_type, direction, currency, amount_cents, platform_fee_cents,
             load_id, leg_id, escrow_id, payer_user_id, payee_user_id, actor_user_id, audit_event_id,
             transfer_group, payment_intent_id, charge_id, transfer_id, stripe_refund_id,
             stripe_dispute_id, adjustment_reference, description, metadata, created_at",
    )
    .bind(params.source_event_key)
    .bind(params.entry_type)
    .bind(params.direction)
    .bind(params.currency)
    .bind(params.amount_cents)
    .bind(params.platform_fee_cents)
    .bind(params.load_id)
    .bind(params.leg_id)
    .bind(params.escrow_id)
    .bind(params.payer_user_id)
    .bind(params.payee_user_id)
    .bind(params.actor_user_id)
    .bind(params.audit_event_id)
    .bind(params.transfer_group)
    .bind(params.payment_intent_id)
    .bind(params.charge_id)
    .bind(params.transfer_id)
    .bind(params.stripe_refund_id)
    .bind(params.stripe_dispute_id)
    .bind(params.adjustment_reference)
    .bind(params.description)
    .bind(&params.metadata)
    .fetch_one(&mut **tx)
    .await
}

pub async fn record_payment_ledger_entry(
    pool: &DbPool,
    params: CreatePaymentLedgerEntryParams<'_>,
) -> Result<PaymentLedgerEntryRecord, sqlx::Error> {
    sqlx::query_as::<_, PaymentLedgerEntryRecord>(
        "INSERT INTO payment_ledger_entries (
             source_event_key, entry_type, direction, currency, amount_cents, platform_fee_cents,
             load_id, leg_id, escrow_id, payer_user_id, payee_user_id, actor_user_id,
             audit_event_id, transfer_group, payment_intent_id, charge_id, transfer_id,
             stripe_refund_id, stripe_dispute_id, adjustment_reference, description, metadata,
             created_at
         )
         VALUES (
             $1, $2, $3, $4, $5, $6,
             $7, $8, $9, $10, $11, $12,
             $13, $14, $15, $16, $17,
             $18, $19, $20, $21, $22,
             CURRENT_TIMESTAMP
         )
         ON CONFLICT (source_event_key) DO UPDATE SET source_event_key = payment_ledger_entries.source_event_key
         RETURNING id, source_event_key, entry_type, direction, currency, amount_cents, platform_fee_cents,
             load_id, leg_id, escrow_id, payer_user_id, payee_user_id, actor_user_id, audit_event_id,
             transfer_group, payment_intent_id, charge_id, transfer_id, stripe_refund_id,
             stripe_dispute_id, adjustment_reference, description, metadata, created_at",
    )
    .bind(params.source_event_key)
    .bind(params.entry_type)
    .bind(params.direction)
    .bind(params.currency)
    .bind(params.amount_cents)
    .bind(params.platform_fee_cents)
    .bind(params.load_id)
    .bind(params.leg_id)
    .bind(params.escrow_id)
    .bind(params.payer_user_id)
    .bind(params.payee_user_id)
    .bind(params.actor_user_id)
    .bind(params.audit_event_id)
    .bind(params.transfer_group)
    .bind(params.payment_intent_id)
    .bind(params.charge_id)
    .bind(params.transfer_id)
    .bind(params.stripe_refund_id)
    .bind(params.stripe_dispute_id)
    .bind(params.adjustment_reference)
    .bind(params.description)
    .bind(&params.metadata)
    .fetch_one(pool)
    .await
}

pub async fn list_payment_ledger_entries_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Vec<PaymentLedgerEntryRecord>, sqlx::Error> {
    sqlx::query_as::<_, PaymentLedgerEntryRecord>(
        "SELECT id, source_event_key, entry_type, direction, currency, amount_cents, platform_fee_cents,
             load_id, leg_id, escrow_id, payer_user_id, payee_user_id, actor_user_id, audit_event_id,
             transfer_group, payment_intent_id, charge_id, transfer_id, stripe_refund_id,
             stripe_dispute_id, adjustment_reference, description, metadata, created_at
         FROM payment_ledger_entries
         WHERE leg_id = $1
         ORDER BY id",
    )
    .bind(leg_id)
    .fetch_all(pool)
    .await
}

pub async fn ensure_finance_approval_request(
    pool: &DbPool,
    params: FinanceApprovalRequestParams<'_>,
) -> Result<FinanceApprovalRequestRecord, sqlx::Error> {
    sqlx::query_as::<_, FinanceApprovalRequestRecord>(
        "INSERT INTO finance_approval_requests (
             approval_type, entity_type, entity_id, organization_id, amount_cents, currency,
             required_approval_count, requested_by_user_id, reason, status, requested_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (approval_type, entity_type, entity_id) WHERE status = 'pending'
         DO UPDATE SET
             amount_cents = EXCLUDED.amount_cents,
             currency = EXCLUDED.currency,
             required_approval_count = EXCLUDED.required_approval_count,
             reason = COALESCE(finance_approval_requests.reason, EXCLUDED.reason),
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, approval_type, entity_type, entity_id, organization_id, amount_cents, currency,
             status, required_approval_count, requested_by_user_id, first_approved_by_user_id,
             second_approved_by_user_id, rejected_by_user_id, reason, decision_note, requested_at,
             first_approved_at, second_approved_at, rejected_at, updated_at",
    )
    .bind(params.approval_type)
    .bind(params.entity_type)
    .bind(params.entity_id)
    .bind(params.organization_id)
    .bind(params.amount_cents)
    .bind(params.currency)
    .bind(params.required_approval_count)
    .bind(params.requested_by_user_id)
    .bind(params.reason)
    .fetch_one(pool)
    .await
}

pub async fn approve_finance_request(
    pool: &DbPool,
    params: FinanceApprovalDecisionParams<'_>,
) -> Result<Option<FinanceApprovalRequestRecord>, sqlx::Error> {
    let existing = find_open_finance_approval_request(
        pool,
        params.approval_type,
        params.entity_type,
        params.entity_id,
    )
    .await?;
    let Some(existing) = existing else {
        return Ok(None);
    };

    if existing.first_approved_by_user_id == Some(params.approver_user_id)
        || existing.second_approved_by_user_id == Some(params.approver_user_id)
    {
        return Ok(Some(existing));
    }

    let approve_second = existing.first_approved_by_user_id.is_some();
    sqlx::query_as::<_, FinanceApprovalRequestRecord>(
        "UPDATE finance_approval_requests
         SET first_approved_by_user_id = CASE WHEN $1 = FALSE THEN $2 ELSE first_approved_by_user_id END,
             first_approved_at = CASE WHEN $1 = FALSE THEN CURRENT_TIMESTAMP ELSE first_approved_at END,
             second_approved_by_user_id = CASE WHEN $1 = TRUE THEN $2 ELSE second_approved_by_user_id END,
             second_approved_at = CASE WHEN $1 = TRUE THEN CURRENT_TIMESTAMP ELSE second_approved_at END,
             decision_note = COALESCE($3, decision_note),
             status = CASE
                 WHEN (CASE WHEN $1 = FALSE THEN 1 ELSE 0 END)
                    + (CASE WHEN first_approved_by_user_id IS NOT NULL THEN 1 ELSE 0 END)
                    + (CASE WHEN $1 = TRUE THEN 1 ELSE 0 END)
                    + (CASE WHEN second_approved_by_user_id IS NOT NULL THEN 1 ELSE 0 END)
                    >= required_approval_count
                 THEN 'approved'
                 ELSE status
             END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4
         RETURNING id, approval_type, entity_type, entity_id, organization_id, amount_cents, currency,
             status, required_approval_count, requested_by_user_id, first_approved_by_user_id,
             second_approved_by_user_id, rejected_by_user_id, reason, decision_note, requested_at,
             first_approved_at, second_approved_at, rejected_at, updated_at",
    )
    .bind(approve_second)
    .bind(params.approver_user_id)
    .bind(params.decision_note)
    .bind(existing.id)
    .fetch_optional(pool)
    .await
}

pub async fn release_has_required_finance_approval(
    pool: &DbPool,
    leg_id: i64,
    required_approval_count: i32,
) -> Result<bool, sqlx::Error> {
    finance_request_has_required_approval(
        pool,
        "escrow_release",
        "load_leg",
        leg_id,
        required_approval_count,
    )
    .await
}

pub async fn finance_request_has_required_approval(
    pool: &DbPool,
    approval_type: &str,
    entity_type: &str,
    entity_id: i64,
    required_approval_count: i32,
) -> Result<bool, sqlx::Error> {
    let approved_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM finance_approval_requests
         WHERE approval_type = $1
           AND entity_type = $2
           AND entity_id = $3
           AND status = 'approved'
           AND required_approval_count >= $4",
    )
    .bind(approval_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(required_approval_count)
    .fetch_one(pool)
    .await?;

    Ok(approved_count > 0)
}

pub async fn find_open_finance_approval_request(
    pool: &DbPool,
    approval_type: &str,
    entity_type: &str,
    entity_id: i64,
) -> Result<Option<FinanceApprovalRequestRecord>, sqlx::Error> {
    sqlx::query_as::<_, FinanceApprovalRequestRecord>(
        "SELECT id, approval_type, entity_type, entity_id, organization_id, amount_cents, currency,
             status, required_approval_count, requested_by_user_id, first_approved_by_user_id,
             second_approved_by_user_id, rejected_by_user_id, reason, decision_note, requested_at,
             first_approved_at, second_approved_at, rejected_at, updated_at
         FROM finance_approval_requests
         WHERE approval_type = $1
           AND entity_type = $2
           AND entity_id = $3
           AND status = 'pending'
         ORDER BY id DESC
         LIMIT 1",
    )
    .bind(approval_type)
    .bind(entity_type)
    .bind(entity_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_pending_finance_release_approvals(
    pool: &DbPool,
    organization_id: Option<i64>,
    limit: i64,
) -> Result<Vec<FinanceApprovalQueueRecord>, sqlx::Error> {
    sqlx::query_as::<_, FinanceApprovalQueueRecord>(
        "SELECT
             approval.id,
             approval.approval_type,
             approval.entity_type,
             approval.entity_id,
             approval.organization_id,
             leg.load_id,
             approval.amount_cents,
             approval.currency,
             approval.status,
             approval.required_approval_count,
             approval.requested_by_user_id,
             approval.first_approved_by_user_id,
             approval.second_approved_by_user_id,
             approval.reason,
             approval.requested_at,
             approval.updated_at
         FROM finance_approval_requests approval
         LEFT JOIN load_legs leg ON leg.id = approval.entity_id AND approval.entity_type = 'load_leg'
         WHERE approval.approval_type IN ('escrow_release', 'escrow_hold')
           AND approval.status = 'pending'
           AND ($1::BIGINT IS NULL OR approval.organization_id = $1)
         ORDER BY approval.updated_at DESC, approval.id DESC
         LIMIT $2",
    )
    .bind(organization_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await
}

async fn upsert_invoice_and_settlement_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    finance_context: &LoadLegFinanceContext,
    escrow: &EscrowRecord,
    created_by_user_id: Option<i64>,
) -> Result<InvoiceSettlementPackage, sqlx::Error> {
    let carrier_net_amount = escrow.amount.saturating_sub(escrow.platform_fee);
    let invoice = sqlx::query_as::<_, CustomerInvoiceRecord>(
        "INSERT INTO customer_invoices (
             invoice_number, load_id, leg_id, customer_user_id, currency,
             freight_amount_cents, accessorial_amount_cents, tax_amount_cents,
             adjustment_amount_cents, total_amount_cents, status, payment_terms_days,
             issued_at, due_at, created_by_user_id, created_at, updated_at
         )
         VALUES (
             $1, $2, $3, $4, $5,
             $6, 0, 0, 0, $6, 'issued', 30,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '30 days', $7,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (leg_id) DO UPDATE SET
             freight_amount_cents = EXCLUDED.freight_amount_cents,
             total_amount_cents = EXCLUDED.total_amount_cents,
             status = CASE WHEN customer_invoices.status = 'draft' THEN 'issued' ELSE customer_invoices.status END,
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, invoice_number, load_id, leg_id, customer_user_id, currency,
             freight_amount_cents, accessorial_amount_cents, tax_amount_cents,
             adjustment_amount_cents, total_amount_cents, status, payment_terms_days,
             issued_at, due_at, paid_at, created_by_user_id, created_at, updated_at",
    )
    .bind(format!("INV-{}", escrow.leg_id))
    .bind(finance_context.load_id)
    .bind(escrow.leg_id)
    .bind(escrow.payer_user_id)
    .bind(&escrow.currency)
    .bind(escrow.amount)
    .bind(created_by_user_id)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO customer_invoice_lines (
             invoice_id, line_type, description, quantity, unit_amount_cents, amount_cents, created_at
         )
         VALUES ($1, 'freight', 'Linehaul freight charge', 1, $2, $2, CURRENT_TIMESTAMP)
         ON CONFLICT DO NOTHING",
    )
    .bind(invoice.id)
    .bind(escrow.amount)
    .execute(&mut **tx)
    .await?;

    let settlement = sqlx::query_as::<_, CarrierSettlementRecord>(
        "INSERT INTO carrier_settlements (
             settlement_number, load_id, leg_id, carrier_user_id, currency,
             gross_amount_cents, platform_fee_cents, accessorial_amount_cents,
             tax_withholding_cents, adjustment_amount_cents, net_amount_cents,
             status, payment_terms_days, approved_at, released_at, created_by_user_id,
             created_at, updated_at
         )
         VALUES (
             $1, $2, $3, $4, $5,
             $6, $7, 0, 0, 0, $8,
             'released', 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, $9,
             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
         )
         ON CONFLICT (leg_id) DO UPDATE SET
             gross_amount_cents = EXCLUDED.gross_amount_cents,
             platform_fee_cents = EXCLUDED.platform_fee_cents,
             net_amount_cents = EXCLUDED.net_amount_cents,
             status = CASE WHEN carrier_settlements.status IN ('pending', 'approved') THEN 'released' ELSE carrier_settlements.status END,
             released_at = COALESCE(carrier_settlements.released_at, CURRENT_TIMESTAMP),
             updated_at = CURRENT_TIMESTAMP
         RETURNING id, settlement_number, load_id, leg_id, carrier_user_id, currency,
             gross_amount_cents, platform_fee_cents, accessorial_amount_cents,
             tax_withholding_cents, adjustment_amount_cents, net_amount_cents,
             status, payment_terms_days, approved_at, released_at, paid_at,
             created_by_user_id, created_at, updated_at",
    )
    .bind(format!("SET-{}", escrow.leg_id))
    .bind(finance_context.load_id)
    .bind(escrow.leg_id)
    .bind(escrow.payee_user_id)
    .bind(&escrow.currency)
    .bind(escrow.amount)
    .bind(escrow.platform_fee)
    .bind(carrier_net_amount)
    .bind(created_by_user_id)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO carrier_settlement_lines (
             settlement_id, line_type, description, quantity, unit_amount_cents, amount_cents, created_at
         )
         VALUES
             ($1, 'gross_freight', 'Gross freight settlement', 1, $2, $2, CURRENT_TIMESTAMP),
             ($1, 'platform_fee', 'Platform fee deducted from settlement', 1, $3, -$3, CURRENT_TIMESTAMP)
         ON CONFLICT DO NOTHING",
    )
    .bind(settlement.id)
    .bind(escrow.amount)
    .bind(escrow.platform_fee)
    .execute(&mut **tx)
    .await?;

    Ok(InvoiceSettlementPackage {
        invoice,
        settlement,
    })
}

pub async fn find_invoice_settlement_for_leg(
    pool: &DbPool,
    leg_id: i64,
) -> Result<Option<InvoiceSettlementPackage>, sqlx::Error> {
    let Some(invoice) = sqlx::query_as::<_, CustomerInvoiceRecord>(
        "SELECT id, invoice_number, load_id, leg_id, customer_user_id, currency,
             freight_amount_cents, accessorial_amount_cents, tax_amount_cents,
             adjustment_amount_cents, total_amount_cents, status, payment_terms_days,
             issued_at, due_at, paid_at, created_by_user_id, created_at, updated_at
         FROM customer_invoices
         WHERE leg_id = $1
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let settlement = sqlx::query_as::<_, CarrierSettlementRecord>(
        "SELECT id, settlement_number, load_id, leg_id, carrier_user_id, currency,
             gross_amount_cents, platform_fee_cents, accessorial_amount_cents,
             tax_withholding_cents, adjustment_amount_cents, net_amount_cents,
             status, payment_terms_days, approved_at, released_at, paid_at,
             created_by_user_id, created_at, updated_at
         FROM carrier_settlements
         WHERE leg_id = $1
         LIMIT 1",
    )
    .bind(leg_id)
    .fetch_one(pool)
    .await?;

    Ok(Some(InvoiceSettlementPackage {
        invoice,
        settlement,
    }))
}

pub async fn list_invoice_settlement_queue(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<InvoiceSettlementQueueRecord>, sqlx::Error> {
    sqlx::query_as::<_, InvoiceSettlementQueueRecord>(
        "SELECT
             invoice.id AS invoice_id,
             invoice.invoice_number,
             invoice.load_id,
             invoice.leg_id,
             invoice.customer_user_id,
             invoice.currency AS invoice_currency,
             invoice.total_amount_cents AS invoice_total_amount_cents,
             invoice.adjustment_amount_cents AS invoice_adjustment_amount_cents,
             invoice.status AS invoice_status,
             invoice.due_at AS invoice_due_at,
             settlement.id AS settlement_id,
             settlement.settlement_number,
             settlement.carrier_user_id,
             settlement.currency AS settlement_currency,
             settlement.gross_amount_cents AS settlement_gross_amount_cents,
             settlement.platform_fee_cents AS settlement_platform_fee_cents,
             settlement.adjustment_amount_cents AS settlement_adjustment_amount_cents,
             settlement.net_amount_cents AS settlement_net_amount_cents,
             settlement.status AS settlement_status,
             GREATEST(invoice.updated_at, settlement.updated_at) AS updated_at
         FROM customer_invoices invoice
         INNER JOIN carrier_settlements settlement ON settlement.leg_id = invoice.leg_id
         ORDER BY GREATEST(invoice.updated_at, settlement.updated_at) DESC, invoice.id DESC
         LIMIT $1",
    )
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await
}

pub async fn apply_invoice_settlement_adjustment(
    pool: &DbPool,
    leg_id: i64,
    adjustment_amount_cents: i64,
    note: Option<&str>,
) -> Result<Option<InvoiceSettlementPackage>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let Some(invoice) = sqlx::query_as::<_, CustomerInvoiceRecord>(
        "UPDATE customer_invoices
         SET adjustment_amount_cents = adjustment_amount_cents + $2,
             total_amount_cents = total_amount_cents + $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE leg_id = $1
           AND status IN ('draft', 'issued')
         RETURNING id, invoice_number, load_id, leg_id, customer_user_id, currency,
             freight_amount_cents, accessorial_amount_cents, tax_amount_cents,
             adjustment_amount_cents, total_amount_cents, status, payment_terms_days,
             issued_at, due_at, paid_at, created_by_user_id, created_at, updated_at",
    )
    .bind(leg_id)
    .bind(adjustment_amount_cents)
    .fetch_optional(&mut *tx)
    .await?
    else {
        tx.rollback().await?;
        return Ok(None);
    };

    sqlx::query(
        "INSERT INTO customer_invoice_lines (
             invoice_id, line_type, description, quantity, unit_amount_cents, amount_cents, created_at
         )
         VALUES ($1, $2, $3, 1, $4, $4, CURRENT_TIMESTAMP)
         ON CONFLICT (invoice_id, line_type) DO UPDATE SET
             description = EXCLUDED.description,
             unit_amount_cents = customer_invoice_lines.unit_amount_cents + EXCLUDED.unit_amount_cents,
             amount_cents = customer_invoice_lines.amount_cents + EXCLUDED.amount_cents",
    )
    .bind(invoice.id)
    .bind("finance_adjustment")
    .bind(note.unwrap_or("Finance adjustment"))
    .bind(adjustment_amount_cents)
    .execute(&mut *tx)
    .await?;

    let settlement = sqlx::query_as::<_, CarrierSettlementRecord>(
        "UPDATE carrier_settlements
         SET adjustment_amount_cents = adjustment_amount_cents + $2,
             net_amount_cents = net_amount_cents + $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE leg_id = $1
           AND status IN ('pending', 'approved', 'released')
         RETURNING id, settlement_number, load_id, leg_id, carrier_user_id, currency,
             gross_amount_cents, platform_fee_cents, accessorial_amount_cents,
             tax_withholding_cents, adjustment_amount_cents, net_amount_cents,
             status, payment_terms_days, approved_at, released_at, paid_at,
             created_by_user_id, created_at, updated_at",
    )
    .bind(leg_id)
    .bind(adjustment_amount_cents)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO carrier_settlement_lines (
             settlement_id, line_type, description, quantity, unit_amount_cents, amount_cents, created_at
         )
         VALUES ($1, $2, $3, 1, $4, $4, CURRENT_TIMESTAMP)
         ON CONFLICT (settlement_id, line_type) DO UPDATE SET
             description = EXCLUDED.description,
             unit_amount_cents = carrier_settlement_lines.unit_amount_cents + EXCLUDED.unit_amount_cents,
             amount_cents = carrier_settlement_lines.amount_cents + EXCLUDED.amount_cents",
    )
    .bind(settlement.id)
    .bind("finance_adjustment")
    .bind(note.unwrap_or("Finance adjustment"))
    .bind(adjustment_amount_cents)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(InvoiceSettlementPackage {
        invoice,
        settlement,
    }))
}

pub async fn accounting_export_rows(
    pool: &DbPool,
    limit: i64,
) -> Result<Vec<AccountingExportRow>, sqlx::Error> {
    sqlx::query_as::<_, AccountingExportRow>(
        "SELECT
             ledger.id AS ledger_id,
             ledger.created_at,
             ledger.entry_type,
             ledger.direction,
             ledger.currency,
             ledger.amount_cents,
             ledger.platform_fee_cents,
             ledger.load_id,
             ledger.leg_id,
             ledger.escrow_id,
             ledger.payment_intent_id,
             ledger.charge_id,
             ledger.transfer_id,
             invoice.invoice_number,
             invoice.status AS invoice_status,
             settlement.settlement_number,
             settlement.status AS settlement_status
         FROM payment_ledger_entries ledger
         LEFT JOIN customer_invoices invoice ON invoice.leg_id = ledger.leg_id
         LEFT JOIN carrier_settlements settlement ON settlement.leg_id = ledger.leg_id
         ORDER BY ledger.created_at DESC, ledger.id DESC
         LIMIT $1",
    )
    .bind(limit.clamp(1, 10_000))
    .fetch_all(pool)
    .await
}

pub async fn find_payment_idempotency_record(
    pool: &DbPool,
    flow: &str,
    idempotency_key: &str,
) -> Result<Option<PaymentIdempotencyRecord>, sqlx::Error> {
    sqlx::query_as::<_, PaymentIdempotencyRecord>(
        "SELECT id, idempotency_key, flow, leg_id, actor_user_id, request_fingerprint, response_json
         FROM payment_idempotency_keys
         WHERE flow = $1 AND idempotency_key = $2
         LIMIT 1",
    )
    .bind(flow)
    .bind(idempotency_key)
    .fetch_optional(pool)
    .await
}

pub async fn record_payment_idempotency_response(
    pool: &DbPool,
    flow: &str,
    idempotency_key: &str,
    leg_id: Option<i64>,
    actor_user_id: Option<i64>,
    request_fingerprint: &str,
    response_json: &Value,
) -> Result<PaymentIdempotencyRecord, sqlx::Error> {
    sqlx::query_as::<_, PaymentIdempotencyRecord>(
        "INSERT INTO payment_idempotency_keys (
             idempotency_key, flow, leg_id, actor_user_id, request_fingerprint,
             response_json, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (flow, idempotency_key)
         DO UPDATE SET updated_at = payment_idempotency_keys.updated_at
         RETURNING id, idempotency_key, flow, leg_id, actor_user_id, request_fingerprint, response_json",
    )
    .bind(idempotency_key)
    .bind(flow)
    .bind(leg_id)
    .bind(actor_user_id)
    .bind(request_fingerprint)
    .bind(response_json)
    .fetch_one(pool)
    .await
}

pub async fn claim_stripe_webhook_event(
    pool: &DbPool,
    stripe_event_id: &str,
    event_type: &str,
    payment_intent_id: Option<&str>,
    stripe_account_id: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let inserted_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO payment_stripe_webhook_events (
             stripe_event_id, event_type, payment_intent_id, stripe_account_id,
             processing_status, created_at, updated_at
         )
         VALUES ($1, $2, $3, $4, 'processing', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
         ON CONFLICT (stripe_event_id) DO NOTHING
         RETURNING id",
    )
    .bind(stripe_event_id)
    .bind(event_type)
    .bind(payment_intent_id)
    .bind(stripe_account_id)
    .fetch_optional(pool)
    .await?;

    Ok(inserted_id.is_some())
}

pub async fn complete_stripe_webhook_event(
    pool: &DbPool,
    stripe_event_id: &str,
    processing_status: &str,
    response_json: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE payment_stripe_webhook_events
         SET processing_status = $1,
             response_json = $2,
             updated_at = CURRENT_TIMESTAMP
         WHERE stripe_event_id = $3",
    )
    .bind(processing_status)
    .bind(response_json)
    .bind(stripe_event_id)
    .execute(pool)
    .await?;

    Ok(())
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
