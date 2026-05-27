use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_query_map};
use shared::{
    EscrowFundRequest, EscrowHoldRequest, EscrowReleaseRequest, RealtimeEventKind, RealtimeTopic,
    StripeWebhookRequest,
};

use crate::{
    api::{
        self, CreditOverrideRequest, EscrowStatusDescriptorLite, FinanceApprovalActionRequest,
        FinanceApprovalQueueItem, InvoiceSettlementQueueItem, PaymentAdjustmentRequest,
        PaymentsOverview, PayoutReviewDecisionRequest, PayoutReviewRow, PlatformBillingRow,
        ShipperCreditRow, StripeWebhookEventDescriptorLite,
    },
    realtime,
    session::{self, use_auth},
};

use super::{
    admin_guard_view,
    payments_helpers::{render_payout_reviews, render_platform_billing, render_shipper_credit},
    shared::{
        FIELD_INPUT_STYLE, FIELD_LABEL_STYLE, ROW_BORDER_STYLE, TABLE_CELL_STYLE,
        TABLE_HEAD_CELL_STYLE, TABLE_HEADER_STYLE, TABLE_OVERFLOW_STYLE, parse_optional_i64,
        parse_required_u64, tone_style,
    },
};

#[component]
pub fn EscrowOperationsPage() -> impl IntoView {
    let auth = use_auth();
    let query = use_query_map();
    let overview = RwSignal::new(None::<PaymentsOverview>);
    let statuses = RwSignal::new(Vec::<EscrowStatusDescriptorLite>::new());
    let webhooks = RwSignal::new(Vec::<StripeWebhookEventDescriptorLite>::new());
    let approvals = RwSignal::new(Vec::<FinanceApprovalQueueItem>::new());
    let invoice_settlements = RwSignal::new(Vec::<InvoiceSettlementQueueItem>::new());
    let platform_billing = RwSignal::new(Vec::<PlatformBillingRow>::new());
    let shipper_credit = RwSignal::new(Vec::<ShipperCreditRow>::new());
    let payout_reviews = RwSignal::new(Vec::<PayoutReviewRow>::new());
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);
    let pending_action = RwSignal::new(None::<String>);

    let leg_id = RwSignal::new(String::new());
    let amount_cents = RwSignal::new(String::new());
    let platform_fee_cents = RwSignal::new(String::new());
    let currency = RwSignal::new("USD".to_string());
    let payment_intent_id = RwSignal::new(String::new());
    let transfer_id = RwSignal::new(String::new());
    let note = RwSignal::new(String::new());
    let webhook_event_type = RwSignal::new("payment_intent.succeeded".to_string());
    let stripe_account_id = RwSignal::new(String::new());
    let payouts_enabled = RwSignal::new(String::new());
    let kyc_status = RwSignal::new(String::new());
    let applied_prefill = RwSignal::new(None::<String>);

    let prefill_context = Memo::new(
        move |_| -> (Option<String>, String, String, Option<String>) {
            query.with(|params| {
                let leg_id = params.get("leg_id");
                let action = params.get("action");
                let source = params.get("source");
                let load_id = params.get("load_id");
                (
                    leg_id,
                    action.unwrap_or_default(),
                    source.unwrap_or_default(),
                    load_id,
                )
            })
        },
    );

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_payments")
    });

    Effect::new(move |_| {
        let (prefill_leg_id, prefill_action, prefill_source, prefill_load_id) =
            prefill_context.get();
        let prefill_key = format!(
            "{}|{}|{}|{}",
            prefill_leg_id.clone().unwrap_or_default(),
            prefill_action,
            prefill_source,
            prefill_load_id.clone().unwrap_or_default()
        );

        if prefill_key == "|||" || applied_prefill.get() == Some(prefill_key.clone()) {
            return;
        }

        if let Some(leg_id_value) = prefill_leg_id.filter(|value| !value.trim().is_empty()) {
            leg_id.set(leg_id_value);
        }

        if !prefill_action.trim().is_empty() && note.get().trim().is_empty() {
            let source_label = match prefill_source.as_str() {
                "admin-load-profile" => "admin load profile",
                "admin-loads" => "admin loads",
                "dispatch-closeout" => "dispatch closeout desk",
                "dispatch-collections" => "dispatch collections desk",
                other if !other.is_empty() => other,
                _ => "admin shortcut",
            };
            let action_label = match prefill_action.as_str() {
                "fund" => "fund escrow",
                "hold" => "place escrow on hold",
                "release" => "release escrow",
                other => other,
            };
            let load_fragment = prefill_load_id
                .map(|value| format!(" for load #{}", value))
                .unwrap_or_default();
            note.set(format!(
                "Loaded from {}{} to {}.",
                source_label, load_fragment, action_label
            ));
        }

        applied_prefill.set(Some(prefill_key));
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let _refresh = refresh_nonce.get();

        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        is_loading.set(true);
        let auth = auth;

        spawn_local(async move {
            let overview_result = api::fetch_payments_overview().await;
            let statuses_result = api::fetch_escrow_status_catalog().await;
            let webhooks_result = api::fetch_stripe_webhook_event_catalog().await;
            let approvals_result = api::fetch_release_approval_queue().await;
            let invoice_settlements_result = api::fetch_invoice_settlement_queue().await;
            let platform_billing_result = api::fetch_platform_billing_accounts().await;
            let shipper_credit_result = api::fetch_shipper_credit_accounts().await;
            let payout_reviews_result = api::fetch_payout_change_reviews().await;

            let mut maybe_error = None::<String>;
            match overview_result {
                Ok(data) => overview.set(Some(data)),
                Err(error) => maybe_error = Some(error),
            }
            match statuses_result {
                Ok(data) => statuses.set(data),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }
            match webhooks_result {
                Ok(data) => webhooks.set(data),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }
            match approvals_result {
                Ok(data) => approvals.set(data.approvals),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }
            match invoice_settlements_result {
                Ok(data) => invoice_settlements.set(data.rows),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }
            match platform_billing_result {
                Ok(data) => platform_billing.set(data.rows),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }
            match shipper_credit_result {
                Ok(data) => shipper_credit.set(data.rows),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }
            match payout_reviews_result {
                Ok(data) => payout_reviews.set(data.rows),
                Err(error) if maybe_error.is_none() => maybe_error = Some(error),
                Err(_) => {}
            }

            if let Some(error) = maybe_error {
                if error.contains("returned 401") {
                    session::invalidate_session(&auth, "Your Rust session expired; sign in again.");
                }
                error_message.set(Some(error));
            } else {
                error_message.set(None);
            }

            is_loading.set(false);
        });
    });

    Effect::new(move |_| {
        let current_session = auth.session.get();
        if !auth.session_ready.get() || !current_session.authenticated || !can_view.get() {
            if let Some(existing_handle) = ws_handle.get_untracked() {
                existing_handle.abort();
                ws_handle.set(None);
            }
            ws_connected.set(false);
            return;
        }

        let current_user_id = current_session.user.as_ref().map(|user| user.id);
        let auth = auth;
        if let Some(existing_handle) = ws_handle.get_untracked() {
            existing_handle.abort();
        }

        let handle = realtime::connect_realtime_listener(
            None,
            vec![RealtimeTopic::AdminPayments],
            move |event| match event.kind {
                RealtimeEventKind::SessionInvalidated => {
                    if event.actor_user_id == current_user_id {
                        if let Some(existing_handle) = ws_handle.get_untracked() {
                            existing_handle.abort();
                            ws_handle.set(None);
                        }
                        session::invalidate_session(
                            &auth,
                            "The current Rust session was invalidated; sign in again.",
                        );
                        ws_connected.set(false);
                    }
                }
                RealtimeEventKind::PaymentsOperationsUpdated => {
                    refresh_nonce.update(|value| *value += 1);
                    action_message.set(Some(format!("Realtime update: {}", event.summary)));
                }
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    let fund_escrow = move |_| {
        let parsed_leg_id = match parse_required_u64(&leg_id.get(), "leg id") {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };
        let parsed_amount = match parse_optional_i64(&amount_cents.get()) {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };
        let parsed_platform_fee = match parse_optional_i64(&platform_fee_cents.get()) {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };

        pending_action.set(Some("fund".into()));
        action_message.set(None);
        let auth = auth;
        let request = EscrowFundRequest {
            idempotency_key: None,
            amount_cents: parsed_amount,
            currency: Some(currency.get()),
            platform_fee_cents: parsed_platform_fee,
            payment_intent_id: (!payment_intent_id.get().trim().is_empty())
                .then(|| payment_intent_id.get()),
            charge_id: None,
            transfer_group: None,
            note: (!note.get().trim().is_empty()).then(|| note.get()),
        };

        spawn_local(async move {
            match api::fund_escrow(parsed_leg_id, &request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let hold_escrow = move |_| {
        let parsed_leg_id = match parse_required_u64(&leg_id.get(), "leg id") {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };

        pending_action.set(Some("hold".into()));
        action_message.set(None);
        let auth = auth;
        let request = EscrowHoldRequest {
            idempotency_key: None,
            note: (!note.get().trim().is_empty()).then(|| note.get()),
        };

        spawn_local(async move {
            match api::hold_escrow(parsed_leg_id, &request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let release_escrow = move |_| {
        let parsed_leg_id = match parse_required_u64(&leg_id.get(), "leg id") {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };

        pending_action.set(Some("release".into()));
        action_message.set(None);
        let auth = auth;
        let request = EscrowReleaseRequest {
            idempotency_key: None,
            transfer_id: (!transfer_id.get().trim().is_empty()).then(|| transfer_id.get()),
            note: (!note.get().trim().is_empty()).then(|| note.get()),
        };

        spawn_local(async move {
            match api::release_escrow(parsed_leg_id, &request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let approve_finance_request = move |target_leg_id: i64, approval_type: String| {
        if target_leg_id <= 0 {
            action_message.set(Some("Approval row is missing a valid leg id.".into()));
            return;
        }
        pending_action.set(Some(format!("approve:{approval_type}:{target_leg_id}")));
        action_message.set(None);
        let auth = auth;
        let note_value = (!note.get().trim().is_empty()).then(|| note.get());
        spawn_local(async move {
            let request = FinanceApprovalActionRequest { note: note_value };
            let result = if approval_type == "escrow_hold" {
                api::approve_hold_escrow(target_leg_id as u64, &request).await
            } else {
                api::approve_release_escrow(target_leg_id as u64, &request).await
            };
            match result {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    refresh_nonce.update(|value| *value += 1);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let record_adjustment = move |target_leg_id: i64| {
        if target_leg_id <= 0 {
            action_message.set(Some("Adjustment row is missing a valid leg id.".into()));
            return;
        }
        let parsed_amount = match parse_optional_i64(&amount_cents.get()) {
            Ok(Some(value)) if value > 0 => value,
            Ok(_) => {
                action_message.set(Some("Enter a positive adjustment amount in cents.".into()));
                return;
            }
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };
        let direction_value = if platform_fee_cents
            .get()
            .trim()
            .eq_ignore_ascii_case("debit")
        {
            "debit".to_string()
        } else {
            "credit".to_string()
        };
        pending_action.set(Some(format!("adjust:{target_leg_id}")));
        action_message.set(None);
        let auth = auth;
        let note_value = (!note.get().trim().is_empty()).then(|| note.get());
        spawn_local(async move {
            match api::record_payment_adjustment(
                target_leg_id as u64,
                &PaymentAdjustmentRequest {
                    amount_cents: parsed_amount,
                    direction: direction_value,
                    adjustment_reference: None,
                    note: note_value,
                    idempotency_key: None,
                },
            )
            .await
            {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let generate_platform_invoices = move |_| {
        pending_action.set(Some("platform-invoices".into()));
        action_message.set(None);
        let auth = auth;
        spawn_local(async move {
            match api::generate_platform_billing_invoices().await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let mark_platform_invoice_paid = move |invoice_id: i64| {
        if invoice_id <= 0 {
            action_message.set(Some(
                "Platform invoice row is missing a valid invoice id.".into(),
            ));
            return;
        }
        pending_action.set(Some(format!("platform-paid:{invoice_id}")));
        action_message.set(None);
        let auth = auth;
        spawn_local(async move {
            match api::mark_platform_invoice_paid(invoice_id).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let approve_credit_override = move |credit_account_id: i64| {
        if credit_account_id <= 0 {
            action_message.set(Some("Credit row is missing a valid account id.".into()));
            return;
        }
        let reason = if note.get().trim().is_empty() {
            "Finance-approved temporary credit override from payments operations.".to_string()
        } else {
            note.get()
        };
        pending_action.set(Some(format!("credit-override:{credit_account_id}")));
        action_message.set(None);
        let auth = auth;
        spawn_local(async move {
            match api::approve_credit_override(&CreditOverrideRequest {
                credit_account_id,
                reason,
            })
            .await
            {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let decide_payout_review = move |review_id: i64, decision: &'static str| {
        if review_id <= 0 {
            action_message.set(Some(
                "Payout review row is missing a valid review id.".into(),
            ));
            return;
        }
        pending_action.set(Some(format!("payout-review:{decision}:{review_id}")));
        action_message.set(None);
        let auth = auth;
        let note_value = (!note.get().trim().is_empty()).then(|| note.get());
        spawn_local(async move {
            match api::decide_payout_change_review(
                review_id,
                &PayoutReviewDecisionRequest {
                    decision: decision.to_string(),
                    note: note_value,
                },
            )
            .await
            {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    let trigger_webhook = move |_| {
        let parsed_leg_id = match leg_id.get().trim() {
            "" => None,
            value => match value.parse::<i64>() {
                Ok(parsed) => Some(parsed),
                Err(_) => {
                    action_message.set(Some(
                        "Enter a valid leg id or leave it blank for account webhooks.".into(),
                    ));
                    return;
                }
            },
        };
        let parsed_amount = match parse_optional_i64(&amount_cents.get()) {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };

        pending_action.set(Some("webhook".into()));
        action_message.set(None);
        let auth = auth;
        let request = StripeWebhookRequest {
            event_id: None,
            event_type: webhook_event_type.get(),
            leg_id: parsed_leg_id,
            payment_intent_id: (!payment_intent_id.get().trim().is_empty())
                .then(|| payment_intent_id.get()),
            charge_id: None,
            transfer_id: (!transfer_id.get().trim().is_empty()).then(|| transfer_id.get()),
            transfer_group: None,
            amount_cents: parsed_amount,
            currency: Some(currency.get()),
            platform_fee_cents: None,
            stripe_account_id: (!stripe_account_id.get().trim().is_empty())
                .then(|| stripe_account_id.get()),
            payouts_enabled: match payouts_enabled.get().trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                "" => None,
                _ => {
                    action_message.set(Some(
                        "Payouts enabled must be true, false, or blank.".into(),
                    ));
                    pending_action.set(None);
                    return;
                }
            },
            kyc_status: (!kyc_status.get().trim().is_empty()).then(|| kyc_status.get()),
            note: (!note.get().trim().is_empty()).then(|| note.get()),
        };

        spawn_local(async move {
            match api::trigger_stripe_webhook(&request).await {
                Ok(result) => {
                    action_message.set(Some(result.message));
                    if result.acknowledged {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    action_message.set(Some(error));
                }
            }
            pending_action.set(None);
        });
    };

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Escrow Operations", &["access_admin_portal", "manage_payments"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>"Escrow Operations"</h2>
                            </div>
                            <div style="display:grid;gap:0.45rem;justify-items:end;">
                                <A href="/admin/stloads">"Open STLOADS operations"</A>
                                <span style=tone_style(if ws_connected.get() { "success" } else { "info" })>{move || if ws_connected.get() { "Realtime connected" } else { "Realtime reconnecting" }}</span>
                            </div>
                        </section>

                        {move || {
                            let (prefill_leg_id, prefill_action, _prefill_source, _prefill_load_id) =
                                prefill_context.get();
                            let has_prefill = prefill_leg_id
                                .as_deref()
                                .map(|value| !value.trim().is_empty())
                                .unwrap_or(false)
                                || !prefill_action.trim().is_empty();

                            let _: () = {
                                view! { <></> }
                            };
                            has_prefill.then_some(())
                        }}
                        {move || error_message.get().map(|message| view! { <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section> })}

                        <section style="display:grid;grid-template-columns:minmax(320px,420px) minmax(520px,1fr);gap:1rem;align-items:start;">
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                                <strong>"Payments"</strong>
                                <label style=FIELD_LABEL_STYLE><span>"Leg ID"</span><input prop:value=move || leg_id.get() on:input=move |ev| leg_id.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE /></label>
                                <label style=FIELD_LABEL_STYLE><span>"Amount (cents)"</span><input prop:value=move || amount_cents.get() on:input=move |ev| amount_cents.set(event_target_value(&ev)) placeholder="245000" style=FIELD_INPUT_STYLE /></label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style=FIELD_LABEL_STYLE><span>"Platform fee"</span><input prop:value=move || platform_fee_cents.get() on:input=move |ev| platform_fee_cents.set(event_target_value(&ev)) placeholder="2500" style=FIELD_INPUT_STYLE /></label>
                                    <label style=FIELD_LABEL_STYLE><span>"Currency"</span><input prop:value=move || currency.get() on:input=move |ev| currency.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE /></label>
                                </div>
                                <label style=FIELD_LABEL_STYLE><span>"Payment intent"</span><input prop:value=move || payment_intent_id.get() on:input=move |ev| payment_intent_id.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE /></label>
                                <label style=FIELD_LABEL_STYLE><span>"Transfer ID"</span><input prop:value=move || transfer_id.get() on:input=move |ev| transfer_id.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE /></label>
                                <label style=FIELD_LABEL_STYLE><span>"Note"</span><textarea prop:value=move || note.get() on:input=move |ev| note.set(event_target_value(&ev)) rows="3" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;" /></label>
                                <div style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                                    <button type="button" on:click=fund_escrow disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("fund") { "Funding..." } else { "Fund" }}</button>
                                    <button type="button" on:click=hold_escrow disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:1px solid #d97706;border-radius:0.85rem;background:#fff7ed;color:#b45309;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("hold") { "Holding..." } else { "Hold" }}</button>
                                    <button type="button" on:click=release_escrow disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:1px solid #0f766e;border-radius:0.85rem;background:#ecfdf5;color:#0f766e;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("release") { "Releasing..." } else { "Release" }}</button>
                                </div>
                                {move || action_message.get().map(|message| view! {
                                    <section style="padding:0.75rem 0.9rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">
                                        {message}
                                    </section>
                                })}
                                <hr style="border:none;border-top:1px solid #e5e7eb;width:100%;" />
                                <strong>"Stripe webhook simulator"</strong>
                                <label style=FIELD_LABEL_STYLE><span>"Event type"</span><input prop:value=move || webhook_event_type.get() on:input=move |ev| webhook_event_type.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE /></label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style=FIELD_LABEL_STYLE><span>"Stripe account"</span><input prop:value=move || stripe_account_id.get() on:input=move |ev| stripe_account_id.set(event_target_value(&ev)) style=FIELD_INPUT_STYLE /></label>
                                    <label style=FIELD_LABEL_STYLE><span>"Payouts enabled"</span><input prop:value=move || payouts_enabled.get() on:input=move |ev| payouts_enabled.set(event_target_value(&ev)) placeholder="true" style=FIELD_INPUT_STYLE /></label>
                                </div>
                                <label style=FIELD_LABEL_STYLE><span>"KYC status"</span><input prop:value=move || kyc_status.get() on:input=move |ev| kyc_status.set(event_target_value(&ev)) placeholder="verified" style=FIELD_INPUT_STYLE /></label>
                                <button type="button" on:click=trigger_webhook disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#0f172a;color:white;cursor:pointer;justify-self:start;">{move || if pending_action.get().as_deref() == Some("webhook") { "Sending..." } else { "Send webhook" }}</button>
                            </div>
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.85rem;align-self:start;overflow:auto;">
                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;">
                                    <strong>"Finance Approvals"</strong>
                                    <span style=tone_style(if approvals.get().is_empty() { "success" } else { "warning" })>
                                        {move || format!("{} pending", approvals.get().len())}
                                    </span>
                                </div>
                                <div style=TABLE_OVERFLOW_STYLE>
                                    <table style="width:100%;border-collapse:collapse;min-width:720px;">
                                        <thead style=TABLE_HEADER_STYLE>
                                            <tr>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Type"</th>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Leg"</th>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Load"</th>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Amount"</th>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Approvals"</th>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Reason"</th>
                                                <th style=TABLE_HEAD_CELL_STYLE>"Action"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {move || {
                                                let rows = approvals.get();
                                                if rows.is_empty() {
                                                    vec![view! {
                                                        <tr>
                                                            <td colspan="7" style="padding:1rem;color:#64748b;">"No pending finance approvals."</td>
                                                        </tr>
                                                    }.into_any()]
                                                } else {
                                                    rows.into_iter().map(|row| {
                                                        let action_key = format!("approve:{}:{}", row.approval_type, row.leg_id);
                                                        let approval_label = if row.approval_type == "escrow_hold" { "Hold" } else { "Release" };
                                                        let approval_type = row.approval_type.clone();
                                                        view! {
                                                            <tr style=ROW_BORDER_STYLE>
                                                                <td style=TABLE_CELL_STYLE><span style=tone_style(if approval_label == "Hold" { "warning" } else { "info" })>{approval_label}</span></td>
                                                                <td style=TABLE_CELL_STYLE>{row.leg_id}</td>
                                                                <td style=TABLE_CELL_STYLE>{row.load_id.map(|id| id.to_string()).unwrap_or_else(|| "-".into())}</td>
                                                                <td style=TABLE_CELL_STYLE>{format!("{} {:.2}", row.currency, row.amount_cents as f64 / 100.0)}</td>
                                                                <td style=TABLE_CELL_STYLE>
                                                                    <span style=tone_style("info")>{format!("{}/{}", row.approval_count, row.required_approval_count)}</span>
                                                                </td>
                                                                <td style="padding:0.75rem;max-width:260px;white-space:normal;">{row.reason.unwrap_or_else(|| "High-value release approval".into())}</td>
                                                                <td style=TABLE_CELL_STYLE>
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |_| approve_finance_request(row.leg_id, approval_type.clone())
                                                                        disabled=move || pending_action.get().is_some()
                                                                        style="padding:0.5rem 0.75rem;border:1px solid #0f766e;border-radius:0.75rem;background:#ecfdf5;color:#0f766e;cursor:pointer;"
                                                                    >
                                                                        {move || if pending_action.get().as_deref() == Some(action_key.as_str()) { "Approving..." } else { "Approve" }}
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
                            </div>
                        </section>
                        <section style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.85rem;overflow:auto;">
                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                <strong>"Invoices And Settlements"</strong>
                                <span style=tone_style("info")>{move || format!("{} rows", invoice_settlements.get().len())}</span>
                            </div>
                            <div style=TABLE_OVERFLOW_STYLE>
                                <table style="width:100%;border-collapse:collapse;min-width:980px;">
                                    <thead style=TABLE_HEADER_STYLE>
                                        <tr>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Invoice"</th>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Settlement"</th>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Load / Leg"</th>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Invoice Total"</th>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Settlement Net"</th>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Adjustments"</th>
                                            <th style=TABLE_HEAD_CELL_STYLE>"Action"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || {
                                            let rows = invoice_settlements.get();
                                            if rows.is_empty() {
                                                vec![view! {
                                                    <tr>
                                                        <td colspan="7" style="padding:1rem;color:#64748b;">"No released invoice and settlement records yet."</td>
                                                    </tr>
                                                }.into_any()]
                                            } else {
                                                rows.into_iter().map(|row| {
                                                    let action_key = format!("adjust:{}", row.leg_id);
                                                    view! {
                                                        <tr style=ROW_BORDER_STYLE>
                                                            <td style=TABLE_CELL_STYLE>
                                                                <div style="font-weight:600;">{row.invoice_number}</div>
                                                                <span style=tone_style("info")>{row.invoice_status}</span>
                                                            </td>
                                                            <td style=TABLE_CELL_STYLE>
                                                                <div style="font-weight:600;">{row.settlement_number}</div>
                                                                <span style=tone_style("success")>{row.settlement_status}</span>
                                                            </td>
                                                            <td style=TABLE_CELL_STYLE>{format!("#{} / {}", row.load_id, row.leg_id)}</td>
                                                            <td style=TABLE_CELL_STYLE>{format!("{} {:.2}", row.currency, row.invoice_total_amount_cents as f64 / 100.0)}</td>
                                                            <td style=TABLE_CELL_STYLE>{format!("{} {:.2}", row.currency, row.settlement_net_amount_cents as f64 / 100.0)}</td>
                                                            <td style=TABLE_CELL_STYLE>
                                                                {format!("Invoice {} / Settlement {}", row.invoice_adjustment_amount_cents, row.settlement_adjustment_amount_cents)}
                                                            </td>
                                                            <td style=TABLE_CELL_STYLE>
                                                                <button
                                                                    type="button"
                                                                    on:click=move |_| record_adjustment(row.leg_id)
                                                                    disabled=move || pending_action.get().is_some()
                                                                    style="padding:0.5rem 0.75rem;border:1px solid #334155;border-radius:0.75rem;background:#f8fafc;color:#0f172a;cursor:pointer;"
                                                                >
                                                                    {move || if pending_action.get().as_deref() == Some(action_key.as_str()) { "Applying..." } else { "Apply adjustment" }}
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
                        {render_platform_billing(platform_billing, pending_action, generate_platform_invoices, mark_platform_invoice_paid)}
                        {render_shipper_credit(shipper_credit, pending_action, approve_credit_override)}
                        {render_payout_reviews(payout_reviews, pending_action, decide_payout_review)}
                    </article>
                }.into_any()
            }
        }}
    }
}
