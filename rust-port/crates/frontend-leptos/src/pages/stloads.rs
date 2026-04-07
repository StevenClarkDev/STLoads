use futures_util::future::AbortHandle;
use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;
use shared::{
    RealtimeEventKind, RealtimeTopic, ResolveSyncErrorRequest, StloadsOperationsScreen,
    TmsCloseRequest, TmsHandoffPayload, TmsRequeueRequest, TmsStatusWebhookRequest,
    TmsWithdrawRequest,
};

use crate::{
    api, realtime,
    session::{self, use_auth},
};

use super::admin_guard_view;

fn tone_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
        "primary" => "background:#ede9fe;padding:0.25rem 0.6rem;border-radius:999px;color:#6d28d9;",
        "secondary" => {
            "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;"
        }
        _ => "background:#111827;padding:0.25rem 0.6rem;border-radius:999px;color:white;",
    }
}

fn default_handoff_payload_json() -> String {
    serde_json::to_string_pretty(&TmsHandoffPayload {
        tms_load_id: "TMS-RUST-1001".into(),
        tenant_id: "demo-tenant".into(),
        external_handoff_id: Some("handoff-demo-1001".into()),
        party_type: "shipper".into(),
        freight_mode: "FTL".into(),
        equipment_type: "Dry Van".into(),
        commodity_description: Some("Consumer goods".into()),
        weight: 42000.0,
        weight_unit: "lbs".into(),
        piece_count: Some(22),
        is_hazardous: Some(false),
        temperature_data: None,
        container_data: None,
        securement_data: None,
        pickup_city: "Dallas".into(),
        pickup_state: Some("TX".into()),
        pickup_zip: Some("75201".into()),
        pickup_country: "US".into(),
        pickup_address: "100 Market St, Dallas, TX".into(),
        pickup_window_start: "2026-04-07T09:00:00Z".into(),
        pickup_window_end: Some("2026-04-07T12:00:00Z".into()),
        pickup_instructions: Some("Check in at dock 4".into()),
        pickup_appointment_ref: Some("PU-1001".into()),
        dropoff_city: "Memphis".into(),
        dropoff_state: Some("TN".into()),
        dropoff_zip: Some("38103".into()),
        dropoff_country: "US".into(),
        dropoff_address: "200 Carrier Ave, Memphis, TN".into(),
        dropoff_window_start: "2026-04-08T15:00:00Z".into(),
        dropoff_window_end: Some("2026-04-08T18:00:00Z".into()),
        dropoff_instructions: Some("Call receiver before arrival".into()),
        dropoff_appointment_ref: Some("DO-1001".into()),
        board_rate: 2450.0,
        rate_currency: Some("USD".into()),
        accessorial_flags: None,
        bid_type: "Fixed".into(),
        quote_status: None,
        tender_posture: None,
        compliance_passed: Some(true),
        compliance_summary: None,
        required_documents_status: None,
        readiness: Some("ready".into()),
        pushed_by: Some("ops@stloads.com".into()),
        push_reason: Some("Admin console publish".into()),
        source_module: Some("leptos_admin".into()),
        payload_version: Some("1.0".into()),
        external_refs: None,
    })
    .unwrap_or_else(|_| "{}".into())
}

fn parse_optional_f64(value: &str) -> Result<Option<f64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        trimmed
            .parse::<f64>()
            .map(Some)
            .map_err(|_| format!("{} is not a valid decimal value.", value))
    }
}

#[component]
pub fn StloadsOperationsPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<StloadsOperationsScreen>);
    let selected_filter = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let pending_sync_error_id = RwSignal::new(None::<u64>);
    let refresh_nonce = RwSignal::new(0_u64);
    let ws_connected = RwSignal::new(false);
    let ws_handle = RwSignal::new(None::<AbortHandle>);
    let pending_action = RwSignal::new(None::<String>);

    let handoff_payload_json = RwSignal::new(default_handoff_payload_json());
    let selected_handoff_id = RwSignal::new(String::new());
    let operator_reason = RwSignal::new(String::new());
    let operator_pushed_by = RwSignal::new(String::new());
    let operator_source_module = RwSignal::new("leptos_admin".to_string());
    let webhook_tms_load_id = RwSignal::new(String::new());
    let webhook_tenant_id = RwSignal::new(String::new());
    let webhook_status = RwSignal::new("in_transit".to_string());
    let webhook_status_at = RwSignal::new(String::new());
    let webhook_rate_update = RwSignal::new(String::new());
    let webhook_detail = RwSignal::new(String::new());

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let status_filter = selected_filter.get();
        let _refresh = refresh_nonce.get();

        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        is_loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_stloads_operations_screen(status_filter.as_deref()).await {
                Ok(next_screen) => {
                    screen.set(Some(next_screen));
                    error_message.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    error_message.set(Some(error));
                }
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
            vec![RealtimeTopic::AdminTmsOperations],
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
                RealtimeEventKind::TmsOperationsUpdated => {
                    refresh_nonce.update(|value| *value += 1);
                    action_message.set(Some(format!("Realtime update: {}", event.summary)));
                }
                _ => {}
            },
        );

        ws_connected.set(handle.is_some());
        ws_handle.set(handle);
    });

    let resolve_sync_issue = move |sync_error_id: u64| {
        if pending_sync_error_id.get().is_some() {
            return;
        }

        pending_sync_error_id.set(Some(sync_error_id));
        action_message.set(None);
        let auth = auth.clone();
        spawn_local(async move {
            match api::resolve_sync_error(
                sync_error_id,
                &ResolveSyncErrorRequest {
                    resolution_note: None,
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
            pending_sync_error_id.set(None);
        });
    };

    let push_handoff = move |_| {
        let payload_text = handoff_payload_json.get();
        let payload = match serde_json::from_str::<TmsHandoffPayload>(&payload_text) {
            Ok(payload) => payload,
            Err(error) => {
                action_message.set(Some(format!("Push payload is not valid JSON: {}", error)));
                return;
            }
        };

        pending_action.set(Some("push".into()));
        action_message.set(None);
        let auth = auth.clone();
        spawn_local(async move {
            match api::push_tms_handoff(&payload).await {
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

    let queue_handoff = move |_| {
        let payload_text = handoff_payload_json.get();
        let payload = match serde_json::from_str::<TmsHandoffPayload>(&payload_text) {
            Ok(payload) => payload,
            Err(error) => {
                action_message.set(Some(format!("Queue payload is not valid JSON: {}", error)));
                return;
            }
        };

        pending_action.set(Some("queue".into()));
        action_message.set(None);
        let auth = auth.clone();
        spawn_local(async move {
            match api::queue_tms_handoff(&payload).await {
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

    let requeue_handoff = move |_| {
        let handoff_id = match selected_handoff_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                action_message.set(Some(
                    "Select or enter a valid handoff id before requeueing.".into(),
                ));
                return;
            }
        };

        pending_action.set(Some("requeue".into()));
        action_message.set(None);
        let auth = auth.clone();
        let request = TmsRequeueRequest {
            handoff_id,
            pushed_by: (!operator_pushed_by.get().trim().is_empty())
                .then(|| operator_pushed_by.get()),
            source_module: (!operator_source_module.get().trim().is_empty())
                .then(|| operator_source_module.get()),
        };
        spawn_local(async move {
            match api::requeue_tms_handoff(&request).await {
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

    let withdraw_handoff = move |_| {
        let handoff_id = match selected_handoff_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                action_message.set(Some(
                    "Select or enter a valid handoff id before withdrawing.".into(),
                ));
                return;
            }
        };

        pending_action.set(Some("withdraw".into()));
        action_message.set(None);
        let auth = auth.clone();
        let request = TmsWithdrawRequest {
            handoff_id,
            reason: (!operator_reason.get().trim().is_empty()).then(|| operator_reason.get()),
            pushed_by: (!operator_pushed_by.get().trim().is_empty())
                .then(|| operator_pushed_by.get()),
            source_module: (!operator_source_module.get().trim().is_empty())
                .then(|| operator_source_module.get()),
        };
        spawn_local(async move {
            match api::withdraw_tms_handoff(&request).await {
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

    let close_handoff = move |_| {
        let handoff_id = match selected_handoff_id.get().trim().parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                action_message.set(Some(
                    "Select or enter a valid handoff id before closing.".into(),
                ));
                return;
            }
        };

        pending_action.set(Some("close".into()));
        action_message.set(None);
        let auth = auth.clone();
        let request = TmsCloseRequest {
            handoff_id,
            reason: (!operator_reason.get().trim().is_empty()).then(|| operator_reason.get()),
            pushed_by: (!operator_pushed_by.get().trim().is_empty())
                .then(|| operator_pushed_by.get()),
            source_module: (!operator_source_module.get().trim().is_empty())
                .then(|| operator_source_module.get()),
        };
        spawn_local(async move {
            match api::close_tms_handoff(&request).await {
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

    let apply_status_webhook = move |_| {
        let parsed_rate_update = match parse_optional_f64(&webhook_rate_update.get()) {
            Ok(value) => value,
            Err(message) => {
                action_message.set(Some(message));
                return;
            }
        };
        if webhook_tms_load_id.get().trim().is_empty() || webhook_tenant_id.get().trim().is_empty()
        {
            action_message.set(Some(
                "Enter both TMS load id and tenant id before applying a status webhook.".into(),
            ));
            return;
        }

        pending_action.set(Some("webhook".into()));
        action_message.set(None);
        let auth = auth.clone();
        let request = TmsStatusWebhookRequest {
            tms_load_id: webhook_tms_load_id.get(),
            tenant_id: webhook_tenant_id.get(),
            tms_status: webhook_status.get(),
            status_at: (!webhook_status_at.get().trim().is_empty())
                .then(|| webhook_status_at.get()),
            source_module: (!operator_source_module.get().trim().is_empty())
                .then(|| operator_source_module.get()),
            pushed_by: (!operator_pushed_by.get().trim().is_empty())
                .then(|| operator_pushed_by.get()),
            detail: (!webhook_detail.get().trim().is_empty()).then(|| webhook_detail.get()),
            rate_update: parsed_rate_update,
        };
        spawn_local(async move {
            match api::apply_tms_status_webhook(&request).await {
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

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "STLOADS Operations", &["access_admin_portal", "manage_tms_operations"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>{move || screen.get().map(|value| value.title).unwrap_or_else(|| "STLOADS Operations".into())}</h2>
                                <p>"This route now loads the Rust STLOADS operations screen from the backend and gives ops direct controls for push, queue, requeue, withdraw, close, and status webhook flows."</p>
                            </div>
                            <div style="display:grid;gap:0.45rem;justify-items:end;">
                                <A href="/admin/stloads/reconciliation" attr:style="padding:0.7rem 1rem;border-radius:0.9rem;background:#111827;color:white;text-decoration:none;">"Open reconciliation"</A>
                                <span style=tone_style(if ws_connected.get() { "success" } else { "secondary" })>{move || if ws_connected.get() { "Realtime connected" } else { "Realtime reconnecting" }}</span>
                            </div>
                        </section>

                        {move || action_message.get().map(|message| view! { <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;">{message}</section> })}
                        {move || error_message.get().map(|message| view! { <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.9rem;background:#fff1f2;color:#be123c;">{message}</section> })}

                        <section style="padding:1rem;border:1px solid #fecaca;border-radius:1rem;background:#fff7f7;display:grid;gap:0.5rem;">
                            <strong>{move || screen.get().map(|value| format!("{} unresolved sync issues", value.sync_issue_summary.total)).unwrap_or_else(|| "Loading unresolved sync issues...".into())}</strong>
                            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">{move || screen.get().map(|value| view! { <><span style=tone_style("danger")>{format!("{} critical", value.sync_issue_summary.critical)}</span><span style=tone_style("warning")>{format!("{} error", value.sync_issue_summary.error)}</span><span style=tone_style("info")>{format!("{} warning", value.sync_issue_summary.warning)}</span></> })}</div>
                        </section>

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:0.85rem;">
                            {move || screen.get().map(|value| value.status_cards.into_iter().map(|card| {
                                let card_key = card.key.clone();
                                let outer_style = if card.is_active { "padding:1rem;border:2px solid #0f172a;border-radius:1rem;background:#ffffff;display:grid;gap:0.35rem;cursor:pointer;text-align:left;" } else { "padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;display:grid;gap:0.35rem;cursor:pointer;text-align:left;" };
                                view! { <button type="button" style=outer_style on:click=move |_| { let is_active = screen.get().and_then(|value| value.active_filter).as_deref() == Some(card_key.as_str()); if is_active { selected_filter.set(None); } else { selected_filter.set(Some(card_key.clone())); } action_message.set(None); }><span style=tone_style(&card.tone)>{card.label}</span><strong style="font-size:1.3rem;">{card.value}</strong>{card.note.map(|note| view! { <small>{note}</small> })}</button> }
                            }).collect_view())}
                        </section>

                        <section style="display:grid;grid-template-columns:minmax(320px,1fr) minmax(320px,420px);gap:1rem;align-items:start;">
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#ffffff;display:grid;gap:0.75rem;">
                                <strong>"Push or queue payload"</strong>
                                <textarea prop:value=move || handoff_payload_json.get() on:input=move |ev| handoff_payload_json.set(event_target_value(&ev)) rows="18" style="width:100%;padding:0.85rem;border:1px solid #d1d5db;border-radius:0.9rem;font-family:Consolas,monospace;resize:vertical;" />
                                <div style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                                    <button type="button" on:click=push_handoff disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#111827;color:white;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("push") { "Publishing..." } else { "Push handoff" }}</button>
                                    <button type="button" on:click=queue_handoff disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:1px solid #1d4ed8;border-radius:0.85rem;background:#eff6ff;color:#1d4ed8;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("queue") { "Queueing..." } else { "Queue handoff" }}</button>
                                    <button type="button" on:click=move |_| handoff_payload_json.set(default_handoff_payload_json()) style="padding:0.65rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;background:white;color:#111827;cursor:pointer;">"Reset sample"</button>
                                </div>
                            </div>

                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                                <strong>"Selected handoff controls"</strong>
                                <label style="display:grid;gap:0.35rem;"><span>"Handoff ID"</span><input prop:value=move || selected_handoff_id.get() on:input=move |ev| selected_handoff_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <label style="display:grid;gap:0.35rem;"><span>"Reason / detail"</span><input prop:value=move || operator_reason.get() on:input=move |ev| operator_reason.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"Pushed by"</span><input prop:value=move || operator_pushed_by.get() on:input=move |ev| operator_pushed_by.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Source module"</span><input prop:value=move || operator_source_module.get() on:input=move |ev| operator_source_module.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                </div>
                                <div style="display:flex;gap:0.6rem;flex-wrap:wrap;">
                                    <button type="button" on:click=requeue_handoff disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:1px solid #6d28d9;border-radius:0.85rem;background:#f5f3ff;color:#6d28d9;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("requeue") { "Requeueing..." } else { "Requeue" }}</button>
                                    <button type="button" on:click=withdraw_handoff disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:1px solid #d97706;border-radius:0.85rem;background:#fff7ed;color:#b45309;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("withdraw") { "Withdrawing..." } else { "Withdraw" }}</button>
                                    <button type="button" on:click=close_handoff disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:1px solid #0f766e;border-radius:0.85rem;background:#ecfdf5;color:#0f766e;cursor:pointer;">{move || if pending_action.get().as_deref() == Some("close") { "Closing..." } else { "Close" }}</button>
                                </div>
                                <hr style="border:none;border-top:1px solid #e5e7eb;width:100%;" />
                                <strong>"Status webhook"</strong>
                                <label style="display:grid;gap:0.35rem;"><span>"TMS load ID"</span><input prop:value=move || webhook_tms_load_id.get() on:input=move |ev| webhook_tms_load_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <label style="display:grid;gap:0.35rem;"><span>"Tenant ID"</span><input prop:value=move || webhook_tenant_id.get() on:input=move |ev| webhook_tenant_id.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"TMS status"</span><input prop:value=move || webhook_status.get() on:input=move |ev| webhook_status.set(event_target_value(&ev)) style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Rate update"</span><input prop:value=move || webhook_rate_update.get() on:input=move |ev| webhook_rate_update.set(event_target_value(&ev)) placeholder="1260.0" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                </div>
                                <label style="display:grid;gap:0.35rem;"><span>"Status at"</span><input prop:value=move || webhook_status_at.get() on:input=move |ev| webhook_status_at.set(event_target_value(&ev)) placeholder="2026-04-06T14:00:00Z" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;" /></label>
                                <label style="display:grid;gap:0.35rem;"><span>"Webhook detail"</span><textarea prop:value=move || webhook_detail.get() on:input=move |ev| webhook_detail.set(event_target_value(&ev)) rows="3" style="padding:0.75rem 0.9rem;border:1px solid #d1d5db;border-radius:0.85rem;resize:vertical;" /></label>
                                <button type="button" on:click=apply_status_webhook disabled=move || pending_action.get().is_some() style="padding:0.65rem 0.9rem;border:none;border-radius:0.85rem;background:#0f172a;color:white;cursor:pointer;justify-self:start;">{move || if pending_action.get().as_deref() == Some("webhook") { "Applying..." } else { "Apply webhook" }}</button>
                            </div>
                        </section>

                        <section style="display:grid;grid-template-columns:minmax(280px,360px) minmax(0,1fr);gap:1rem;align-items:start;">
                            <div style="border:1px solid #e5e7eb;border-radius:1rem;padding:1rem;background:#fcfcfb;display:grid;gap:0.75rem;">
                                <strong>"Recent sync issues"</strong>
                                {move || if is_loading.get() && screen.get().is_none() { view! { <p style="margin:0;">"Loading STLOADS issues from the Rust backend..."</p> }.into_any() } else { screen.get().map(|value| value.recent_sync_issues.into_iter().map(|issue| {
                                    let severity = issue.severity;
                                    let severity_style = tone_style(&severity);
                                    let sync_error_id = issue.id;
                                    let is_resolving = Signal::derive(move || pending_sync_error_id.get() == Some(sync_error_id));
                                    view! { <div style="padding:0.85rem;border:1px solid #e5e7eb;border-radius:0.9rem;display:grid;gap:0.5rem;"><div style="display:flex;justify-content:space-between;gap:0.5rem;align-items:center;flex-wrap:wrap;"><span style=severity_style>{severity}</span><small>{issue.created_at_label}</small></div><code>{issue.error_class}</code><span>{issue.title}</span>{issue.handoff_ref.map(|handoff| view! { <small>{handoff}</small> })}<div style="display:flex;justify-content:flex-end;"><button type="button" style="padding:0.5rem 0.8rem;border-radius:0.75rem;border:1px solid #0f766e;background:#ecfdf5;color:#0f766e;cursor:pointer;" disabled=move || is_resolving.get() on:click=move |_| resolve_sync_issue(sync_error_id)>{move || if is_resolving.get() { "Resolving..." } else { "Resolve" }}</button></div></div> }
                                }).collect_view().into_any()).unwrap_or_else(|| view! { <p style="margin:0;">"No STLOADS issue data is available yet."</p> }.into_any()) }}
                            </div>

                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:1rem;">
                                <div style="padding:1rem;border-bottom:1px solid #e5e7eb;display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;align-items:center;">
                                    <div><strong>"Handoff records"</strong><p style="margin:0.35rem 0 0;">{move || screen.get().map(|value| format!("Showing {} handoffs", value.active_filter.unwrap_or_else(|| "all".into()))).unwrap_or_else(|| "Loading handoff filter...".into())}</p></div>
                                    <small>{move || screen.get().map(|value| format!("{} total tracked rows", value.pagination.total)).unwrap_or_else(|| "Loading rows...".into())}</small>
                                </div>
                                <table style="width:100%;border-collapse:collapse;min-width:980px;">
                                    <thead style="background:#f8fafc;"><tr><th style="text-align:left;padding:0.9rem;">"Handoff"</th><th style="text-align:left;padding:0.9rem;">"TMS Load"</th><th style="text-align:left;padding:0.9rem;">"Route"</th><th style="text-align:left;padding:0.9rem;">"Mode"</th><th style="text-align:left;padding:0.9rem;">"Equipment"</th><th style="text-align:left;padding:0.9rem;">"Rate"</th><th style="text-align:left;padding:0.9rem;">"Status"</th><th style="text-align:left;padding:0.9rem;">"Load"</th><th style="text-align:left;padding:0.9rem;">"Retries"</th><th style="text-align:left;padding:0.9rem;">"Action"</th></tr></thead>
                                    <tbody>
                                        {move || if is_loading.get() && screen.get().is_none() { view! { <tr><td colspan="10" style="padding:1rem;">"Loading STLOADS handoffs from the Rust backend..."</td></tr> }.into_any() } else { screen.get().map(|value| value.handoffs.into_iter().map(|handoff| {
                                            let handoff_id = handoff.handoff_id;
                                            let tms_load_id = handoff.tms_load_id.clone();
                                            let status_key = handoff.status_key.clone();
                                            view! {
                                                <tr style="border-top:1px solid #f1f5f9;">
                                                    <td style="padding:0.9rem;">{handoff.handoff_ref}</td>
                                                    <td style="padding:0.9rem;">{handoff.tms_load_id}</td>
                                                    <td style="padding:0.9rem;">{handoff.route_label}</td>
                                                    <td style="padding:0.9rem;">{handoff.freight_mode}</td>
                                                    <td style="padding:0.9rem;">{handoff.equipment_type}</td>
                                                    <td style="padding:0.9rem;">{handoff.rate_label}</td>
                                                    <td style="padding:0.9rem;"><span style=tone_style(&handoff.status_tone)>{handoff.status_label}</span><div><small>{status_key}</small></div></td>
                                                    <td style="padding:0.9rem;">{handoff.load_number.unwrap_or_else(|| "Pending local load".into())}</td>
                                                    <td style="padding:0.9rem;">{handoff.retry_count}</td>
                                                    <td style="padding:0.9rem;">
                                                        <button type="button" style="padding:0.5rem 0.8rem;border-radius:0.75rem;border:1px solid #111827;background:#111827;color:white;cursor:pointer;" on:click=move |_| { selected_handoff_id.set(handoff_id.to_string()); webhook_tms_load_id.set(tms_load_id.clone()); action_message.set(Some(format!("Selected handoff #{} for the operator console.", handoff_id))); }>
                                                            "Use"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view().into_any()).unwrap_or_else(|| view! { <tr><td colspan="10" style="padding:1rem;">"No STLOADS handoff data is available yet."</td></tr> }.into_any()) }}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <section style="display:grid;gap:0.35rem;">{move || screen.get().map(|value| value.notes.into_iter().map(|note| view! { <p style="margin:0;">{note}</p> }).collect_view())}</section>
                    </article>
                }.into_any()
            }
        }}
    }
}
