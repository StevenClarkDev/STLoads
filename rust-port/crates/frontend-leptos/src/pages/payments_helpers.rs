use leptos::prelude::*;

use super::shared::{
    MUTED_TEXT_STYLE, ROW_BORDER_STYLE, TABLE_CELL_STYLE, TABLE_HEAD_CELL_STYLE,
    TABLE_HEADER_STYLE, TABLE_OVERFLOW_STYLE, tone_style,
};
use crate::api::{PayoutReviewRow, PlatformBillingRow, ShipperCreditRow};

pub(super) fn render_platform_billing(
    platform_billing: RwSignal<Vec<PlatformBillingRow>>,
    pending_action: RwSignal<Option<String>>,
    generate_platform_invoices: impl Fn(leptos::ev::MouseEvent) + Copy + Send + Sync + 'static,
    mark_platform_invoice_paid: impl Fn(i64) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <section style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                <strong>"STLoads Platform Billing"</strong>
                <button
                    type="button"
                    on:click=generate_platform_invoices
                    disabled=move || pending_action.get().is_some()
                    style="padding:0.55rem 0.8rem;border:1px solid #334155;border-radius:0.75rem;background:#f8fafc;color:#0f172a;cursor:pointer;"
                >
                    {move || if pending_action.get().as_deref() == Some("platform-invoices") { "Generating..." } else { "Generate invoices" }}
                </button>
            </div>
            <div style=TABLE_OVERFLOW_STYLE>
                <table style="width:100%;border-collapse:collapse;min-width:1060px;">
                    <thead style=TABLE_HEADER_STYLE>
                        <tr>
                            <th style=TABLE_HEAD_CELL_STYLE>"Account"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Plan"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Billing"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Payment method"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Latest invoice"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Open"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Past due"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = platform_billing.get();
                            if rows.is_empty() {
                                vec![view! {
                                    <tr>
                                        <td colspan="8" style="padding:1rem;color:#64748b;">"No platform billing accounts yet."</td>
                                    </tr>
                                }.into_any()]
                            } else {
                                rows.into_iter().map(|row| {
                                    let billing_tone = if row.billing_status == "active" { "success" } else { "warning" };
                                    let latest_invoice_id = row.latest_invoice_id;
                                    let paid_key = latest_invoice_id.map(|id| format!("platform-paid:{id}"));
                                    let invoice_status_label = row
                                        .latest_invoice_status
                                        .unwrap_or_else(|| "none".into());
                                    let invoice_status_tone = if invoice_status_label == "paid" {
                                        "success"
                                    } else {
                                        "info"
                                    };
                                    let can_mark_paid = latest_invoice_id.is_some()
                                        && invoice_status_label != "paid";
                                    view! {
                                        <tr style=ROW_BORDER_STYLE>
                                            <td style=TABLE_CELL_STYLE>
                                                <div style="font-weight:600;">{format!("#{}", row.billing_account_id)}</div>
                                                <span style=MUTED_TEXT_STYLE>{row.customer_user_id.map(|id| format!("Customer {}", id)).unwrap_or_else(|| "Organization account".into())}</span>
                                            </td>
                                            <td style=TABLE_CELL_STYLE>{row.plan_name.unwrap_or_else(|| "Unassigned".into())}</td>
                                            <td style=TABLE_CELL_STYLE><span style=tone_style(billing_tone)>{row.billing_status}</span></td>
                                            <td style=TABLE_CELL_STYLE>{row.payment_method_status}</td>
                                            <td style=TABLE_CELL_STYLE>
                                                <div style="font-weight:600;">{row.latest_invoice_number.unwrap_or_else(|| "-".into())}</div>
                                                <span style=tone_style(invoice_status_tone)>{invoice_status_label}</span>
                                            </td>
                                            <td style=TABLE_CELL_STYLE>{format!("${:.2}", row.open_invoice_cents as f64 / 100.0)}</td>
                                            <td style=TABLE_CELL_STYLE>{format!("${:.2}", row.past_due_invoice_cents as f64 / 100.0)}</td>
                                            <td style=TABLE_CELL_STYLE>
                                                <button
                                                    type="button"
                                                    on:click=move |_| {
                                                        if let Some(invoice_id) = latest_invoice_id {
                                                            mark_platform_invoice_paid(invoice_id);
                                                        }
                                                    }
                                                    disabled=move || pending_action.get().is_some() || !can_mark_paid
                                                    style="padding:0.5rem 0.75rem;border:1px solid #0f766e;border-radius:0.75rem;background:#ecfdf5;color:#0f766e;cursor:pointer;"
                                                >
                                                    {move || {
                                                        if paid_key.as_deref().is_some_and(|key| pending_action.get().as_deref() == Some(key)) {
                                                            "Marking..."
                                                        } else {
                                                            "Mark paid"
                                                        }
                                                    }}
                                                </button>
                                            </td>
                                        </tr>
                                    }.into_any()
                                }).collect::<Vec<_>>()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </section>
    }
}

pub(super) fn render_shipper_credit(
    shipper_credit: RwSignal<Vec<ShipperCreditRow>>,
    pending_action: RwSignal<Option<String>>,
    approve_credit_override: impl Fn(i64) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <section style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                <strong>"Shipper Credit And AR"</strong>
                <span style=tone_style(if shipper_credit.get().iter().any(|row| row.credit_hold || row.override_required || row.overdue_ar_cents > 0) { "warning" } else { "success" })>
                    {move || format!("{} accounts", shipper_credit.get().len())}
                </span>
            </div>
            <div style=TABLE_OVERFLOW_STYLE>
                <table style="width:100%;border-collapse:collapse;min-width:1040px;">
                    <thead style=TABLE_HEADER_STYLE>
                        <tr>
                            <th style=TABLE_HEAD_CELL_STYLE>"Customer"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Status"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Limit"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Open AR"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Overdue"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Terms"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = shipper_credit.get();
                            if rows.is_empty() {
                                vec![view! {
                                    <tr>
                                        <td colspan="7" style="padding:1rem;color:#64748b;">"No shipper credit accounts yet."</td>
                                    </tr>
                                }.into_any()]
                            } else {
                                rows.into_iter().map(|row| {
                                    let action_key = format!("credit-override:{}", row.credit_account_id);
                                    let needs_override = row.credit_hold || row.override_required || row.credit_status == "collections" || row.credit_status == "over_limit";
                                    view! {
                                        <tr style=ROW_BORDER_STYLE>
                                            <td style=TABLE_CELL_STYLE>
                                                <div style="font-weight:600;">{row.customer_user_id.map(|id| format!("Customer {}", id)).unwrap_or_else(|| format!("Credit #{}", row.credit_account_id))}</div>
                                                <span style=MUTED_TEXT_STYLE>{row.internal_risk_note.unwrap_or_else(|| "No active risk note".into())}</span>
                                            </td>
                                            <td style=TABLE_CELL_STYLE><span style=tone_style(if needs_override { "warning" } else { "success" })>{row.credit_status}</span></td>
                                            <td style=TABLE_CELL_STYLE>{format!("${:.2}", row.credit_limit_cents as f64 / 100.0)}</td>
                                            <td style=TABLE_CELL_STYLE>{format!("${:.2}", row.open_ar_cents as f64 / 100.0)}</td>
                                            <td style=TABLE_CELL_STYLE>{format!("${:.2}", row.overdue_ar_cents as f64 / 100.0)}</td>
                                            <td style=TABLE_CELL_STYLE>{format!("{} days", row.payment_terms_days)}</td>
                                            <td style=TABLE_CELL_STYLE>
                                                <button
                                                    type="button"
                                                    on:click=move |_| approve_credit_override(row.credit_account_id)
                                                    disabled=move || pending_action.get().is_some() || !needs_override
                                                    style="padding:0.5rem 0.75rem;border:1px solid #b45309;border-radius:0.75rem;background:#fff7ed;color:#92400e;cursor:pointer;"
                                                >
                                                    {move || if pending_action.get().as_deref() == Some(action_key.as_str()) { "Approving..." } else { "Approve override" }}
                                                </button>
                                            </td>
                                        </tr>
                                    }.into_any()
                                }).collect::<Vec<_>>()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </section>
    }
}

pub(super) fn render_payout_reviews(
    payout_reviews: RwSignal<Vec<PayoutReviewRow>>,
    pending_action: RwSignal<Option<String>>,
    decide_payout_review: impl Fn(i64, &'static str) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <section style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                <strong>"Payout Destination Reviews"</strong>
                <span style=tone_style(if payout_reviews.get().is_empty() { "success" } else { "warning" })>
                    {move || format!("{} open", payout_reviews.get().len())}
                </span>
            </div>
            <div style=TABLE_OVERFLOW_STYLE>
                <table style="width:100%;border-collapse:collapse;min-width:1040px;">
                    <thead style=TABLE_HEADER_STYLE>
                        <tr>
                            <th style=TABLE_HEAD_CELL_STYLE>"Carrier"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Stripe account"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Change"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Status"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Notification"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Cooling off"</th>
                            <th style=TABLE_HEAD_CELL_STYLE>"Action"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = payout_reviews.get();
                            if rows.is_empty() {
                                vec![view! {
                                    <tr>
                                        <td colspan="7" style="padding:1rem;color:#64748b;">"No open payout destination reviews."</td>
                                    </tr>
                                }.into_any()]
                            } else {
                                rows.into_iter().map(|row| {
                                    let approve_key = format!("payout-review:approve:{}", row.review_id);
                                    let reject_key = format!("payout-review:reject:{}", row.review_id);
                                    let decide_payout_review_reject = decide_payout_review;
                                    view! {
                                        <tr style=ROW_BORDER_STYLE>
                                            <td style=TABLE_CELL_STYLE>{format!("Carrier {}", row.carrier_user_id)}</td>
                                            <td style=TABLE_CELL_STYLE>{row.stripe_connect_account_id.unwrap_or_else(|| "-".into())}</td>
                                            <td style=TABLE_CELL_STYLE>{row.change_type}</td>
                                            <td style=TABLE_CELL_STYLE><span style=tone_style("warning")>{row.risk_status}</span></td>
                                            <td style=TABLE_CELL_STYLE>{row.notification_sent_at.unwrap_or_else(|| "Pending".into())}</td>
                                            <td style=TABLE_CELL_STYLE>{row.cooling_off_until.unwrap_or_else(|| "-".into())}</td>
                                            <td style=TABLE_CELL_STYLE>
                                                <div style="display:flex;gap:0.45rem;flex-wrap:wrap;">
                                                    <button
                                                        type="button"
                                                        on:click=move |_| decide_payout_review(row.review_id, "approve")
                                                        disabled=move || pending_action.get().is_some()
                                                        style="padding:0.5rem 0.75rem;border:1px solid #0f766e;border-radius:0.75rem;background:#ecfdf5;color:#0f766e;cursor:pointer;"
                                                    >
                                                        {move || if pending_action.get().as_deref() == Some(approve_key.as_str()) { "Approving..." } else { "Approve" }}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        on:click=move |_| decide_payout_review_reject(row.review_id, "reject")
                                                        disabled=move || pending_action.get().is_some()
                                                        style="padding:0.5rem 0.75rem;border:1px solid #be123c;border-radius:0.75rem;background:#fff1f2;color:#be123c;cursor:pointer;"
                                                    >
                                                        {move || if pending_action.get().as_deref() == Some(reject_key.as_str()) { "Rejecting..." } else { "Reject" }}
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }.into_any()
                                }).collect::<Vec<_>>()
                            }
                        }}
                    </tbody>
                </table>
            </div>
        </section>
    }
}
