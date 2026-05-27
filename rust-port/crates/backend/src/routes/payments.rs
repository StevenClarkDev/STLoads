use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use std::time::Duration as StdDuration;

use db::{
    audit::{AuditEventInput, insert_audit_event},
    auth::find_user_by_id,
    dispatch::{find_load_leg_by_id, find_load_leg_scope},
    payments::{
        AccountingExportRow, CreatePaymentLedgerEntryParams, EscrowTransitionParams,
        FinanceApprovalDecisionParams, FinanceApprovalRequestParams, accounting_export_rows,
        apply_escrow_transition, apply_invoice_settlement_adjustment, approve_finance_request,
        claim_stripe_webhook_event, ensure_finance_approval_request,
        finance_request_has_required_approval, find_escrow_by_payment_intent_id,
        find_escrow_for_leg, find_payment_idempotency_record, list_invoice_settlement_queue,
        list_pending_finance_release_approvals, record_payment_idempotency_response,
        record_payment_ledger_entry, release_has_required_finance_approval,
        set_user_stripe_connect_account_id, update_user_connect_state,
    },
    tracking::execution_closeout_readiness,
};
use domain::{
    auth::UserRole,
    payments::{
        EscrowStatus, EscrowStatusDescriptor, PaymentsModuleContract, StripeWebhookEventDescriptor,
        escrow_status_descriptors, payments_module_contract, stripe_webhook_events,
    },
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use shared::{
    ApiResponse, EscrowFundRequest, EscrowHoldRequest, EscrowLifecycleResponse,
    EscrowReleaseRequest, RealtimeEvent, RealtimeEventKind, RealtimeTopic, StripeWebhookRequest,
    StripeWebhookResponse,
};
use sqlx::Row;
use tracing::warn;

use crate::{
    app::REQUEST_ID_HEADER, auth_session, auth_session::ResolvedSession,
    rate_limit::RateLimitPolicy, rate_limit::client_fingerprint, realtime_bus::RoutedRealtimeEvent,
    state::AppState,
};

fn payments_policy(name: &'static str) -> RateLimitPolicy {
    RateLimitPolicy::new(name, 30, StdDuration::from_secs(60 * 60))
}

const HIGH_VALUE_RELEASE_APPROVAL_CENTS: i64 = 500_000;
const HIGH_VALUE_RELEASE_REQUIRED_APPROVALS: i32 = 2;
const MANUAL_HOLD_REQUIRED_APPROVALS: i32 = 1;

fn webhook_policy(name: &'static str) -> RateLimitPolicy {
    RateLimitPolicy::new(name, 120, StdDuration::from_secs(60))
}

fn rate_limit_message(flow: &str, retry_after_seconds: u64) -> String {
    format!(
        "Too many {} attempts. Wait about {} seconds before trying again.",
        flow, retry_after_seconds
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn payment_idempotency_key(headers: &HeaderMap, payload_key: Option<&str>) -> Option<String> {
    payload_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            headers
                .get("idempotency-key")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

fn payment_request_fingerprint<T: Serialize>(flow: &str, leg_id: i64, payload: &T) -> String {
    use sha2::Digest;

    let raw = serde_json::to_string(payload).unwrap_or_default();
    let digest = Sha256::digest(format!("{flow}:{leg_id}:{raw}").as_bytes());
    digest.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn effective_payment_idempotency_key(
    provided_key: Option<String>,
    flow: &str,
    leg_id: i64,
    actor_user_id: i64,
    request_fingerprint: &str,
) -> String {
    provided_key.unwrap_or_else(|| {
        format!(
            "auto:{}:{}:{}:{}",
            flow, leg_id, actor_user_id, request_fingerprint
        )
    })
}

fn finance_approval_count(record: &db::payments::FinanceApprovalRequestRecord) -> i32 {
    let mut count = 0;
    if record.first_approved_by_user_id.is_some() {
        count += 1;
    }
    if record.second_approved_by_user_id.is_some() {
        count += 1;
    }
    count
}

fn can_perform_finance_action(session: &ResolvedSession) -> bool {
    let has_finance_permission =
        session.session.permissions.iter().any(|permission| {
            permission == "manage_payments" || permission == "access_admin_portal"
        });
    let has_mfa = session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "mfa_verified");
    has_finance_permission && has_mfa
}

fn can_view_finance(session: &ResolvedSession) -> bool {
    session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "manage_payments" || permission == "access_admin_portal")
}

// Manual finance events carry idempotency, Stripe references, direction, amount,
// and operator notes as one audited action.
#[allow(clippy::too_many_arguments)]
async fn record_manual_finance_ledger_event(
    state: AppState,
    headers: HeaderMap,
    leg_id: i64,
    flow: &str,
    entry_type: &str,
    direction: &str,
    amount_cents: i64,
    adjustment_reference: Option<&str>,
    stripe_dispute_id: Option<&str>,
    note: Option<&str>,
    provided_idempotency_key: Option<&str>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, entry_type,
        )));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Sign in with finance access before recording this payment event.",
        )));
    };
    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Payment finance events require manage_payments or admin access with MFA step-up.",
        )));
    }
    if amount_cents <= 0 {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Invalid Amount".into(),
            message: "Finance event amount must be positive cents.".into(),
        }));
    }
    if !matches!(direction, "debit" | "credit" | "hold" | "release" | "info") {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Invalid Direction".into(),
            message: "Finance event direction must be debit, credit, hold, release, or info."
                .into(),
        }));
    }

    let fingerprint_payload = json!({
        "entry_type": entry_type,
        "direction": direction,
        "amount_cents": amount_cents,
        "adjustment_reference": adjustment_reference,
        "stripe_dispute_id": stripe_dispute_id,
        "note": note,
    });
    let request_fingerprint = payment_request_fingerprint(flow, leg_id, &fingerprint_payload);
    let idempotency_key = effective_payment_idempotency_key(
        provided_idempotency_key.map(str::to_string),
        flow,
        leg_id,
        session.user.id,
        &request_fingerprint,
    );
    if let Some(replay) =
        replay_payment_idempotency(pool, flow, &idempotency_key, &request_fingerprint).await
    {
        return Json(ApiResponse::ok(replay));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };
    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only finance admins for this organization can record this payment event.",
        )));
    }
    let Some(existing_escrow) = find_escrow_for_leg(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Escrow".into(),
            message: "Escrow must exist before payment finance events can be recorded.".into(),
        }));
    };

    let reference = adjustment_reference
        .or(stripe_dispute_id)
        .unwrap_or(&idempotency_key);
    let source_event_key = format!("escrow:{}:{}:{}", existing_escrow.id, entry_type, reference);
    let audit_entity_id = leg_id.to_string();
    let request_id = request_id_from_headers(&headers);
    let audit_event_id = insert_audit_event(
        pool,
        &AuditEventInput {
            actor_user_id: Some(session.user.id),
            organization_id: Some(scope.organization_id),
            target_organization_id: Some(scope.organization_id),
            entity_type: "payment_ledger_entry",
            entity_id: Some(&audit_entity_id),
            action: entry_type,
            reason: note,
            ticket_ref: None,
            request_id: request_id.as_deref(),
            ip_address: None,
            user_agent: None,
            source: "rust-payments",
            metadata: Some(json!({
                "flow": flow,
                "direction": direction,
                "amount_cents": amount_cents,
                "adjustment_reference": adjustment_reference,
                "stripe_dispute_id": stripe_dispute_id,
            })),
            before_state: None,
            after_state: Some(json!({
                "source_event_key": source_event_key,
            })),
        },
    )
    .await
    .ok();
    let ledger_result = record_payment_ledger_entry(
        pool,
        CreatePaymentLedgerEntryParams {
            source_event_key: &source_event_key,
            entry_type,
            direction,
            currency: &existing_escrow.currency,
            amount_cents,
            platform_fee_cents: 0,
            load_id: Some(scope.load_id),
            leg_id: Some(leg_id),
            escrow_id: Some(existing_escrow.id),
            payer_user_id: Some(existing_escrow.payer_user_id),
            payee_user_id: Some(existing_escrow.payee_user_id),
            actor_user_id: Some(session.user.id),
            audit_event_id,
            transfer_group: existing_escrow.transfer_group.as_deref(),
            payment_intent_id: existing_escrow.payment_intent_id.as_deref(),
            charge_id: existing_escrow.charge_id.as_deref(),
            transfer_id: existing_escrow.transfer_id.as_deref(),
            stripe_refund_id: None,
            stripe_dispute_id,
            adjustment_reference,
            description: note.or(Some("Manual finance ledger event")),
            metadata: json!({
                "flow": flow,
                "recorded_by": session.user.id,
            }),
        },
    )
    .await;

    match ledger_result {
        Ok(entry) => {
            let response = EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(existing_escrow.id),
                payment_intent_id: existing_escrow.payment_intent_id,
                client_secret: None,
                transfer_id: existing_escrow.transfer_id,
                status_label: entry.entry_type,
                message: format!(
                    "{} ledger entry #{} recorded for {} cents.",
                    entry_type, entry.id, amount_cents
                ),
            };
            remember_payment_idempotency(
                pool,
                flow,
                &idempotency_key,
                leg_id,
                Some(session.user.id),
                &request_fingerprint,
                &response,
            )
            .await;
            Json(ApiResponse::ok(response))
        }
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: Some(existing_escrow.id),
            payment_intent_id: existing_escrow.payment_intent_id,
            client_secret: None,
            transfer_id: existing_escrow.transfer_id,
            status_label: "Ledger Error".into(),
            message: format!("Finance ledger event could not be recorded: {}", error),
        })),
    }
}

async fn replay_payment_idempotency(
    pool: &db::DbPool,
    flow: &str,
    idempotency_key: &str,
    request_fingerprint: &str,
) -> Option<EscrowLifecycleResponse> {
    let record = find_payment_idempotency_record(pool, flow, idempotency_key)
        .await
        .ok()
        .flatten()?;
    if record.request_fingerprint != request_fingerprint {
        return Some(EscrowLifecycleResponse {
            success: false,
            leg_id: record.leg_id.unwrap_or_default(),
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Idempotency Conflict".into(),
            message: "This idempotency key was already used with a different payment request body."
                .into(),
        });
    }

    serde_json::from_value::<EscrowLifecycleResponse>(record.response_json).ok()
}

async fn remember_payment_idempotency(
    pool: &db::DbPool,
    flow: &str,
    idempotency_key: &str,
    leg_id: i64,
    actor_user_id: Option<i64>,
    request_fingerprint: &str,
    response: &EscrowLifecycleResponse,
) {
    if !response.success {
        return;
    }
    let _ = record_payment_idempotency_response(
        pool,
        flow,
        idempotency_key,
        Some(leg_id),
        actor_user_id,
        request_fingerprint,
        &json!(response),
    )
    .await;
}

#[derive(Debug, Serialize)]
struct PaymentsOverview {
    contract: PaymentsModuleContract,
    escrow_statuses: usize,
    webhook_events: usize,
}

#[derive(Debug, Deserialize)]
struct StripeConnectLinkRequest {
    refresh_url: Option<String>,
    return_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct StripeConnectLinkResponse {
    success: bool,
    user_id: i64,
    account_id: Option<String>,
    onboarding_url: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct FinanceApprovalActionRequest {
    note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EscrowRefundRequest {
    refund_id: Option<String>,
    note: Option<String>,
    idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PaymentAdjustmentRequest {
    amount_cents: i64,
    direction: String,
    adjustment_reference: Option<String>,
    note: Option<String>,
    idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PaymentDisputeRequest {
    amount_cents: i64,
    stripe_dispute_id: Option<String>,
    note: Option<String>,
    idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct FinanceApprovalActionResponse {
    success: bool,
    leg_id: i64,
    approval_id: Option<i64>,
    status: String,
    required_approval_count: i32,
    approval_count: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct FinanceApprovalQueueItem {
    approval_id: i64,
    approval_type: String,
    leg_id: i64,
    load_id: Option<i64>,
    amount_cents: i64,
    currency: String,
    status: String,
    required_approval_count: i32,
    approval_count: i32,
    requested_by_user_id: Option<i64>,
    first_approved_by_user_id: Option<i64>,
    second_approved_by_user_id: Option<i64>,
    reason: Option<String>,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct FinanceApprovalQueueResponse {
    approvals: Vec<FinanceApprovalQueueItem>,
    message: String,
}

#[derive(Debug, Serialize)]
struct InvoiceSettlementQueueItem {
    invoice_id: i64,
    invoice_number: String,
    settlement_id: i64,
    settlement_number: String,
    load_id: i64,
    leg_id: i64,
    customer_user_id: Option<i64>,
    carrier_user_id: Option<i64>,
    currency: String,
    invoice_total_amount_cents: i64,
    invoice_adjustment_amount_cents: i64,
    invoice_status: String,
    settlement_gross_amount_cents: i64,
    settlement_platform_fee_cents: i64,
    settlement_adjustment_amount_cents: i64,
    settlement_net_amount_cents: i64,
    settlement_status: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct InvoiceSettlementQueueResponse {
    rows: Vec<InvoiceSettlementQueueItem>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct AccountingExportQuery {
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
struct AccountingExportResponse {
    filename: String,
    content_type: String,
    row_count: u64,
    csv: String,
}

#[derive(Debug, Serialize)]
struct CarrierFinanceOption {
    key: &'static str,
    label: &'static str,
    supported: bool,
    status: &'static str,
    message: &'static str,
}

#[derive(Debug, Serialize)]
struct CarrierFinanceOptionsResponse {
    supported_option_count: usize,
    options: Vec<CarrierFinanceOption>,
    decision: &'static str,
}

#[derive(Debug, Serialize)]
struct PlatformBillingModelResponse {
    commercial_model: &'static str,
    freight_money_separation: &'static str,
    supported_records: Vec<&'static str>,
    deferred_integrations: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct PayoutChangeControlsResponse {
    controls: Vec<&'static str>,
    release_policy: &'static str,
}

#[derive(Debug, Serialize)]
struct PlatformBillingRow {
    billing_account_id: i64,
    organization_id: Option<i64>,
    customer_user_id: Option<i64>,
    plan_name: Option<String>,
    billing_status: String,
    payment_method_status: String,
    latest_invoice_id: Option<i64>,
    latest_invoice_number: Option<String>,
    latest_invoice_status: Option<String>,
    open_invoice_cents: i64,
    past_due_invoice_cents: i64,
}

#[derive(Debug, Serialize)]
struct PlatformBillingScreenResponse {
    rows: Vec<PlatformBillingRow>,
    message: String,
}

#[derive(Debug, Serialize)]
struct FinanceMutationResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct ShipperCreditRow {
    credit_account_id: i64,
    organization_id: Option<i64>,
    customer_user_id: Option<i64>,
    credit_status: String,
    credit_limit_cents: i64,
    open_ar_cents: i64,
    overdue_ar_cents: i64,
    payment_terms_days: i32,
    credit_hold: bool,
    override_required: bool,
    internal_risk_note: Option<String>,
}

#[derive(Debug, Serialize)]
struct ShipperCreditScreenResponse {
    rows: Vec<ShipperCreditRow>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct CreditOverrideRequest {
    credit_account_id: i64,
    reason: String,
}

#[derive(Debug, Serialize)]
struct PayoutReviewRow {
    review_id: i64,
    carrier_user_id: i64,
    stripe_connect_account_id: Option<String>,
    change_type: String,
    risk_status: String,
    cooling_off_until: Option<String>,
    notification_sent_at: Option<String>,
    review_note: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct PayoutReviewQueueResponse {
    rows: Vec<PayoutReviewRow>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct PayoutReviewDecisionRequest {
    decision: String,
    note: Option<String>,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/contract", get(contract))
        .route("/escrow-statuses", get(escrow_statuses))
        .route("/webhook-events", get(webhook_events))
        .route("/legs/{leg_id}/fund", post(fund_leg_escrow))
        .route("/legs/{leg_id}/hold", post(hold_leg_escrow))
        .route("/legs/{leg_id}/hold-approval", post(approve_hold_escrow))
        .route("/legs/{leg_id}/release", post(release_leg_escrow))
        .route("/legs/{leg_id}/refund", post(refund_leg_escrow))
        .route("/legs/{leg_id}/adjustment", post(record_payment_adjustment))
        .route("/legs/{leg_id}/dispute", post(record_payment_dispute))
        .route(
            "/legs/{leg_id}/release-approval",
            post(approve_release_escrow),
        )
        .route("/release-approvals", get(release_approval_queue))
        .route("/finance-approvals", get(release_approval_queue))
        .route("/invoice-settlements", get(invoice_settlement_queue))
        .route("/accounting/export", get(accounting_export))
        .route("/carrier-finance/options", get(carrier_finance_options))
        .route("/platform-billing/model", get(platform_billing_model))
        .route("/platform-billing/accounts", get(platform_billing_accounts))
        .route(
            "/platform-billing/generate-invoices",
            post(generate_platform_billing_invoices),
        )
        .route(
            "/platform-billing/invoices/{invoice_id}/mark-paid",
            post(mark_platform_invoice_paid),
        )
        .route("/shipper-credit/accounts", get(shipper_credit_accounts))
        .route("/shipper-credit/override", post(approve_credit_override))
        .route("/payout-change-controls", get(payout_change_controls))
        .route("/payout-change-reviews", get(payout_change_reviews))
        .route(
            "/payout-change-reviews/{review_id}/decision",
            post(decide_payout_change_review),
        )
        .route("/connect/onboarding-link", post(connect_onboarding_link))
        .route(
            "/connect/users/{user_id}/onboarding-link",
            post(admin_connect_onboarding_link),
        )
        .route("/webhooks/stripe", post(stripe_webhook))
}

async fn index() -> Json<ApiResponse<PaymentsOverview>> {
    let contract = payments_module_contract();
    Json(ApiResponse::ok(PaymentsOverview {
        escrow_statuses: escrow_status_descriptors().len(),
        webhook_events: contract.webhook_events.len(),
        contract,
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("payments route group ready"))
}

async fn contract() -> Json<ApiResponse<PaymentsModuleContract>> {
    Json(ApiResponse::ok(payments_module_contract()))
}

async fn escrow_statuses() -> Json<ApiResponse<Vec<EscrowStatusDescriptor>>> {
    Json(ApiResponse::ok(escrow_status_descriptors().to_vec()))
}

async fn webhook_events() -> Json<ApiResponse<Vec<StripeWebhookEventDescriptor>>> {
    Json(ApiResponse::ok(stripe_webhook_events().to_vec()))
}

async fn accounting_export(
    State(state): State<AppState>,
    Query(query): Query<AccountingExportQuery>,
    headers: HeaderMap,
) -> Json<ApiResponse<AccountingExportResponse>> {
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(AccountingExportResponse {
            filename: "accounting-export-unauthorized.csv".into(),
            content_type: "text/csv".into(),
            row_count: 0,
            csv: accounting_export_header(),
        }));
    };
    if !session.session.permissions.iter().any(|permission| {
        permission == "manage_payments"
            || permission == "access_admin_portal"
            || permission == "view_audit_events"
    }) {
        return Json(ApiResponse::ok(AccountingExportResponse {
            filename: "accounting-export-forbidden.csv".into(),
            content_type: "text/csv".into(),
            row_count: 0,
            csv: accounting_export_header(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(AccountingExportResponse {
            filename: "accounting-export-unavailable.csv".into(),
            content_type: "text/csv".into(),
            row_count: 0,
            csv: accounting_export_header(),
        }));
    };

    match accounting_export_rows(pool, query.limit.unwrap_or(1_000)).await {
        Ok(rows) => Json(ApiResponse::ok(AccountingExportResponse {
            filename: "accounting-export.csv".into(),
            content_type: "text/csv".into(),
            row_count: rows.len() as u64,
            csv: accounting_rows_to_csv(&rows),
        })),
        Err(error) => Json(ApiResponse::ok(AccountingExportResponse {
            filename: "accounting-export-error.csv".into(),
            content_type: "text/csv".into(),
            row_count: 0,
            csv: format!(
                "{}\n\"export_error\",\"{}\"",
                accounting_export_header(),
                csv_escape(&error.to_string())
            ),
        })),
    }
}

async fn carrier_finance_options() -> Json<ApiResponse<CarrierFinanceOptionsResponse>> {
    let options = vec![
        CarrierFinanceOption {
            key: "factoring",
            label: "Factoring",
            supported: false,
            status: "deferred",
            message: "Factoring is deferred until STLoads has an approved finance partner, eligibility policy, fee schedule, and repayment reconciliation workflow.",
        },
        CarrierFinanceOption {
            key: "quick_pay",
            label: "Quick Pay",
            supported: false,
            status: "deferred",
            message: "Quick Pay is deferred until Finance approves pricing, risk controls, settlement treatment, and carrier disclosures.",
        },
        CarrierFinanceOption {
            key: "fuel_advance",
            label: "Fuel Advance",
            supported: false,
            status: "deferred",
            message: "Fuel advances are deferred until eligibility, repayment, fraud checks, and settlement offsets are implemented.",
        },
        CarrierFinanceOption {
            key: "fuel_card",
            label: "Fuel Card",
            supported: false,
            status: "deferred",
            message: "Fuel cards are deferred until provider integration, spend controls, exception handling, and reconciliation are approved.",
        },
        CarrierFinanceOption {
            key: "carrier_advance",
            label: "Carrier Advance",
            supported: false,
            status: "deferred",
            message: "Carrier advances are deferred until advance limits, approval workflow, audit evidence, and settlement repayment are implemented.",
        },
    ];
    Json(ApiResponse::ok(CarrierFinanceOptionsResponse {
        supported_option_count: options.iter().filter(|option| option.supported).count(),
        options,
        decision: "No carrier finance products are enabled in the first enterprise release.",
    }))
}

async fn platform_billing_model() -> Json<ApiResponse<PlatformBillingModelResponse>> {
    Json(ApiResponse::ok(PlatformBillingModelResponse {
        commercial_model: "hybrid_subscription_usage",
        freight_money_separation: "STLoads platform subscription and usage billing is separate from freight escrow, shipper invoices, and carrier settlements.",
        supported_records: vec![
            "subscription plans",
            "billing accounts",
            "usage events",
            "platform billing status",
            "payment method status",
        ],
        deferred_integrations: vec![
            "automatic card collection",
            "subscription invoice payment collection",
            "external billing-provider sync",
        ],
    }))
}

async fn platform_billing_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<PlatformBillingScreenResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(PlatformBillingScreenResponse {
            rows: Vec::new(),
            message: "Platform billing requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(PlatformBillingScreenResponse {
            rows: Vec::new(),
            message: "Sign in with finance permissions before viewing platform billing.".into(),
        }));
    };
    if !can_view_finance(&session) {
        return Json(ApiResponse::ok(PlatformBillingScreenResponse {
            rows: Vec::new(),
            message: "Platform billing requires manage_payments or admin access.".into(),
        }));
    }
    let org_id = auth_session::session_organization_id(&session);
    let rows = sqlx::query(
        "SELECT account.id AS billing_account_id,
                account.organization_id,
                account.customer_user_id,
                plan.plan_name,
                account.billing_status,
                account.payment_method_status,
                latest_invoice.id AS latest_invoice_id,
                latest_invoice.invoice_number AS latest_invoice_number,
                latest_invoice.status AS latest_invoice_status,
                COALESCE(SUM(CASE WHEN invoice.status IN ('issued', 'past_due') THEN invoice.total_amount_cents ELSE 0 END), 0)::BIGINT AS open_invoice_cents,
                COALESCE(SUM(CASE WHEN invoice.status = 'past_due' OR (invoice.status = 'issued' AND invoice.due_at < CURRENT_TIMESTAMP) THEN invoice.total_amount_cents ELSE 0 END), 0)::BIGINT AS past_due_invoice_cents
         FROM stloads_billing_accounts account
         LEFT JOIN stloads_subscription_plans plan ON plan.id = account.plan_id
         LEFT JOIN stloads_platform_invoices invoice ON invoice.billing_account_id = account.id
         LEFT JOIN LATERAL (
             SELECT id, invoice_number, status
             FROM stloads_platform_invoices latest
             WHERE latest.billing_account_id = account.id
             ORDER BY latest.issued_at DESC, latest.id DESC
             LIMIT 1
         ) latest_invoice ON TRUE
         WHERE ($1::BIGINT IS NULL OR account.organization_id = $1)
         GROUP BY account.id, account.organization_id, account.customer_user_id, plan.plan_name,
                  account.billing_status, account.payment_method_status, latest_invoice.id,
                  latest_invoice.invoice_number, latest_invoice.status
         ORDER BY account.updated_at DESC, account.id DESC
         LIMIT 100",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => Json(ApiResponse::ok(PlatformBillingScreenResponse {
            message: format!("{} platform billing account(s).", rows.len()),
            rows: rows
                .into_iter()
                .map(|row| PlatformBillingRow {
                    billing_account_id: row.get("billing_account_id"),
                    organization_id: row.get("organization_id"),
                    customer_user_id: row.get("customer_user_id"),
                    plan_name: row.get("plan_name"),
                    billing_status: row.get("billing_status"),
                    payment_method_status: row.get("payment_method_status"),
                    latest_invoice_id: row.get("latest_invoice_id"),
                    latest_invoice_number: row.get("latest_invoice_number"),
                    latest_invoice_status: row.get("latest_invoice_status"),
                    open_invoice_cents: row.get("open_invoice_cents"),
                    past_due_invoice_cents: row.get("past_due_invoice_cents"),
                })
                .collect(),
        })),
        Err(error) => Json(ApiResponse::ok(PlatformBillingScreenResponse {
            rows: Vec::new(),
            message: format!("Platform billing could not load: {}", error),
        })),
    }
}

async fn generate_platform_billing_invoices(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<FinanceMutationResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Platform invoice generation requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Sign in with finance permissions before generating platform invoices.".into(),
        }));
    };
    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Platform invoice generation requires finance/admin access with MFA.".into(),
        }));
    }
    let inserted = sqlx::query_scalar::<_, i64>(
        "WITH usage_totals AS (
             SELECT billing_account_id, COALESCE(SUM(quantity), 0)::BIGINT AS usage_quantity
             FROM stloads_usage_events
             WHERE billing_account_id IS NOT NULL
             GROUP BY billing_account_id
         ), candidates AS (
             SELECT account.id AS billing_account_id,
                    account.organization_id,
                    COALESCE(plan.monthly_base_cents, 0)::BIGINT AS base_amount_cents,
                    (COALESCE(plan.per_load_cents, 0) * COALESCE(usage.usage_quantity, 0))::BIGINT AS usage_amount_cents
             FROM stloads_billing_accounts account
             LEFT JOIN stloads_subscription_plans plan ON plan.id = account.plan_id
             LEFT JOIN usage_totals usage ON usage.billing_account_id = account.id
             WHERE account.billing_status IN ('active', 'trialing', 'past_due')
         ), inserted AS (
             INSERT INTO stloads_platform_invoices (
                 billing_account_id, organization_id, invoice_number, currency,
                 base_amount_cents, usage_amount_cents, total_amount_cents, status,
                 issued_at, due_at, created_at, updated_at
             )
             SELECT billing_account_id, organization_id, CONCAT('PLAT-', billing_account_id, '-', TO_CHAR(CURRENT_DATE, 'YYYYMMDD')),
                    'USD', base_amount_cents, usage_amount_cents, base_amount_cents + usage_amount_cents,
                    'issued', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP + INTERVAL '30 days', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
             FROM candidates
             WHERE base_amount_cents + usage_amount_cents > 0
             ON CONFLICT (invoice_number) DO NOTHING
             RETURNING id
         )
         SELECT COUNT(*)::BIGINT FROM inserted",
    )
    .fetch_one(pool)
    .await;
    match inserted {
        Ok(count) => Json(ApiResponse::ok(FinanceMutationResponse {
            success: true,
            message: format!("Generated {} platform invoice(s).", count),
        })),
        Err(error) => Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: format!("Platform invoice generation failed: {}", error),
        })),
    }
}

async fn mark_platform_invoice_paid(
    State(state): State<AppState>,
    Path(invoice_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<FinanceMutationResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Platform invoice payment requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Sign in with finance permissions before marking invoices paid.".into(),
        }));
    };
    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Marking platform invoices paid requires finance/admin access with MFA."
                .into(),
        }));
    }
    match sqlx::query(
        "UPDATE stloads_platform_invoices
         SET status = 'paid', paid_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(invoice_id)
    .execute(pool)
    .await
    {
        Ok(result) => Json(ApiResponse::ok(FinanceMutationResponse {
            success: result.rows_affected() > 0,
            message: format!(
                "Marked {} platform invoice(s) paid.",
                result.rows_affected()
            ),
        })),
        Err(error) => Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: format!("Platform invoice payment failed: {}", error),
        })),
    }
}

async fn payout_change_controls() -> Json<ApiResponse<PayoutChangeControlsResponse>> {
    Json(ApiResponse::ok(PayoutChangeControlsResponse {
        controls: vec![
            "Stripe Connect remains the approved payout account verification provider for the first enterprise release.",
            "Payout destination change reviews are persisted with cooling-off and finance review states.",
            "Suspicious bank, email, phone, or ownership changes must create a finance review before payout release policy can be relaxed.",
            "Returned or failed payouts must remain held until finance review is complete.",
        ],
        release_policy: "Payout destination changes cannot silently redirect carrier money; high-risk changes require review, notification, and cooling-off evidence.",
    }))
}

async fn shipper_credit_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<ShipperCreditScreenResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ShipperCreditScreenResponse {
            rows: Vec::new(),
            message: "Shipper credit controls require the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(ShipperCreditScreenResponse {
            rows: Vec::new(),
            message: "Sign in with finance permissions before viewing shipper credit.".into(),
        }));
    };
    if !can_view_finance(&session) {
        return Json(ApiResponse::ok(ShipperCreditScreenResponse {
            rows: Vec::new(),
            message: "Shipper credit controls require manage_payments or admin access.".into(),
        }));
    }
    let org_id = auth_session::session_organization_id(&session);
    let rows = sqlx::query(
        "WITH ar AS (
             SELECT customer_user_id,
                    COALESCE(SUM(CASE WHEN status IN ('issued', 'partially_paid') THEN total_amount_cents ELSE 0 END), 0)::BIGINT AS open_ar_cents,
                    COALESCE(SUM(CASE WHEN status IN ('issued', 'partially_paid') AND due_at < CURRENT_TIMESTAMP THEN total_amount_cents ELSE 0 END), 0)::BIGINT AS overdue_ar_cents
             FROM customer_invoices
             GROUP BY customer_user_id
         )
         SELECT credit.id AS credit_account_id,
                credit.organization_id,
                credit.customer_user_id,
                CASE
                    WHEN credit.credit_hold THEN 'hold'
                    WHEN COALESCE(ar.overdue_ar_cents, credit.overdue_ar_cents) > 0 THEN 'collections'
                    WHEN credit.credit_limit_cents > 0 AND COALESCE(ar.open_ar_cents, credit.open_ar_cents) > credit.credit_limit_cents THEN 'over_limit'
                    ELSE credit.credit_status
                END AS credit_status,
                credit.credit_limit_cents,
                COALESCE(ar.open_ar_cents, credit.open_ar_cents)::BIGINT AS open_ar_cents,
                COALESCE(ar.overdue_ar_cents, credit.overdue_ar_cents)::BIGINT AS overdue_ar_cents,
                credit.payment_terms_days,
                credit.credit_hold,
                credit.override_required,
                credit.internal_risk_note
         FROM shipper_credit_accounts credit
         LEFT JOIN ar ON ar.customer_user_id = credit.customer_user_id
         WHERE ($1::BIGINT IS NULL OR credit.organization_id = $1)
         ORDER BY credit.credit_hold DESC, overdue_ar_cents DESC, open_ar_cents DESC
         LIMIT 100",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => Json(ApiResponse::ok(ShipperCreditScreenResponse {
            message: format!("{} shipper credit account(s).", rows.len()),
            rows: rows
                .into_iter()
                .map(|row| ShipperCreditRow {
                    credit_account_id: row.get("credit_account_id"),
                    organization_id: row.get("organization_id"),
                    customer_user_id: row.get("customer_user_id"),
                    credit_status: row.get("credit_status"),
                    credit_limit_cents: row.get("credit_limit_cents"),
                    open_ar_cents: row.get("open_ar_cents"),
                    overdue_ar_cents: row.get("overdue_ar_cents"),
                    payment_terms_days: row.get("payment_terms_days"),
                    credit_hold: row.get("credit_hold"),
                    override_required: row.get("override_required"),
                    internal_risk_note: row.get("internal_risk_note"),
                })
                .collect(),
        })),
        Err(error) => Json(ApiResponse::ok(ShipperCreditScreenResponse {
            rows: Vec::new(),
            message: format!("Shipper credit controls could not load: {}", error),
        })),
    }
}

async fn approve_credit_override(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreditOverrideRequest>,
) -> Json<ApiResponse<FinanceMutationResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Credit override requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Sign in with finance permissions before approving credit override.".into(),
        }));
    };
    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Credit override requires finance/admin access with MFA.".into(),
        }));
    }
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(error) => {
            return Json(ApiResponse::ok(FinanceMutationResponse {
                success: false,
                message: format!("Credit override could not start: {}", error),
            }));
        }
    };
    let result = sqlx::query(
        "UPDATE shipper_credit_accounts
         SET credit_hold = FALSE,
             override_required = FALSE,
             credit_status = CASE WHEN credit_status = 'hold' THEN 'watch' ELSE credit_status END,
             internal_risk_note = COALESCE(internal_risk_note || E'\n', '') || $2,
             updated_by_user_id = $3,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1",
    )
    .bind(payload.credit_account_id)
    .bind(&payload.reason)
    .bind(session.user.id)
    .execute(&mut *tx)
    .await;
    if let Err(error) = result {
        let _ = tx.rollback().await;
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: format!("Credit override failed: {}", error),
        }));
    }
    let _ = sqlx::query(
        "INSERT INTO shipper_credit_override_requests (
            credit_account_id, requested_by_user_id, approved_by_user_id, status, reason,
            expires_at, created_at, approved_at, updated_at
         ) VALUES ($1, $2, $2, 'approved', $3, CURRENT_TIMESTAMP + INTERVAL '7 days',
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(payload.credit_account_id)
    .bind(session.user.id)
    .bind(&payload.reason)
    .execute(&mut *tx)
    .await;
    let _ = tx.commit().await;
    Json(ApiResponse::ok(FinanceMutationResponse {
        success: true,
        message: "Credit hold override approved and recorded.".into(),
    }))
}

async fn payout_change_reviews(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<PayoutReviewQueueResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(PayoutReviewQueueResponse {
            rows: Vec::new(),
            message: "Payout change review queue requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(PayoutReviewQueueResponse {
            rows: Vec::new(),
            message: "Sign in with finance permissions before viewing payout reviews.".into(),
        }));
    };
    if !can_view_finance(&session) {
        return Json(ApiResponse::ok(PayoutReviewQueueResponse {
            rows: Vec::new(),
            message: "Payout review queue requires manage_payments or admin access.".into(),
        }));
    }
    let rows = sqlx::query(
        "SELECT id, carrier_user_id, stripe_connect_account_id, change_type, risk_status,
                cooling_off_until, notification_sent_at, review_note, created_at
         FROM payout_destination_change_reviews
         WHERE risk_status IN ('review_required', 'cooling_off', 'blocked')
         ORDER BY created_at DESC
         LIMIT 100",
    )
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => Json(ApiResponse::ok(PayoutReviewQueueResponse {
            message: format!("{} payout review(s).", rows.len()),
            rows: rows
                .into_iter()
                .map(|row| PayoutReviewRow {
                    review_id: row.get("id"),
                    carrier_user_id: row.get("carrier_user_id"),
                    stripe_connect_account_id: row.get("stripe_connect_account_id"),
                    change_type: row.get("change_type"),
                    risk_status: row.get("risk_status"),
                    cooling_off_until: row
                        .get::<Option<chrono::NaiveDateTime>, _>("cooling_off_until")
                        .map(|value| value.to_string()),
                    notification_sent_at: row
                        .get::<Option<chrono::NaiveDateTime>, _>("notification_sent_at")
                        .map(|value| value.to_string()),
                    review_note: row.get("review_note"),
                    created_at: row
                        .get::<chrono::NaiveDateTime, _>("created_at")
                        .to_string(),
                })
                .collect(),
        })),
        Err(error) => Json(ApiResponse::ok(PayoutReviewQueueResponse {
            rows: Vec::new(),
            message: format!("Payout reviews could not load: {}", error),
        })),
    }
}

async fn decide_payout_change_review(
    State(state): State<AppState>,
    Path(review_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<PayoutReviewDecisionRequest>,
) -> Json<ApiResponse<FinanceMutationResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Payout review decision requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Sign in with finance permissions before deciding payout review.".into(),
        }));
    };
    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: "Payout review decision requires finance/admin access with MFA.".into(),
        }));
    }
    let status = match payload.decision.trim().to_ascii_lowercase().as_str() {
        "approve" | "approved" => "approved",
        "reject" | "rejected" => "rejected",
        "block" | "blocked" => "blocked",
        _ => "review_required",
    };
    match sqlx::query(
        "UPDATE payout_destination_change_reviews
         SET risk_status = $1,
             reviewed_by_user_id = $2,
             review_note = $3,
             reviewed_at = CURRENT_TIMESTAMP,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $4",
    )
    .bind(status)
    .bind(session.user.id)
    .bind(payload.note.as_deref())
    .bind(review_id)
    .execute(pool)
    .await
    {
        Ok(result) => Json(ApiResponse::ok(FinanceMutationResponse {
            success: result.rows_affected() > 0,
            message: format!("Payout review marked {}.", status),
        })),
        Err(error) => Json(ApiResponse::ok(FinanceMutationResponse {
            success: false,
            message: format!("Payout review decision failed: {}", error),
        })),
    }
}

async fn connect_onboarding_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<StripeConnectLinkRequest>,
) -> Json<ApiResponse<StripeConnectLinkResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: 0,
            account_id: None,
            onboarding_url: None,
            message: format!(
                "Stripe Connect onboarding is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let Some(session) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: 0,
            account_id: None,
            onboarding_url: None,
            message: "Sign in as a carrier before creating a Stripe Connect onboarding link."
                .into(),
        }));
    };
    let rate_decision = state
        .check_rate_limit(
            payments_policy("stripe_connect_onboarding"),
            format!("carrier:{}", session.user.id),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: session.user.id,
            account_id: session.user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: rate_limit_message(
                "Stripe Connect onboarding",
                rate_decision.retry_after_seconds,
            ),
        }));
    }

    if session.user.primary_role() != Some(UserRole::Carrier) {
        return Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id: session.user.id,
            account_id: session.user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: "Only carrier accounts can start Stripe Connect payout onboarding.".into(),
        }));
    }

    Json(ApiResponse::ok(
        create_or_refresh_connect_link(
            &state,
            pool,
            &session.user,
            &payload,
            request_id_from_headers(&headers).as_deref(),
        )
        .await,
    ))
}

async fn admin_connect_onboarding_link(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<StripeConnectLinkRequest>,
) -> Result<Json<ApiResponse<StripeConnectLinkResponse>>, StatusCode> {
    let _session = resolve_payments_session(&state, &headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter(|session| session.user.primary_role() == Some(UserRole::Admin))
        .ok_or(StatusCode::FORBIDDEN)?;

    let rate_decision = state
        .check_rate_limit(
            payments_policy("admin_stripe_connect_onboarding"),
            format!("admin-target:{user_id}"),
        )
        .await;
    if !rate_decision.allowed {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: None,
            onboarding_url: None,
            message: rate_limit_message(
                "admin Stripe Connect onboarding",
                rate_decision.retry_after_seconds,
            ),
        })));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: None,
            onboarding_url: None,
            message: format!(
                "Stripe Connect onboarding is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: None,
            onboarding_url: None,
            message: "The selected carrier account was not found.".into(),
        })));
    };

    if user.primary_role() != Some(UserRole::Carrier) {
        return Ok(Json(ApiResponse::ok(StripeConnectLinkResponse {
            success: false,
            user_id,
            account_id: user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: "Only carrier accounts can receive Stripe Connect payout onboarding links."
                .into(),
        })));
    }

    Ok(Json(ApiResponse::ok(
        create_or_refresh_connect_link(
            &state,
            pool,
            &user,
            &payload,
            request_id_from_headers(&headers).as_deref(),
        )
        .await,
    )))
}

async fn fund_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowFundRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let request_id = request_id_from_headers(&headers);
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Funding",
        )));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Unauthorized".into(),
            message: "Sign in with payments access before funding escrow from the Rust route."
                .into(),
        }));
    };
    let provided_idempotency_key =
        payment_idempotency_key(&headers, payload.idempotency_key.as_deref());
    let request_fingerprint = payment_request_fingerprint("escrow_fund", leg_id, &payload);
    let idempotency_key = effective_payment_idempotency_key(
        provided_idempotency_key,
        "escrow_fund",
        leg_id,
        session.user.id,
        &request_fingerprint,
    );
    if let Some(replay) =
        replay_payment_idempotency(pool, "escrow_fund", &idempotency_key, &request_fingerprint)
            .await
    {
        return Json(ApiResponse::ok(replay));
    }

    let rate_decision = state
        .check_rate_limit(
            payments_policy("escrow_fund"),
            format!("{}:{}", session.user.id, leg_id),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Rate Limited".into(),
            message: rate_limit_message("escrow funding", rate_decision.retry_after_seconds),
        }));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only the load owner or an admin can fund escrow for this leg.",
        )));
    }

    let Some(leg) = find_load_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    let Some(payee_user_id) = leg.booked_carrier_id else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Locked".into(),
            message: "Escrow cannot be funded until the leg has a booked carrier.".into(),
        }));
    };

    let Some(carrier) = find_user_by_id(pool, payee_user_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Carrier".into(),
            message: "The booked carrier could not be found for this escrow funding flow.".into(),
        }));
    };

    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Unavailable".into(),
            message: "The load owner could not be resolved for this leg.".into(),
        }));
    };

    let payment_currency =
        match normalize_supported_freight_payment_currency(payload.currency.as_deref()) {
            Ok(currency) => currency,
            Err(message) => {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: None,
                    payment_intent_id: None,
                    client_secret: None,
                    transfer_id: None,
                    status_label: "Currency Deferred".into(),
                    message,
                }));
            }
        };

    let Some(amount) = payload
        .amount_cents
        .or_else(|| leg.booked_amount.or(leg.price).map(currency_to_cents))
    else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Amount".into(),
            message: "Escrow funding requires either an explicit amount or a priced leg.".into(),
        }));
    };

    if state.stripe.is_configured() && payload.payment_intent_id.is_none() {
        if carrier
            .stripe_connect_account_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: None,
                payment_intent_id: None,
                client_secret: None,
                transfer_id: None,
                status_label: "Carrier Not Ready".into(),
                message: "Carrier has not completed Stripe Connect payout setup, so a live PaymentIntent cannot be created.".into(),
            }));
        }

        let stripe_currency = payment_currency.to_ascii_lowercase();
        let transfer_group = payload
            .transfer_group
            .clone()
            .unwrap_or_else(|| format!("LEG_{}", leg_id));
        let description = format!(
            "Funding leg {} for ${:.2}",
            leg.leg_code
                .clone()
                .unwrap_or_else(|| format!("LEG-{}", leg_id)),
            amount as f64 / 100.0
        );
        let payment_intent = match state
            .stripe
            .create_payment_intent(
                amount,
                &stripe_currency,
                &transfer_group,
                leg_id,
                &description,
                request_id.as_deref(),
                Some(&idempotency_key),
            )
            .await
        {
            Ok(value) => value,
            Err(error) => {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: None,
                    payment_intent_id: None,
                    client_secret: None,
                    transfer_id: None,
                    status_label: "Stripe Error".into(),
                    message: format!("Stripe PaymentIntent creation failed: {}", error),
                }));
            }
        };

        return match apply_escrow_transition(
            pool,
            EscrowTransitionParams {
                leg_id,
                payer_user_id,
                payee_user_id,
                amount,
                platform_fee: payload.platform_fee_cents.unwrap_or(0),
                currency: &payment_currency,
                status: EscrowStatus::Unfunded,
                transfer_group: Some(&transfer_group),
                payment_intent_id: Some(&payment_intent.id),
                charge_id: None,
                transfer_id: None,
                stripe_refund_id: None,
                stripe_dispute_id: None,
                adjustment_reference: None,
                actor_user_id: Some(session.user.id),
                note: payload.note.as_deref().or(Some(
                    "Live Stripe PaymentIntent created by the Rust payments route.",
                )),
            },
        )
        .await
        {
            Ok(Some(escrow)) => {
                let response = EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                payment_intent_id: Some(payment_intent.id),
                client_secret: payment_intent.client_secret,
                transfer_id: None,
                status_label: "Payment Intent Created".into(),
                message: "Live Stripe PaymentIntent created. The escrow will move to funded when Stripe sends payment_intent.succeeded.".into(),
                };
                remember_payment_idempotency(
                    pool,
                    "escrow_fund",
                    &idempotency_key,
                    leg_id,
                    Some(session.user.id),
                    &request_fingerprint,
                    &response,
                )
                .await;
                Json(ApiResponse::ok(response))
            }
            Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
            Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: None,
                payment_intent_id: Some(payment_intent.id),
                client_secret: payment_intent.client_secret,
                transfer_id: None,
                status_label: "Error".into(),
                message: format!(
                    "PaymentIntent was created, but escrow persistence failed: {}",
                    error
                ),
            })),
        };
    }

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id,
            payee_user_id,
            amount,
            platform_fee: payload.platform_fee_cents.unwrap_or(0),
            currency: &payment_currency,
            status: EscrowStatus::Funded,
            transfer_group: payload.transfer_group.as_deref(),
            payment_intent_id: payload.payment_intent_id.as_deref(),
            charge_id: payload.charge_id.as_deref(),
            transfer_id: None,
            stripe_refund_id: None,
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            publish_payments_event(
                &state,
                leg_id,
                Some(session.user.id.max(0) as u64),
                Some(payee_user_id.max(0) as u64),
                vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                format!(
                    "{} funded escrow for load leg #{}.",
                    session.user.name, leg_id
                ),
            );

            let response = EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "Funded".into(),
                message: "Escrow funded through the Rust payments route; admin and load-board views will refresh through targeted realtime events.".into(),
            };
            remember_payment_idempotency(
                pool,
                "escrow_fund",
                &idempotency_key,
                leg_id,
                Some(session.user.id),
                &request_fingerprint,
                &response,
            )
            .await;
            Json(ApiResponse::ok(response))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Error".into(),
            message: format!("Funding failed: {}", error),
        })),
    }
}

async fn hold_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowHoldRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Hold",
        )));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Unauthorized".into(),
            message: "Sign in with payments access before holding escrow from the Rust route."
                .into(),
        }));
    };
    let provided_idempotency_key =
        payment_idempotency_key(&headers, payload.idempotency_key.as_deref());
    let request_fingerprint = payment_request_fingerprint("escrow_hold", leg_id, &payload);
    let idempotency_key = effective_payment_idempotency_key(
        provided_idempotency_key,
        "escrow_hold",
        leg_id,
        session.user.id,
        &request_fingerprint,
    );
    if let Some(replay) =
        replay_payment_idempotency(pool, "escrow_hold", &idempotency_key, &request_fingerprint)
            .await
    {
        return Json(ApiResponse::ok(replay));
    }

    let rate_decision = state
        .check_rate_limit(
            payments_policy("escrow_hold"),
            format!("{}:{}", session.user.id, leg_id),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Rate Limited".into(),
            message: rate_limit_message("escrow hold", rate_decision.retry_after_seconds),
        }));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only the load owner or an admin can place escrow on hold.",
        )));
    }

    let Some(leg) = find_load_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };
    let Some(payee_user_id) = leg.booked_carrier_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Escrow hold requires a booked carrier on the selected leg.",
        )));
    };
    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "The load owner could not be resolved for this leg.",
        )));
    };

    let Some(amount) = find_escrow_for_leg(pool, leg_id)
        .await
        .ok()
        .flatten()
        .map(|escrow| escrow.amount)
        .or_else(|| leg.booked_amount.or(leg.price).map(currency_to_cents))
    else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Amount".into(),
            message: "Escrow hold requires an existing funded amount or a priced leg.".into(),
        }));
    };

    match finance_request_has_required_approval(
        pool,
        "escrow_hold",
        "load_leg",
        leg_id,
        MANUAL_HOLD_REQUIRED_APPROVALS,
    )
    .await
    {
        Ok(true) => {}
        Ok(false) => {
            match ensure_finance_approval_request(
                pool,
                FinanceApprovalRequestParams {
                    approval_type: "escrow_hold",
                    entity_type: "load_leg",
                    entity_id: leg_id,
                    organization_id: Some(scope.organization_id),
                    amount_cents: amount,
                    currency: "USD",
                    required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
                    requested_by_user_id: Some(session.user.id),
                    reason: Some("Manual escrow hold requires finance approval before placement."),
                },
            )
            .await
            {
                Ok(approval) => {
                    return Json(ApiResponse::ok(EscrowLifecycleResponse {
                        success: false,
                        leg_id,
                        escrow_id: None,
                        payment_intent_id: None,
                        client_secret: None,
                        transfer_id: None,
                        status_label: "Hold Approval Required".into(),
                        message: format!(
                            "Manual hold requires {}/{} finance approvals before placement. Approval request #{} is {}.",
                            finance_approval_count(&approval),
                            approval.required_approval_count,
                            approval.id,
                            approval.status
                        ),
                    }));
                }
                Err(error) => {
                    return Json(ApiResponse::ok(EscrowLifecycleResponse {
                        success: false,
                        leg_id,
                        escrow_id: None,
                        payment_intent_id: None,
                        client_secret: None,
                        transfer_id: None,
                        status_label: "Hold Approval Check Failed".into(),
                        message: format!(
                            "Manual hold could not create the finance approval request: {}",
                            error
                        ),
                    }));
                }
            }
        }
        Err(error) => {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: None,
                payment_intent_id: None,
                client_secret: None,
                transfer_id: None,
                status_label: "Hold Approval Check Failed".into(),
                message: format!("Manual hold could not verify finance approvals: {}", error),
            }));
        }
    }

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id,
            payee_user_id,
            amount,
            platform_fee: 0,
            currency: "USD",
            status: EscrowStatus::OnHold,
            transfer_group: None,
            payment_intent_id: None,
            charge_id: None,
            transfer_id: None,
            stripe_refund_id: None,
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            publish_payments_event(
                &state,
                leg_id,
                Some(session.user.id.max(0) as u64),
                Some(payee_user_id.max(0) as u64),
                vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                format!(
                    "{} placed escrow on hold for load leg #{}.",
                    session.user.name, leg_id
                ),
            );

            let response = EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "On Hold".into(),
                message: "Escrow placed on hold through the Rust payments route.".into(),
            };
            remember_payment_idempotency(
                pool,
                "escrow_hold",
                &idempotency_key,
                leg_id,
                Some(session.user.id),
                &request_fingerprint,
                &response,
            )
            .await;
            Json(ApiResponse::ok(response))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Error".into(),
            message: format!("Escrow hold failed: {}", error),
        })),
    }
}

async fn refund_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowRefundRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Refund",
        )));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Sign in with finance access before refunding escrow.",
        )));
    };
    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Refunds require manage_payments or admin access with MFA step-up.",
        )));
    }

    let provided_idempotency_key =
        payment_idempotency_key(&headers, payload.idempotency_key.as_deref());
    let request_fingerprint = payment_request_fingerprint("escrow_refund", leg_id, &payload);
    let idempotency_key = effective_payment_idempotency_key(
        provided_idempotency_key,
        "escrow_refund",
        leg_id,
        session.user.id,
        &request_fingerprint,
    );
    if let Some(replay) = replay_payment_idempotency(
        pool,
        "escrow_refund",
        &idempotency_key,
        &request_fingerprint,
    )
    .await
    {
        return Json(ApiResponse::ok(replay));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };
    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only finance admins for this organization can refund escrow.",
        )));
    }

    let Some(existing_escrow) = find_escrow_for_leg(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Escrow".into(),
            message: "Escrow must exist before a refund can be recorded.".into(),
        }));
    };
    if matches!(
        existing_escrow.escrow_status(),
        Some(EscrowStatus::Refunded)
    ) {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: Some(existing_escrow.id),
            payment_intent_id: existing_escrow.payment_intent_id,
            client_secret: None,
            transfer_id: existing_escrow.transfer_id,
            status_label: "Already Refunded".into(),
            message: "This escrow is already marked refunded.".into(),
        }));
    }

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id: existing_escrow.payer_user_id,
            payee_user_id: existing_escrow.payee_user_id,
            amount: existing_escrow.amount,
            platform_fee: existing_escrow.platform_fee,
            currency: existing_escrow.currency.as_str(),
            status: EscrowStatus::Refunded,
            transfer_group: existing_escrow.transfer_group.as_deref(),
            payment_intent_id: existing_escrow.payment_intent_id.as_deref(),
            charge_id: existing_escrow.charge_id.as_deref(),
            transfer_id: existing_escrow.transfer_id.as_deref(),
            stripe_refund_id: payload.refund_id.as_deref(),
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref().or(Some(
                "Full escrow refund recorded through the Rust finance route.",
            )),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            let response = EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "Refunded".into(),
                message: "Full escrow refund recorded with idempotent finance controls.".into(),
            };
            remember_payment_idempotency(
                pool,
                "escrow_refund",
                &idempotency_key,
                leg_id,
                Some(session.user.id),
                &request_fingerprint,
                &response,
            )
            .await;
            Json(ApiResponse::ok(response))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: Some(existing_escrow.id),
            payment_intent_id: existing_escrow.payment_intent_id,
            client_secret: None,
            transfer_id: existing_escrow.transfer_id,
            status_label: "Refund Error".into(),
            message: format!("Escrow refund could not be recorded: {}", error),
        })),
    }
}

async fn record_payment_adjustment(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<PaymentAdjustmentRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let normalized_direction = payload.direction.trim().to_ascii_lowercase();
    let signed_adjustment = match normalized_direction.as_str() {
        "credit" | "release" => payload.amount_cents,
        "debit" | "hold" => -payload.amount_cents,
        _ => payload.amount_cents,
    };
    if let Some(pool) = state.pool.as_ref() {
        let _ = apply_invoice_settlement_adjustment(
            pool,
            leg_id,
            signed_adjustment,
            payload.note.as_deref(),
        )
        .await;
    }
    record_manual_finance_ledger_event(
        state,
        headers,
        leg_id,
        "payment_adjustment",
        "adjustment",
        normalized_direction.as_str(),
        payload.amount_cents,
        payload.adjustment_reference.as_deref(),
        None,
        payload.note.as_deref(),
        payload.idempotency_key.as_deref(),
    )
    .await
}

async fn record_payment_dispute(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<PaymentDisputeRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    record_manual_finance_ledger_event(
        state,
        headers,
        leg_id,
        "payment_dispute",
        "dispute",
        "hold",
        payload.amount_cents,
        None,
        payload.stripe_dispute_id.as_deref(),
        payload.note.as_deref(),
        payload.idempotency_key.as_deref(),
    )
    .await
}

async fn release_approval_queue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<FinanceApprovalQueueResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceApprovalQueueResponse {
            approvals: Vec::new(),
            message: "Finance approval queue requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceApprovalQueueResponse {
            approvals: Vec::new(),
            message: "Sign in with finance permissions before viewing approval queue.".into(),
        }));
    };
    if !session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "manage_payments" || permission == "access_admin_portal")
    {
        return Json(ApiResponse::ok(FinanceApprovalQueueResponse {
            approvals: Vec::new(),
            message: "Finance approval queue requires manage_payments or admin access.".into(),
        }));
    }

    let organization_id = auth_session::session_organization_id(&session);
    match list_pending_finance_release_approvals(pool, organization_id, 50).await {
        Ok(rows) => {
            let approvals = rows
                .into_iter()
                .map(|row| {
                    let approval_count = i32::from(row.first_approved_by_user_id.is_some())
                        + i32::from(row.second_approved_by_user_id.is_some());
                    FinanceApprovalQueueItem {
                        approval_id: row.id,
                        approval_type: row.approval_type,
                        leg_id: row.entity_id,
                        load_id: row.load_id,
                        amount_cents: row.amount_cents,
                        currency: row.currency,
                        status: row.status,
                        required_approval_count: row.required_approval_count,
                        approval_count,
                        requested_by_user_id: row.requested_by_user_id,
                        first_approved_by_user_id: row.first_approved_by_user_id,
                        second_approved_by_user_id: row.second_approved_by_user_id,
                        reason: row.reason,
                        updated_at: row.updated_at.to_string(),
                    }
                })
                .collect::<Vec<_>>();
            Json(ApiResponse::ok(FinanceApprovalQueueResponse {
                message: format!("{} pending finance approval(s).", approvals.len()),
                approvals,
            }))
        }
        Err(error) => Json(ApiResponse::ok(FinanceApprovalQueueResponse {
            approvals: Vec::new(),
            message: format!("Finance approval queue could not load: {}", error),
        })),
    }
}

async fn invoice_settlement_queue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<InvoiceSettlementQueueResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(InvoiceSettlementQueueResponse {
            rows: Vec::new(),
            message: "Invoice and settlement queue requires the database connection.".into(),
        }));
    };
    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(InvoiceSettlementQueueResponse {
            rows: Vec::new(),
            message: "Sign in with finance permissions before viewing invoices and settlements."
                .into(),
        }));
    };
    if !session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "manage_payments" || permission == "access_admin_portal")
    {
        return Json(ApiResponse::ok(InvoiceSettlementQueueResponse {
            rows: Vec::new(),
            message: "Invoice and settlement queue requires manage_payments or admin access."
                .into(),
        }));
    }

    match list_invoice_settlement_queue(pool, 100).await {
        Ok(rows) => {
            let rows = rows
                .into_iter()
                .map(|row| InvoiceSettlementQueueItem {
                    invoice_id: row.invoice_id,
                    invoice_number: row.invoice_number,
                    settlement_id: row.settlement_id,
                    settlement_number: row.settlement_number,
                    load_id: row.load_id,
                    leg_id: row.leg_id,
                    customer_user_id: row.customer_user_id,
                    carrier_user_id: row.carrier_user_id,
                    currency: row.invoice_currency,
                    invoice_total_amount_cents: row.invoice_total_amount_cents,
                    invoice_adjustment_amount_cents: row.invoice_adjustment_amount_cents,
                    invoice_status: row.invoice_status,
                    settlement_gross_amount_cents: row.settlement_gross_amount_cents,
                    settlement_platform_fee_cents: row.settlement_platform_fee_cents,
                    settlement_adjustment_amount_cents: row.settlement_adjustment_amount_cents,
                    settlement_net_amount_cents: row.settlement_net_amount_cents,
                    settlement_status: row.settlement_status,
                    updated_at: row.updated_at.to_string(),
                })
                .collect::<Vec<_>>();
            Json(ApiResponse::ok(InvoiceSettlementQueueResponse {
                message: format!("{} invoice/settlement row(s).", rows.len()),
                rows,
            }))
        }
        Err(error) => Json(ApiResponse::ok(InvoiceSettlementQueueResponse {
            rows: Vec::new(),
            message: format!("Invoice and settlement queue could not load: {}", error),
        })),
    }
}

async fn approve_release_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<FinanceApprovalActionRequest>,
) -> Json<ApiResponse<FinanceApprovalActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "unavailable".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Finance approval workflow requires the database connection.".into(),
        }));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "unauthorized".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Sign in with finance permissions before approving release.".into(),
        }));
    };

    if !session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "manage_payments" || permission == "access_admin_portal")
    {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "forbidden".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Finance release approval requires manage_payments or admin access.".into(),
        }));
    }
    if !session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "mfa_verified")
    {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "mfa_required".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "MFA step-up is required before approving high-value release.".into(),
        }));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "missing_leg".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "The selected load leg was not found.".into(),
        }));
    };
    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "forbidden".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Only finance admins for this organization can approve release.".into(),
        }));
    }

    let Some(existing_escrow) = find_escrow_for_leg(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "missing_escrow".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Escrow must exist before release approval can be recorded.".into(),
        }));
    };
    let payout_amount = existing_escrow.amount - existing_escrow.platform_fee;
    if payout_amount < HIGH_VALUE_RELEASE_APPROVAL_CENTS {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: true,
            leg_id,
            approval_id: None,
            status: "not_required".into(),
            required_approval_count: 0,
            approval_count: 0,
            message: "This release is below the high-value approval threshold.".into(),
        }));
    }

    let approval = match ensure_finance_approval_request(
        pool,
        FinanceApprovalRequestParams {
            approval_type: "escrow_release",
            entity_type: "load_leg",
            entity_id: leg_id,
            organization_id: Some(scope.organization_id),
            amount_cents: payout_amount,
            currency: &existing_escrow.currency,
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            requested_by_user_id: None,
            reason: Some("High-value escrow release requires finance approval."),
        },
    )
    .await
    {
        Ok(approval) => approval,
        Err(error) => {
            return Json(ApiResponse::ok(FinanceApprovalActionResponse {
                success: false,
                leg_id,
                approval_id: None,
                status: "approval_error".into(),
                required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
                approval_count: 0,
                message: format!("Finance approval could not be prepared: {}", error),
            }));
        }
    };

    if approval.requested_by_user_id == Some(session.user.id) {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: Some(approval.id),
            status: approval.status.clone(),
            required_approval_count: approval.required_approval_count,
            approval_count: finance_approval_count(&approval),
            message: "The requester cannot approve their own high-value release.".into(),
        }));
    }

    match approve_finance_request(
        pool,
        FinanceApprovalDecisionParams {
            approval_type: "escrow_release",
            entity_type: "load_leg",
            entity_id: leg_id,
            approver_user_id: session.user.id,
            decision_note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(updated)) => {
            let approval_count = finance_approval_count(&updated);
            Json(ApiResponse::ok(FinanceApprovalActionResponse {
                success: updated.status == "approved",
                leg_id,
                approval_id: Some(updated.id),
                status: updated.status.clone(),
                required_approval_count: updated.required_approval_count,
                approval_count,
                message: if updated.status == "approved" {
                    "High-value release has the required finance approvals.".into()
                } else {
                    format!(
                        "Finance approval recorded. {}/{} approvals are complete.",
                        approval_count, updated.required_approval_count
                    )
                },
            }))
        }
        Ok(None) => Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "missing_approval".into(),
            required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "No open finance approval request was available for this release.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: Some(approval.id),
            status: "approval_error".into(),
            required_approval_count: approval.required_approval_count,
            approval_count: finance_approval_count(&approval),
            message: format!("Finance approval could not be recorded: {}", error),
        })),
    }
}

async fn approve_hold_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<FinanceApprovalActionRequest>,
) -> Json<ApiResponse<FinanceApprovalActionResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "unavailable".into(),
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Finance hold approval workflow requires the database connection.".into(),
        }));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "unauthorized".into(),
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Sign in with finance permissions before approving hold.".into(),
        }));
    };

    if !can_perform_finance_action(&session) {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "mfa_required".into(),
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Finance hold approval requires manage_payments or admin access with MFA."
                .into(),
        }));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "missing_leg".into(),
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "The selected load leg was not found.".into(),
        }));
    };
    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "forbidden".into(),
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "Only finance admins for this organization can approve hold.".into(),
        }));
    }

    let amount = find_escrow_for_leg(pool, leg_id)
        .await
        .ok()
        .flatten()
        .map(|escrow| escrow.amount)
        .unwrap_or_default();
    let approval = match ensure_finance_approval_request(
        pool,
        FinanceApprovalRequestParams {
            approval_type: "escrow_hold",
            entity_type: "load_leg",
            entity_id: leg_id,
            organization_id: Some(scope.organization_id),
            amount_cents: amount,
            currency: "USD",
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            requested_by_user_id: None,
            reason: Some("Manual escrow hold requires finance approval before placement."),
        },
    )
    .await
    {
        Ok(approval) => approval,
        Err(error) => {
            return Json(ApiResponse::ok(FinanceApprovalActionResponse {
                success: false,
                leg_id,
                approval_id: None,
                status: "approval_error".into(),
                required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
                approval_count: 0,
                message: format!("Finance hold approval could not be prepared: {}", error),
            }));
        }
    };

    if approval.requested_by_user_id == Some(session.user.id) {
        return Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: Some(approval.id),
            status: approval.status.clone(),
            required_approval_count: approval.required_approval_count,
            approval_count: finance_approval_count(&approval),
            message: "The requester cannot approve their own manual hold.".into(),
        }));
    }

    match approve_finance_request(
        pool,
        FinanceApprovalDecisionParams {
            approval_type: "escrow_hold",
            entity_type: "load_leg",
            entity_id: leg_id,
            approver_user_id: session.user.id,
            decision_note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(updated)) => {
            let approval_count = finance_approval_count(&updated);
            Json(ApiResponse::ok(FinanceApprovalActionResponse {
                success: updated.status == "approved",
                leg_id,
                approval_id: Some(updated.id),
                status: updated.status.clone(),
                required_approval_count: updated.required_approval_count,
                approval_count,
                message: if updated.status == "approved" {
                    "Manual hold has the required finance approval.".into()
                } else {
                    format!(
                        "Hold approval recorded. {}/{} approvals are complete.",
                        approval_count, updated.required_approval_count
                    )
                },
            }))
        }
        Ok(None) => Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: None,
            status: "missing_approval".into(),
            required_approval_count: MANUAL_HOLD_REQUIRED_APPROVALS,
            approval_count: 0,
            message: "No open finance approval request was available for this hold.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(FinanceApprovalActionResponse {
            success: false,
            leg_id,
            approval_id: Some(approval.id),
            status: "approval_error".into(),
            required_approval_count: approval.required_approval_count,
            approval_count: finance_approval_count(&approval),
            message: format!("Finance hold approval could not be recorded: {}", error),
        })),
    }
}

async fn release_leg_escrow(
    State(state): State<AppState>,
    Path(leg_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<EscrowReleaseRequest>,
) -> Json<ApiResponse<EscrowLifecycleResponse>> {
    let request_id = request_id_from_headers(&headers);
    if state.config.kill_switch_payments {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Paused".into(),
            message: "Payment release is temporarily disabled by an operational kill switch."
                .into(),
        }));
    }

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(unavailable_payment_response(
            &state, leg_id, "Release",
        )));
    };

    let Ok(Some(session)) = resolve_payments_session(&state, &headers).await else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Unauthorized".into(),
            message: "Sign in with payments access before releasing escrow from the Rust route."
                .into(),
        }));
    };

    let provided_idempotency_key =
        payment_idempotency_key(&headers, payload.idempotency_key.as_deref());
    let request_fingerprint = payment_request_fingerprint("escrow_release", leg_id, &payload);
    let idempotency_key = effective_payment_idempotency_key(
        provided_idempotency_key,
        "escrow_release",
        leg_id,
        session.user.id,
        &request_fingerprint,
    );
    if let Some(replay) = replay_payment_idempotency(
        pool,
        "escrow_release",
        &idempotency_key,
        &request_fingerprint,
    )
    .await
    {
        return Json(ApiResponse::ok(replay));
    }

    let rate_decision = state
        .check_rate_limit(
            payments_policy("escrow_release"),
            format!("{}:{}", session.user.id, leg_id),
        )
        .await;
    if !rate_decision.allowed {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Rate Limited".into(),
            message: rate_limit_message("escrow release", rate_decision.retry_after_seconds),
        }));
    }

    let Some(scope) = find_load_leg_scope(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };

    if !can_manage_leg_payments(&session, scope.load_owner_user_id, scope.organization_id) {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Only the load owner or an admin can release escrow for this leg.",
        )));
    }
    if !session
        .session
        .permissions
        .iter()
        .any(|permission| permission == "mfa_verified")
    {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "MFA step-up is required before releasing escrow.",
        )));
    }

    let Some(leg) = find_load_leg_by_id(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(missing_leg_response(leg_id)));
    };
    let Some(existing_escrow) = find_escrow_for_leg(pool, leg_id).await.ok().flatten() else {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id: None,
            status_label: "Missing Escrow".into(),
            message: "Escrow must exist before it can be released.".into(),
        }));
    };
    let Some(payee_user_id) = leg.booked_carrier_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Escrow release requires a booked carrier on the selected leg.",
        )));
    };
    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "Escrow release requires a load owner payer.",
        )));
    };
    if shipper_credit_blocks_release(pool, payer_user_id).await {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: Some(existing_escrow.id),
            payment_intent_id: existing_escrow.payment_intent_id,
            client_secret: None,
            transfer_id: existing_escrow.transfer_id,
            status_label: "Credit Hold".into(),
            message: "Escrow release is blocked by shipper credit hold, over-limit exposure, or overdue AR until Finance approves an override.".into(),
        }));
    }
    if payout_destination_blocks_release(pool, payee_user_id).await {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: Some(existing_escrow.id),
            payment_intent_id: existing_escrow.payment_intent_id,
            client_secret: None,
            transfer_id: existing_escrow.transfer_id,
            status_label: "Payout Review".into(),
            message: "Escrow release is blocked because the carrier payout destination has an open review, cooling-off, or blocked status.".into(),
        }));
    }
    if let Some(message) = compliance_payout_blocks_release(pool, payee_user_id, leg_id).await {
        return Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: Some(existing_escrow.id),
            payment_intent_id: existing_escrow.payment_intent_id,
            client_secret: None,
            transfer_id: existing_escrow.transfer_id,
            status_label: "Compliance Hold".into(),
            message,
        }));
    }
    match execution_closeout_readiness(pool, leg_id).await {
        Ok(readiness)
            if readiness.delivery_pod_count > 0
                && readiness.open_exception_count == 0
                && readiness.pending_offline_count == 0
                && matches!(
                    readiness.pod_review_status.as_deref(),
                    Some("approved") | Some("accepted")
                ) => {}
        Ok(readiness) => {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: Some(existing_escrow.id),
                payment_intent_id: existing_escrow.payment_intent_id,
                client_secret: None,
                transfer_id: existing_escrow.transfer_id,
                status_label: "Closeout Blocked".into(),
                message: format!(
                    "Escrow release is blocked until closeout is approved: POD count {}, POD review {}, open exceptions {}, pending offline submissions {}.",
                    readiness.delivery_pod_count,
                    readiness
                        .pod_review_status
                        .unwrap_or_else(|| "pending".into()),
                    readiness.open_exception_count,
                    readiness.pending_offline_count
                ),
            }));
        }
        Err(error) => {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: Some(existing_escrow.id),
                payment_intent_id: existing_escrow.payment_intent_id,
                client_secret: None,
                transfer_id: existing_escrow.transfer_id,
                status_label: "Closeout Check Failed".into(),
                message: format!(
                    "Escrow release could not verify closeout readiness: {}",
                    error
                ),
            }));
        }
    }
    let Some(payer_user_id) = scope.load_owner_user_id else {
        return Json(ApiResponse::ok(forbidden_payment_response(
            leg_id,
            "The load owner could not be resolved for this leg.",
        )));
    };
    let payout_amount_for_approval = existing_escrow.amount - existing_escrow.platform_fee;
    if payout_amount_for_approval >= HIGH_VALUE_RELEASE_APPROVAL_CENTS {
        match release_has_required_finance_approval(
            pool,
            leg_id,
            HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
        )
        .await
        {
            Ok(true) => {}
            Ok(false) => {
                match ensure_finance_approval_request(
                    pool,
                    FinanceApprovalRequestParams {
                        approval_type: "escrow_release",
                        entity_type: "load_leg",
                        entity_id: leg_id,
                        organization_id: Some(scope.organization_id),
                        amount_cents: payout_amount_for_approval,
                        currency: &existing_escrow.currency,
                        required_approval_count: HIGH_VALUE_RELEASE_REQUIRED_APPROVALS,
                        requested_by_user_id: Some(session.user.id),
                        reason: Some("High-value escrow release requires finance approval."),
                    },
                )
                .await
                {
                    Ok(approval) => {
                        return Json(ApiResponse::ok(EscrowLifecycleResponse {
                            success: false,
                            leg_id,
                            escrow_id: Some(existing_escrow.id),
                            payment_intent_id: existing_escrow.payment_intent_id,
                            client_secret: None,
                            transfer_id: existing_escrow.transfer_id,
                            status_label: "Approval Required".into(),
                            message: format!(
                                "High-value release requires {}/{} finance approvals before payout. Approval request #{} is {}.",
                                finance_approval_count(&approval),
                                approval.required_approval_count,
                                approval.id,
                                approval.status
                            ),
                        }));
                    }
                    Err(error) => {
                        return Json(ApiResponse::ok(EscrowLifecycleResponse {
                            success: false,
                            leg_id,
                            escrow_id: Some(existing_escrow.id),
                            payment_intent_id: existing_escrow.payment_intent_id,
                            client_secret: None,
                            transfer_id: existing_escrow.transfer_id,
                            status_label: "Approval Check Failed".into(),
                            message: format!(
                                "High-value release could not create the finance approval request: {}",
                                error
                            ),
                        }));
                    }
                }
            }
            Err(error) => {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Approval Check Failed".into(),
                    message: format!(
                        "High-value release could not verify finance approvals: {}",
                        error
                    ),
                }));
            }
        }
    }

    let transfer_id = match payload
        .transfer_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => Some(value.to_string()),
        None if state.stripe.is_configured() => {
            if existing_escrow.escrow_status() != Some(EscrowStatus::Funded) {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Not Funded".into(),
                    message: "Live Stripe release requires a funded escrow.".into(),
                }));
            }

            let Some(charge_id) = existing_escrow
                .charge_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Missing Charge".into(),
                    message:
                        "Live Stripe release requires the charge_id from payment_intent.succeeded."
                            .into(),
                }));
            };

            let Some(carrier) = find_user_by_id(pool, payee_user_id).await.ok().flatten() else {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Missing Carrier".into(),
                    message: "The booked carrier could not be found for the live Stripe transfer."
                        .into(),
                }));
            };

            let Some(destination_account_id) = carrier
                .stripe_connect_account_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Carrier Not Ready".into(),
                    message: "Carrier does not have a Stripe Connect account for payout.".into(),
                }));
            };

            let payout_amount = existing_escrow.amount - existing_escrow.platform_fee;
            if payout_amount <= 0 {
                return Json(ApiResponse::ok(EscrowLifecycleResponse {
                    success: false,
                    leg_id,
                    escrow_id: Some(existing_escrow.id),
                    payment_intent_id: existing_escrow.payment_intent_id,
                    client_secret: None,
                    transfer_id: existing_escrow.transfer_id,
                    status_label: "Invalid Payout".into(),
                    message: "Escrow amount minus platform fee must be positive before release."
                        .into(),
                }));
            }

            match state
                .stripe
                .create_transfer(
                    payout_amount,
                    &existing_escrow.currency,
                    destination_account_id,
                    charge_id,
                    existing_escrow.transfer_group.as_deref(),
                    request_id.as_deref(),
                    Some(&idempotency_key),
                )
                .await
            {
                Ok(transfer) => Some(transfer.id),
                Err(error) => {
                    return Json(ApiResponse::ok(EscrowLifecycleResponse {
                        success: false,
                        leg_id,
                        escrow_id: Some(existing_escrow.id),
                        payment_intent_id: existing_escrow.payment_intent_id,
                        client_secret: None,
                        transfer_id: existing_escrow.transfer_id,
                        status_label: "Stripe Error".into(),
                        message: format!("Live Stripe transfer failed: {}", error),
                    }));
                }
            }
        }
        None if state.config.stripe_live_transfers_required => {
            return Json(ApiResponse::ok(EscrowLifecycleResponse {
                success: false,
                leg_id,
                escrow_id: Some(existing_escrow.id),
                payment_intent_id: existing_escrow.payment_intent_id,
                client_secret: None,
                transfer_id: existing_escrow.transfer_id,
                status_label: "Stripe Required".into(),
                message: "STRIPE_LIVE_TRANSFERS_REQUIRED is enabled, but no live Stripe transfer could be created because STRIPE_SECRET is not configured.".into(),
            }));
        }
        None => None,
    };

    match apply_escrow_transition(
        pool,
        EscrowTransitionParams {
            leg_id,
            payer_user_id,
            payee_user_id,
            amount: existing_escrow.amount,
            platform_fee: existing_escrow.platform_fee,
            currency: existing_escrow.currency.as_str(),
            status: EscrowStatus::Released,
            transfer_group: existing_escrow.transfer_group.as_deref(),
            payment_intent_id: existing_escrow.payment_intent_id.as_deref(),
            charge_id: existing_escrow.charge_id.as_deref(),
            transfer_id: transfer_id.as_deref(),
            stripe_refund_id: None,
            stripe_dispute_id: None,
            adjustment_reference: None,
            actor_user_id: Some(session.user.id),
            note: payload.note.as_deref(),
        },
    )
    .await
    {
        Ok(Some(escrow)) => {
            publish_payments_event(
                &state,
                leg_id,
                Some(session.user.id.max(0) as u64),
                Some(payee_user_id.max(0) as u64),
                vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                format!(
                    "{} released escrow for load leg #{}.",
                    session.user.name, leg_id
                ),
            );

            let response = EscrowLifecycleResponse {
                success: true,
                leg_id,
                escrow_id: Some(escrow.id),
                payment_intent_id: escrow.payment_intent_id,
                client_secret: None,
                transfer_id: escrow.transfer_id,
                status_label: "Released".into(),
                message: "Escrow released through the Rust payments route; payout readiness will refresh on admin and load-board views.".into(),
            };
            remember_payment_idempotency(
                pool,
                "escrow_release",
                &idempotency_key,
                leg_id,
                Some(session.user.id),
                &request_fingerprint,
                &response,
            )
            .await;
            Json(ApiResponse::ok(response))
        }
        Ok(None) => Json(ApiResponse::ok(missing_leg_response(leg_id))),
        Err(error) => Json(ApiResponse::ok(EscrowLifecycleResponse {
            success: false,
            leg_id,
            escrow_id: None,
            payment_intent_id: None,
            client_secret: None,
            transfer_id,
            status_label: "Error".into(),
            message: format!("Escrow release failed: {}", error),
        })),
    }
}

async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ApiResponse<StripeWebhookResponse>>, StatusCode> {
    let rate_decision = state
        .check_rate_limit(
            webhook_policy("stripe_webhook"),
            client_fingerprint(&headers),
        )
        .await;
    if !rate_decision.allowed {
        warn!(
            retry_after_seconds = rate_decision.retry_after_seconds,
            "Stripe webhook rate limited"
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let actor_session = match authorize_payments_webhook(&state, &headers, &body).await {
        Ok(session) => session,
        Err(status) => {
            warn!(status = ?status, "Stripe webhook authorization failed");
            return Err(status);
        }
    };
    let payload = parse_stripe_webhook_payload(&body).map_err(|error| {
        warn!(error = %error, body_len = body.len(), "Stripe webhook payload parsing failed");
        StatusCode::BAD_REQUEST
    })?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
            acknowledged: false,
            event_type: payload.event_type,
            leg_id: payload.leg_id,
            user_id: None,
            message: format!(
                "Stripe webhook handling is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if let Some(event_id) = payload.event_id.as_deref() {
        let claimed = claim_stripe_webhook_event(
            pool,
            event_id,
            &payload.event_type,
            payload.payment_intent_id.as_deref(),
            payload.stripe_account_id.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if !claimed {
            return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: true,
                event_type: payload.event_type,
                leg_id: payload.leg_id,
                user_id: None,
                message:
                    "Duplicate Stripe webhook event ignored; original processing was already claimed."
                        .into(),
            })));
        }
    }

    match payload.event_type.as_str() {
        "payment_intent.succeeded" => {
            let Some((leg_id, payer_user_id, payee_user_id, amount, platform_fee, currency)) =
                resolve_webhook_escrow_context(pool, &payload).await
            else {
                return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                    acknowledged: false,
                    event_type: payload.event_type,
                    leg_id: payload.leg_id,
                    user_id: None,
                    message: "The webhook could not resolve an escrow context for funding.".into(),
                })));
            };

            let updated = apply_escrow_transition(
                pool,
                EscrowTransitionParams {
                    leg_id,
                    payer_user_id,
                    payee_user_id,
                    amount,
                    platform_fee,
                    currency: currency.as_str(),
                    status: EscrowStatus::Funded,
                    transfer_group: payload.transfer_group.as_deref(),
                    payment_intent_id: payload.payment_intent_id.as_deref(),
                    charge_id: payload.charge_id.as_deref(),
                    transfer_id: None,
                    stripe_refund_id: None,
                    stripe_dispute_id: None,
                    adjustment_reference: None,
                    actor_user_id: actor_session.as_ref().map(|session| session.user.id),
                    note: payload.note.as_deref(),
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if updated.is_some() {
                publish_payments_event(
                    &state,
                    leg_id,
                    actor_session
                        .as_ref()
                        .map(|session| session.user.id.max(0) as u64),
                    Some(payee_user_id.max(0) as u64),
                    vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                    format!("Stripe webhook funded escrow for load leg #{}.", leg_id),
                );
            }

            Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: updated.is_some(),
                event_type: payload.event_type,
                leg_id: Some(leg_id),
                user_id: None,
                message: "Stripe funding webhook applied through the Rust payments route.".into(),
            })))
        }
        "payment_intent.payment_failed" => {
            let Some((leg_id, payer_user_id, payee_user_id, amount, platform_fee, currency)) =
                resolve_webhook_escrow_context(pool, &payload).await
            else {
                return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                    acknowledged: false,
                    event_type: payload.event_type,
                    leg_id: payload.leg_id,
                    user_id: None,
                    message:
                        "The webhook could not resolve an escrow context for the failed payment."
                            .into(),
                })));
            };

            let updated = apply_escrow_transition(
                pool,
                EscrowTransitionParams {
                    leg_id,
                    payer_user_id,
                    payee_user_id,
                    amount,
                    platform_fee,
                    currency: currency.as_str(),
                    status: EscrowStatus::Failed,
                    transfer_group: payload.transfer_group.as_deref(),
                    payment_intent_id: payload.payment_intent_id.as_deref(),
                    charge_id: payload.charge_id.as_deref(),
                    transfer_id: None,
                    stripe_refund_id: None,
                    stripe_dispute_id: None,
                    adjustment_reference: None,
                    actor_user_id: actor_session.as_ref().map(|session| session.user.id),
                    note: payload.note.as_deref(),
                },
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if updated.is_some() {
                publish_payments_event(
                    &state,
                    leg_id,
                    actor_session
                        .as_ref()
                        .map(|session| session.user.id.max(0) as u64),
                    Some(payee_user_id.max(0) as u64),
                    vec![payer_user_id.max(0) as u64, payee_user_id.max(0) as u64],
                    format!(
                        "Stripe webhook marked escrow as failed for load leg #{}.",
                        leg_id
                    ),
                );
            }

            Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: updated.is_some(),
                event_type: payload.event_type,
                leg_id: Some(leg_id),
                user_id: None,
                message: "Stripe failure webhook applied through the Rust payments route.".into(),
            })))
        }
        "account.updated" => {
            let Some(stripe_account_id) = payload.stripe_account_id.as_deref() else {
                return Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                    acknowledged: false,
                    event_type: payload.event_type,
                    leg_id: None,
                    user_id: None,
                    message: "Stripe account.updated requires stripe_account_id.".into(),
                })));
            };

            let updated_user = update_user_connect_state(
                pool,
                stripe_account_id,
                payload.payouts_enabled.unwrap_or(false),
                payload.kyc_status.as_deref(),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Some(user) = updated_user.as_ref() {
                state.publish_realtime(
                    RoutedRealtimeEvent::new(RealtimeEvent {
                        request_id: None,
                        kind: RealtimeEventKind::PaymentsOperationsUpdated,
                        leg_id: None,
                        conversation_id: None,
                        offer_id: None,
                        message_id: None,
                        actor_user_id: actor_session
                            .as_ref()
                            .map(|session| session.user.id.max(0) as u64),
                        subject_user_id: Some(user.id.max(0) as u64),
                        presence_state: None,
                        last_read_message_id: None,
                        summary: format!(
                            "Stripe account update synced payout readiness for user #{}.",
                            user.id
                        ),
                    })
                    .for_user_ids([user.id.max(0) as u64])
                    .for_permission_keys(["manage_payments"])
                    .with_topics([
                        RealtimeTopic::AdminPayments.as_key(),
                        RealtimeTopic::AdminDashboard.as_key(),
                    ]),
                );
            }

            Ok(Json(ApiResponse::ok(StripeWebhookResponse {
                acknowledged: updated_user.is_some(),
                event_type: payload.event_type,
                leg_id: None,
                user_id: updated_user.as_ref().map(|user| user.id),
                message: "Stripe account update synced through the Rust payments route.".into(),
            })))
        }
        _ => Ok(Json(ApiResponse::ok(StripeWebhookResponse {
            acknowledged: false,
            event_type: payload.event_type,
            leg_id: payload.leg_id,
            user_id: None,
            message: "Unsupported Stripe webhook event for the current Rust payments slice.".into(),
        }))),
    }
}

async fn resolve_payments_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<ResolvedSession>, String> {
    let session = auth_session::resolve_session_from_headers(state, headers).await?;
    Ok(session.filter(|session| {
        session.session.permissions.iter().any(|permission| {
            permission == "manage_payments" || permission == "access_admin_portal"
        })
    }))
}

async fn create_or_refresh_connect_link(
    state: &AppState,
    pool: &db::DbPool,
    user: &db::auth::UserRecord,
    payload: &StripeConnectLinkRequest,
    request_id: Option<&str>,
) -> StripeConnectLinkResponse {
    if !state.stripe.is_configured() {
        return StripeConnectLinkResponse {
            success: false,
            user_id: user.id,
            account_id: user.stripe_connect_account_id.clone(),
            onboarding_url: None,
            message: "STRIPE_SECRET is not configured, so Rust cannot create a live Stripe Connect onboarding link.".into(),
        };
    }

    let account_id = match user
        .stripe_connect_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(existing) => existing.to_string(),
        None => match create_and_store_connect_account(state, pool, user, request_id).await {
            Ok(account_id) => account_id,
            Err(response) => return response,
        },
    };

    match state
        .stripe
        .create_account_link_with_urls(
            &account_id,
            payload.refresh_url.as_deref(),
            payload.return_url.as_deref(),
            request_id,
        )
        .await
    {
        Ok(link) => StripeConnectLinkResponse {
            success: true,
            user_id: user.id,
            account_id: Some(account_id),
            onboarding_url: Some(link.url),
            message: "Stripe Connect onboarding link created through the Rust payments backend."
                .into(),
        },
        Err(error) if should_recreate_missing_connect_account(&error) => {
            let refreshed_account_id =
                match create_and_store_connect_account(state, pool, user, request_id).await {
                    Ok(account_id) => account_id,
                    Err(response) => return response,
                };

            match state
                .stripe
                .create_account_link_with_urls(
                    &refreshed_account_id,
                    payload.refresh_url.as_deref(),
                    payload.return_url.as_deref(),
                    request_id,
                )
                .await
            {
                Ok(link) => StripeConnectLinkResponse {
                    success: true,
                    user_id: user.id,
                    account_id: Some(refreshed_account_id),
                    onboarding_url: Some(link.url),
                    message:
                        "Stripe Connect onboarding link was recreated after clearing a stale Stripe account reference."
                            .into(),
                },
                Err(error) => StripeConnectLinkResponse {
                    success: false,
                    user_id: user.id,
                    account_id: Some(refreshed_account_id),
                    onboarding_url: None,
                    message: format!("Stripe Connect onboarding link creation failed: {}", error),
                },
            }
        }
        Err(error) => StripeConnectLinkResponse {
            success: false,
            user_id: user.id,
            account_id: Some(account_id),
            onboarding_url: None,
            message: format!("Stripe Connect onboarding link creation failed: {}", error),
        },
    }
}

async fn create_and_store_connect_account(
    state: &AppState,
    pool: &db::DbPool,
    user: &db::auth::UserRecord,
    request_id: Option<&str>,
) -> Result<String, StripeConnectLinkResponse> {
    match state
        .stripe
        .create_express_account(&user.email, request_id)
        .await
    {
        Ok(account) => {
            let previous_connect_account_id = user.stripe_connect_account_id.clone();
            if previous_connect_account_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_some_and(|existing| existing != account.id)
            {
                let review_created = sqlx::query(
                    "INSERT INTO payout_destination_change_reviews (
                         carrier_user_id, stripe_connect_account_id, change_type, risk_status,
                         cooling_off_until, requested_by_user_id, notification_sent_at,
                         review_note, created_at, updated_at
                     )
                     VALUES (
                         $1, $2, 'stripe_connect_account_recreated', 'cooling_off',
                         CURRENT_TIMESTAMP + INTERVAL '24 hours', $1, CURRENT_TIMESTAMP,
                         $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                     )",
                )
                .bind(user.id)
                .bind(&account.id)
                .bind(format!(
                    "Carrier payout destination changed from {} to {}; notification queued.",
                    previous_connect_account_id.as_deref().unwrap_or("unknown"),
                    account.id
                ))
                .execute(pool)
                .await
                .map(|result| result.rows_affected() > 0)
                .unwrap_or(false);

                if review_created {
                    let _ = state
                        .email
                        .send_html_with_request_id(
                            &user.email,
                            Some(&user.name),
                            "STLoads payout destination change review",
                            &format!(
                                "<p>Hello {},</p><p>Your STLoads carrier payout destination changed from <strong>{}</strong> to <strong>{}</strong>.</p><p>For your protection, carrier payout release is held for finance review and cooling-off before funds can move to the new destination.</p><p>If you did not request this change, contact STLoads support immediately.</p>",
                                html_escape(&user.name),
                                html_escape(previous_connect_account_id.as_deref().unwrap_or("unknown")),
                                html_escape(&account.id)
                            ),
                            "payout_destination_change_review",
                            request_id,
                        )
                        .await;
                }
            }
            if let Err(error) = set_user_stripe_connect_account_id(pool, user.id, &account.id).await
            {
                return Err(StripeConnectLinkResponse {
                    success: false,
                    user_id: user.id,
                    account_id: Some(account.id),
                    onboarding_url: None,
                    message: format!(
                        "Stripe account was created, but saving it to the Rust database failed: {}",
                        error
                    ),
                });
            }
            Ok(account.id)
        }
        Err(error) => Err(StripeConnectLinkResponse {
            success: false,
            user_id: user.id,
            account_id: None,
            onboarding_url: None,
            message: format!("Stripe Express account creation failed: {}", error),
        }),
    }
}

fn should_recreate_missing_connect_account(error: &str) -> bool {
    let normalized = error.trim().to_ascii_lowercase();
    normalized.contains("no such account:")
        || normalized.contains("no such account ")
        || normalized.contains("resource_missing")
}

async fn authorize_payments_webhook(
    state: &AppState,
    headers: &HeaderMap,
    payload: &[u8],
) -> Result<Option<ResolvedSession>, StatusCode> {
    let configured_secrets = stripe_webhook_secrets(state);
    if !configured_secrets.is_empty() {
        if let Some(signature_header) = headers
            .get("stripe-signature")
            .and_then(|value| value.to_str().ok())
        {
            return verify_stripe_signature(signature_header, payload, &configured_secrets)
                .map(|_| None)
                .map_err(|_| StatusCode::BAD_REQUEST);
        }

        let supplied_secret = headers
            .get("x-stripe-webhook-secret")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if supplied_secret
            .map(|supplied| configured_secrets.contains(&supplied))
            .unwrap_or(false)
        {
            return Ok(None);
        }
    }

    match resolve_payments_session(state, headers).await {
        Ok(Some(session)) => Ok(Some(session)),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn stripe_webhook_secrets(state: &AppState) -> Vec<&str> {
    [
        state.config.stripe_webhook_shared_secret.as_deref(),
        state.config.stripe_webhook_connect_secret.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .collect()
}

fn parse_stripe_webhook_payload(body: &[u8]) -> Result<StripeWebhookRequest, String> {
    if let Ok(payload) = serde_json::from_slice::<StripeWebhookRequest>(body) {
        return Ok(payload);
    }

    let value = serde_json::from_slice::<Value>(body)
        .map_err(|error| format!("Stripe webhook JSON parsing failed: {}", error))?;
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "Stripe webhook event type is missing.".to_string())?
        .to_string();
    let object = value
        .get("data")
        .and_then(|data| data.get("object"))
        .ok_or_else(|| "Stripe webhook data.object is missing.".to_string())?;

    match event_type.as_str() {
        "payment_intent.succeeded" | "payment_intent.payment_failed" => {
            let payment_intent_id = object.get("id").and_then(Value::as_str).map(str::to_string);
            let charge_id = object
                .get("latest_charge")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or_else(|| {
                    object
                        .get("charges")
                        .and_then(|charges| charges.get("data"))
                        .and_then(Value::as_array)
                        .and_then(|items| items.first())
                        .and_then(|charge| charge.get("id"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });

            Ok(StripeWebhookRequest {
                event_id: value.get("id").and_then(Value::as_str).map(str::to_string),
                event_type,
                leg_id: object
                    .get("metadata")
                    .and_then(|metadata| metadata.get("leg_id"))
                    .and_then(|value| value.as_i64().or_else(|| value.as_str()?.parse().ok())),
                payment_intent_id,
                charge_id,
                transfer_id: None,
                transfer_group: object
                    .get("transfer_group")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                amount_cents: object.get("amount").and_then(Value::as_i64),
                currency: object
                    .get("currency")
                    .and_then(Value::as_str)
                    .map(str::to_uppercase),
                platform_fee_cents: object.get("application_fee_amount").and_then(Value::as_i64),
                stripe_account_id: None,
                payouts_enabled: None,
                kyc_status: None,
                note: Some("Parsed from a verified Stripe webhook payload.".into()),
            })
        }
        "account.updated" => {
            let requirements_due = object
                .get("requirements")
                .and_then(|requirements| requirements.get("currently_due"))
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
                .unwrap_or(false);

            Ok(StripeWebhookRequest {
                event_id: value.get("id").and_then(Value::as_str).map(str::to_string),
                event_type,
                leg_id: None,
                payment_intent_id: None,
                charge_id: None,
                transfer_id: None,
                transfer_group: None,
                amount_cents: None,
                currency: None,
                platform_fee_cents: None,
                stripe_account_id: object.get("id").and_then(Value::as_str).map(str::to_string),
                payouts_enabled: object.get("payouts_enabled").and_then(Value::as_bool),
                kyc_status: Some(if requirements_due {
                    "pending".into()
                } else {
                    "verified".into()
                }),
                note: Some("Parsed from a verified Stripe account webhook payload.".into()),
            })
        }
        _ => Ok(StripeWebhookRequest {
            event_id: value.get("id").and_then(Value::as_str).map(str::to_string),
            event_type,
            leg_id: None,
            payment_intent_id: None,
            charge_id: None,
            transfer_id: None,
            transfer_group: None,
            amount_cents: None,
            currency: None,
            platform_fee_cents: None,
            stripe_account_id: None,
            payouts_enabled: None,
            kyc_status: None,
            note: Some("Unsupported Stripe event parsed for acknowledgement routing.".into()),
        }),
    }
}

fn verify_stripe_signature(
    signature_header: &str,
    payload: &[u8],
    secrets: &[&str],
) -> Result<(), String> {
    let timestamp = parse_stripe_signature_part(signature_header, "t")
        .ok_or_else(|| "Stripe signature timestamp is missing.".to_string())?;
    let signatures = parse_stripe_signature_parts(signature_header, "v1");
    if signatures.is_empty() {
        return Err("Stripe v1 signature is missing.".into());
    }

    let timestamp_seconds = timestamp
        .parse::<i64>()
        .map_err(|_| "Stripe signature timestamp is invalid.".to_string())?;
    let now_seconds = chrono::Utc::now().timestamp();
    if (now_seconds - timestamp_seconds).abs() > 300 {
        return Err("Stripe signature timestamp is outside the allowed tolerance.".into());
    }

    let signed_payload = [timestamp.as_bytes(), b".", payload].concat();
    for secret in secrets {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|_| "Stripe webhook secret is invalid.".to_string())?;
        mac.update(&signed_payload);
        let expected = mac.finalize().into_bytes();

        for signature in &signatures {
            if let Ok(decoded) = decode_hex(signature)
                && mac_verify_bytes(expected.as_slice(), decoded.as_slice())
            {
                return Ok(());
            }
        }
    }

    Err("Stripe signature verification failed.".into())
}

fn parse_stripe_signature_part<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    parse_stripe_signature_parts(header, key).into_iter().next()
}

fn parse_stripe_signature_parts<'a>(header: &'a str, key: &str) -> Vec<&'a str> {
    header
        .split(',')
        .filter_map(|part| {
            let (part_key, value) = part.trim().split_once('=')?;
            (part_key == key).then_some(value.trim())
        })
        .filter(|value| !value.is_empty())
        .collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err("Hex value has an odd length.".into());
    }

    let mut bytes = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = hex_value(pair[0])?;
        let low = hex_value(pair[1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("Invalid hex digit.".into()),
    }
}

fn mac_verify_bytes(expected: &[u8], supplied: &[u8]) -> bool {
    if expected.len() != supplied.len() {
        return false;
    }

    expected
        .iter()
        .zip(supplied.iter())
        .fold(0_u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

async fn resolve_webhook_escrow_context(
    pool: &db::DbPool,
    payload: &StripeWebhookRequest,
) -> Option<(i64, i64, i64, i64, i64, String)> {
    if let Some(payment_intent_id) = payload.payment_intent_id.as_deref()
        && let Some(existing_escrow) = find_escrow_by_payment_intent_id(pool, payment_intent_id)
            .await
            .ok()
            .flatten()
    {
        return Some((
            existing_escrow.leg_id,
            existing_escrow.payer_user_id,
            existing_escrow.payee_user_id,
            payload.amount_cents.unwrap_or(existing_escrow.amount),
            payload
                .platform_fee_cents
                .unwrap_or(existing_escrow.platform_fee),
            payload.currency.clone().unwrap_or(existing_escrow.currency),
        ));
    }

    let leg_id = payload.leg_id?;
    let scope = find_load_leg_scope(pool, leg_id).await.ok().flatten()?;
    let leg = find_load_leg_by_id(pool, leg_id).await.ok().flatten()?;
    let payee_user_id = leg.booked_carrier_id?;
    let payer_user_id = scope.load_owner_user_id?;
    let amount = payload
        .amount_cents
        .or_else(|| leg.booked_amount.or(leg.price).map(currency_to_cents))?;

    Some((
        leg_id,
        payer_user_id,
        payee_user_id,
        amount,
        payload.platform_fee_cents.unwrap_or(0),
        payload.currency.clone().unwrap_or_else(|| "USD".into()),
    ))
}

fn can_manage_leg_payments(
    session: &ResolvedSession,
    load_owner_user_id: Option<i64>,
    organization_id: i64,
) -> bool {
    crate::auth_session::session_matches_organization(session, organization_id)
        && (session.user.primary_role() == Some(UserRole::Admin)
            || load_owner_user_id == Some(session.user.id))
}

async fn shipper_credit_blocks_release(pool: &db::DbPool, payer_user_id: i64) -> bool {
    sqlx::query_scalar::<_, bool>(
        "WITH ar AS (
             SELECT customer_user_id,
                    COALESCE(SUM(CASE WHEN status IN ('issued', 'partially_paid') THEN total_amount_cents ELSE 0 END), 0)::BIGINT AS open_ar_cents,
                    COALESCE(SUM(CASE WHEN status IN ('issued', 'partially_paid') AND due_at < CURRENT_TIMESTAMP THEN total_amount_cents ELSE 0 END), 0)::BIGINT AS overdue_ar_cents
             FROM customer_invoices
             WHERE customer_user_id = $1
             GROUP BY customer_user_id
         )
         SELECT EXISTS (
            SELECT 1
            FROM shipper_credit_accounts credit
            LEFT JOIN ar ON ar.customer_user_id = credit.customer_user_id
            WHERE credit.customer_user_id = $1
              AND (
                  credit.credit_hold = TRUE
                  OR credit.override_required = TRUE
                  OR credit.credit_status IN ('over_limit', 'hold', 'collections')
                  OR (credit.credit_limit_cents > 0 AND COALESCE(ar.open_ar_cents, credit.open_ar_cents) > credit.credit_limit_cents)
                  OR COALESCE(ar.overdue_ar_cents, credit.overdue_ar_cents) > 0
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM shipper_credit_override_requests override_request
                  WHERE override_request.credit_account_id = credit.id
                    AND override_request.status = 'approved'
                    AND (override_request.expires_at IS NULL OR override_request.expires_at > CURRENT_TIMESTAMP)
              )
        )",
    )
    .bind(payer_user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

async fn payout_destination_blocks_release(pool: &db::DbPool, carrier_user_id: i64) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1
            FROM payout_destination_change_reviews
            WHERE carrier_user_id = $1
              AND risk_status IN ('review_required', 'cooling_off', 'blocked')
        )",
    )
    .bind(carrier_user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

async fn compliance_payout_blocks_release(
    pool: &db::DbPool,
    payee_user_id: i64,
    leg_id: i64,
) -> Option<String> {
    let sanctions_tax_blocker = sqlx::query_scalar::<_, Option<String>>(
        "WITH profile AS (
             SELECT sanctions_status, beneficial_owner_status, tax_document_status, payout_tax_blocking
             FROM sanctions_tax_profiles
             WHERE user_id = $1
             LIMIT 1
         )
         SELECT CASE
             WHEN NOT EXISTS (SELECT 1 FROM profile)
                 THEN 'Payout tax and sanctions profile is missing.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE sanctions_status IN ('possible_match', 'blocked'))
                 THEN 'Payout blocked by sanctions screening.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE beneficial_owner_status = 'blocked')
                 THEN 'Payout blocked by beneficial owner review.'
             WHEN EXISTS (SELECT 1 FROM profile WHERE payout_tax_blocking = TRUE AND tax_document_status NOT IN ('verified', 'not_required'))
                 THEN 'Payout blocked until tax documentation is verified or assigned not required.'
             ELSE NULL
         END",
    )
    .bind(payee_user_id)
    .fetch_one(pool)
    .await
    .ok()
    .flatten();
    if sanctions_tax_blocker.is_some() {
        return sanctions_tax_blocker;
    }

    sqlx::query_scalar::<_, Option<String>>(
        "SELECT CONCAT('Payout blocked by ', review_type, ' review: ', array_to_string(reasons, '; '))
         FROM risk_review_items
         WHERE hold_payout = TRUE
           AND status IN ('open', 'in_review', 'blocked')
           AND (subject_user_id = $1 OR leg_id = $2)
         ORDER BY
           CASE severity WHEN 'critical' THEN 4 WHEN 'high' THEN 3 WHEN 'medium' THEN 2 ELSE 1 END DESC,
           score DESC,
           created_at DESC
         LIMIT 1",
    )
    .bind(payee_user_id)
    .bind(leg_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
}

fn publish_payments_event(
    state: &AppState,
    leg_id: i64,
    actor_user_id: Option<u64>,
    subject_user_id: Option<u64>,
    target_user_ids: Vec<u64>,
    summary: String,
) {
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            request_id: None,
            kind: RealtimeEventKind::PaymentsOperationsUpdated,
            leg_id: Some(leg_id.max(0) as u64),
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id,
            subject_user_id,
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_user_ids(target_user_ids)
        .for_permission_keys(["manage_payments"])
        .with_topics([
            RealtimeTopic::AdminPayments.as_key(),
            RealtimeTopic::AdminDashboard.as_key(),
            RealtimeTopic::LoadBoard.as_key(),
        ]),
    );
}

fn currency_to_cents(value: f64) -> i64 {
    (value * 100.0).round() as i64
}

fn accounting_export_header() -> String {
    "ledger_id,created_at,entry_type,direction,currency,amount_cents,platform_fee_cents,load_id,leg_id,escrow_id,payment_intent_id,charge_id,transfer_id,invoice_number,invoice_status,settlement_number,settlement_status".into()
}

fn accounting_rows_to_csv(rows: &[AccountingExportRow]) -> String {
    let mut csv = accounting_export_header();
    for row in rows {
        csv.push('\n');
        csv.push_str(
            &[
                row.ledger_id.to_string(),
                row.created_at.to_string(),
                row.entry_type.clone(),
                row.direction.clone(),
                row.currency.clone(),
                row.amount_cents.to_string(),
                row.platform_fee_cents.to_string(),
                row.load_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.leg_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.escrow_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                row.payment_intent_id.clone().unwrap_or_default(),
                row.charge_id.clone().unwrap_or_default(),
                row.transfer_id.clone().unwrap_or_default(),
                row.invoice_number.clone().unwrap_or_default(),
                row.invoice_status.clone().unwrap_or_default(),
                row.settlement_number.clone().unwrap_or_default(),
                row.settlement_status.clone().unwrap_or_default(),
            ]
            .into_iter()
            .map(|value| format!("\"{}\"", csv_escape(&value)))
            .collect::<Vec<_>>()
            .join(","),
        );
    }
    csv
}

fn csv_escape(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn normalize_supported_freight_payment_currency(value: Option<&str>) -> Result<String, String> {
    let currency = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("USD")
        .to_ascii_uppercase();
    if currency == "USD" {
        return Ok(currency);
    }

    Err(format!(
        "Freight payment currency '{}' is not supported for the first enterprise release. Non-USD rating, invoicing, settlement, FX, duties, taxes, customs fees, and Incoterms remain deferred until Legal and Finance approve cross-border financial rules.",
        currency
    ))
}

fn unavailable_payment_response(
    state: &AppState,
    leg_id: i64,
    action_label: &str,
) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        payment_intent_id: None,
        client_secret: None,
        transfer_id: None,
        status_label: "Unavailable".into(),
        message: format!(
            "{} action is unavailable because the database is {} on {}.",
            action_label,
            state.database_state(),
            state.config.deployment_target
        ),
    }
}

fn missing_leg_response(leg_id: i64) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        payment_intent_id: None,
        client_secret: None,
        transfer_id: None,
        status_label: "Missing".into(),
        message: "The requested load leg was not found.".into(),
    }
}

fn forbidden_payment_response(leg_id: i64, message: &str) -> EscrowLifecycleResponse {
    EscrowLifecycleResponse {
        success: false,
        leg_id,
        escrow_id: None,
        payment_intent_id: None,
        client_secret: None,
        transfer_id: None,
        status_label: "Forbidden".into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_valid_stripe_signature() {
        let payload = br#"{"event_type":"payment_intent.succeeded","payment_intent_id":"pi_test"}"#;
        let secret = "whsec_test";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let signature = sign_test_payload(secret, &timestamp, payload);
        let header = format!("t={},v1={}", timestamp, signature);

        assert!(verify_stripe_signature(&header, payload, &[secret]).is_ok());
    }

    #[test]
    fn rejects_invalid_stripe_signature() {
        let payload = br#"{"event_type":"payment_intent.succeeded","payment_intent_id":"pi_test"}"#;
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let header = format!("t={},v1=deadbeef", timestamp);

        assert!(verify_stripe_signature(&header, payload, &["whsec_test"]).is_err());
    }

    #[test]
    fn parses_real_stripe_payment_intent_payload() {
        let payload = br#"{
            "id": "evt_test",
            "type": "payment_intent.succeeded",
            "data": {
                "object": {
                    "id": "pi_test",
                    "amount": 125000,
                    "currency": "usd",
                    "latest_charge": "ch_test",
                    "transfer_group": "leg_9301",
                    "application_fee_amount": 1250,
                    "metadata": { "leg_id": "9311" }
                }
            }
        }"#;

        let parsed = parse_stripe_webhook_payload(payload).expect("payload should parse");

        assert_eq!(parsed.event_id.as_deref(), Some("evt_test"));
        assert_eq!(parsed.event_type, "payment_intent.succeeded");
        assert_eq!(parsed.payment_intent_id.as_deref(), Some("pi_test"));
        assert_eq!(parsed.charge_id.as_deref(), Some("ch_test"));
        assert_eq!(parsed.leg_id, Some(9311));
        assert_eq!(parsed.amount_cents, Some(125000));
        assert_eq!(parsed.currency.as_deref(), Some("USD"));
        assert_eq!(parsed.platform_fee_cents, Some(1250));
    }

    #[test]
    fn detects_stale_connect_account_errors() {
        assert!(should_recreate_missing_connect_account(
            "Stripe API returned 400 Bad Request: No such account: 'acct_123'"
        ));
        assert!(should_recreate_missing_connect_account(
            "Stripe API returned 404 Not Found: resource_missing"
        ));
        assert!(!should_recreate_missing_connect_account(
            "Stripe API returned 400 Bad Request: Invalid email"
        ));
    }

    #[test]
    fn freight_payment_currency_is_usd_only_until_cross_border_finance_is_approved() {
        assert_eq!(
            normalize_supported_freight_payment_currency(None).as_deref(),
            Ok("USD")
        );
        assert_eq!(
            normalize_supported_freight_payment_currency(Some(" usd ")).as_deref(),
            Ok("USD")
        );

        let error = normalize_supported_freight_payment_currency(Some("CAD"))
            .expect_err("non-USD currency should be blocked");
        assert!(error.contains("Non-USD rating"));
        assert!(error.contains("Legal and Finance"));
    }

    fn sign_test_payload(secret: &str, timestamp: &str, payload: &[u8]) -> String {
        let signed_payload = [timestamp.as_bytes(), b".", payload].concat();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("test secret");
        mac.update(&signed_payload);
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>()
    }
}
