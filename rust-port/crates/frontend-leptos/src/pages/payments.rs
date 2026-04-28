use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_query_map};
use shared::{
    EscrowFundRequest, EscrowHoldRequest, EscrowReleaseRequest, RealtimeEventKind, RealtimeTopic,
    StripeWebhookRequest,
};

use crate::{
    api::{self, EscrowStatusDescriptorLite, PaymentsOverview, StripeWebhookEventDescriptorLite},
    realtime,
    session::{self, use_auth},
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

fn parse_required_u64(value: &str, field: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Enter a valid {} before running this action.", field))
}

fn parse_optional_i64(value: &str) -> Result<Option<i64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        trimmed
            .parse::<i64>()
            .map(Some)
            .map_err(|_| format!("{} is not a valid whole number.", value))
    }
}

#[component]
pub fn EscrowOperationsPage() -> impl IntoView {
    let auth = use_auth();
    let query = use_query_map();
    let overview = RwSignal::new(None::<PaymentsOverview>);
    let statuses = RwSignal::new(Vec::<EscrowStatusDescriptorLite>::new());
    let webhooks = RwSignal::new(Vec::<StripeWebhookEventDescriptorLite>::new());
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
        let auth = auth.clone();

        spawn_local(async move {
            let overview_result = api::fetch_payments_overview().await;
            let statuses_result = api::fetch_escrow_status_catalog().await;
            let webhooks_result = api::fetch_stripe_webhook_event_catalog().await;

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
        let auth = auth.clone();
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
        let auth = auth.clone();
        let request = EscrowFundRequest {
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
        let auth = auth.clone();
        let request = EscrowHoldRequest {
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
        let auth = auth.clone();
        let request = EscrowReleaseRequest {
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
        let auth = auth.clone();
        let request = StripeWebhookRequest {
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

                            has_prefill.then(|| {
                                view! { <></> }
                            })
                        }}
                        {move || error_message.get().map(|message| view! { <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section> })}

                        <section style="display:grid;grid-template-columns:minmax(320px,420px);gap:1rem;align-items:start;">
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                                <strong>"Payments"</strong>
                                <label style="display:grid;gap:0.35rem;"><span>"Leg ID"</span><input prop:value=move || leg_id.get() on:input=move |ev| leg_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <label style="display:grid;gap:0.35rem;"><span>"Amount (cents)"</span><input prop:value=move || amount_cents.get() on:input=move |ev| amount_cents.set(event_target_value(&ev)) placeholder="245000" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"Platform fee"</span><input prop:value=move || platform_fee_cents.get() on:input=move |ev| platform_fee_cents.set(event_target_value(&ev)) placeholder="2500" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Currency"</span><input prop:value=move || currency.get() on:input=move |ev| currency.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                </div>
                                <label style="display:grid;gap:0.35rem;"><span>"Payment intent"</span><input prop:value=move || payment_intent_id.get() on:input=move |ev| payment_intent_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <label style="display:grid;gap:0.35rem;"><span>"Transfer ID"</span><input prop:value=move || transfer_id.get() on:input=move |ev| transfer_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <label style="display:grid;gap:0.35rem;"><span>"Note"</span><textarea prop:value=move || note.get() on:input=move |ev| note.set(event_target_value(&ev)) rows="3" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;" /></label>
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
                                <label style="display:grid;gap:0.35rem;"><span>"Event type"</span><input prop:value=move || webhook_event_type.get() on:input=move |ev| webhook_event_type.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"Stripe account"</span><input prop:value=move || stripe_account_id.get() on:input=move |ev| stripe_account_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Payouts enabled"</span><input prop:value=move || payouts_enabled.get() on:input=move |ev| payouts_enabled.set(event_target_value(&ev)) placeholder="true" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                </div>
                                <label style="display:grid;gap:0.35rem;"><span>"KYC status"</span><input prop:value=move || kyc_status.get() on:input=move |ev| kyc_status.set(event_target_value(&ev)) placeholder="verified" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <button type="button" on:click=trigger_webhook disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#0f172a;color:white;cursor:pointer;justify-self:start;">{move || if pending_action.get().as_deref() == Some("webhook") { "Sending..." } else { "Send webhook" }}</button>
                            </div>
                        </section>
                    </article>
                }.into_any()
            }
        }}
    }
}
