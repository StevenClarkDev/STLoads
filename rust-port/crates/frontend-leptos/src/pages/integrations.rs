use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api::{
        self, ApiLifecycleScreen, CreatePartnerApiKeyRequest, EdiIntegrationScreen,
        IntegrationPortalScreen, QueueSandboxResetRequest, SandboxGovernanceScreen,
        UpsertEdiPartnerProfileRequest, UpsertWebhookEndpointRequest, ValidateEdiMessageRequest,
    },
    session::{self, use_auth},
};

use super::{
    admin_guard_view,
    integrations_helpers::{render_api_lifecycle, render_sandbox_governance},
    shared::{
        FIELD_LABEL_STYLE, ROW_BORDER_STYLE, TABLE_CELL_STYLE, split_comma_values, tone_style,
    },
};

#[component]
pub fn IntegrationPortalPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<IntegrationPortalScreen>);
    let edi_screen = RwSignal::new(None::<EdiIntegrationScreen>);
    let sandbox_screen = RwSignal::new(None::<SandboxGovernanceScreen>);
    let lifecycle_screen = RwSignal::new(None::<ApiLifecycleScreen>);
    let is_loading = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let action_message = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let pending_action = RwSignal::new(None::<String>);
    let created_api_key = RwSignal::new(None::<String>);

    let api_client_name = RwSignal::new(String::new());
    let api_scopes = RwSignal::new("loads:write,webhooks:read".to_string());
    let api_rate_limit = RwSignal::new("60".to_string());
    let api_requires_signature = RwSignal::new(true);
    let api_expires_at = RwSignal::new(String::new());

    let webhook_name = RwSignal::new(String::new());
    let webhook_url = RwSignal::new(String::new());
    let webhook_events = RwSignal::new("load.booked,load.cancelled,payment.released".to_string());
    let webhook_status = RwSignal::new("active".to_string());

    let edi_partner_name = RwSignal::new(String::new());
    let edi_isa_id = RwSignal::new(String::new());
    let edi_gs_id = RwSignal::new(String::new());
    let edi_transactions = RwSignal::new("204,990,214,210,997".to_string());
    let edi_transport_type = RwSignal::new("sftp".to_string());
    let edi_validation_mode = RwSignal::new("strict".to_string());

    let edi_transaction_code = RwSignal::new("204".to_string());
    let edi_direction = RwSignal::new("inbound".to_string());
    let edi_control_number = RwSignal::new(String::new());
    let edi_business_key = RwSignal::new(String::new());
    let edi_payload_excerpt = RwSignal::new(String::new());
    let sandbox_reset_reason = RwSignal::new(String::new());

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_tms_operations")
            || session::has_permission(&auth, "manage_dispatch_desk")
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
            match api::fetch_integration_portal().await {
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
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let _refresh = refresh_nonce.get();
        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        spawn_local(async move {
            match api::fetch_sandbox_governance_screen().await {
                Ok(next_screen) => sandbox_screen.set(Some(next_screen)),
                Err(error) => error_message.set(Some(error)),
            }
            match api::fetch_api_lifecycle_screen().await {
                Ok(next_screen) => lifecycle_screen.set(Some(next_screen)),
                Err(error) => error_message.set(Some(error)),
            }
        });
    });

    Effect::new(move |_| {
        let ready = auth.session_ready.get();
        let current_session = auth.session.get();
        let _refresh = refresh_nonce.get();
        if !ready || !current_session.authenticated || !can_view.get() {
            return;
        }

        spawn_local(async move {
            match api::fetch_edi_integration_screen().await {
                Ok(next_screen) => edi_screen.set(Some(next_screen)),
                Err(error) => error_message.set(Some(error)),
            }
        });
    });

    let create_key = move |ev: SubmitEvent| {
        ev.prevent_default();
        if pending_action.get().is_some() {
            return;
        }

        let rate_limit = match api_rate_limit.get().trim().parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                action_message.set(Some("Enter a valid per-minute API rate limit.".into()));
                return;
            }
        };
        let payload = CreatePartnerApiKeyRequest {
            client_name: api_client_name.get(),
            scopes: split_comma_values(api_scopes.get()),
            rate_limit_per_minute: Some(rate_limit),
            require_request_signature: Some(api_requires_signature.get()),
            expires_at: Some(api_expires_at.get()).filter(|value| !value.trim().is_empty()),
        };

        pending_action.set(Some("api-key".into()));
        action_message.set(None);
        created_api_key.set(None);
        spawn_local(async move {
            match api::create_partner_api_key(&payload).await {
                Ok(response) => {
                    if response.success {
                        created_api_key.set(response.api_key);
                        api_client_name.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                    action_message.set(Some(response.message));
                }
                Err(error) => action_message.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let save_webhook = move |ev: SubmitEvent| {
        ev.prevent_default();
        if pending_action.get().is_some() {
            return;
        }

        let payload = UpsertWebhookEndpointRequest {
            endpoint_name: webhook_name.get(),
            target_url: webhook_url.get(),
            event_types: split_comma_values(webhook_events.get()),
            status: Some(webhook_status.get()),
        };

        pending_action.set(Some("webhook".into()));
        action_message.set(None);
        spawn_local(async move {
            match api::upsert_webhook_endpoint(&payload).await {
                Ok(response) => {
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                    action_message.set(Some(response.message));
                }
                Err(error) => action_message.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let save_edi_partner = move |ev: SubmitEvent| {
        ev.prevent_default();
        if pending_action.get().is_some() {
            return;
        }
        let payload = UpsertEdiPartnerProfileRequest {
            partner_name: edi_partner_name.get(),
            isa_id: Some(edi_isa_id.get()).filter(|value| !value.trim().is_empty()),
            gs_id: Some(edi_gs_id.get()).filter(|value| !value.trim().is_empty()),
            transport_type: Some(edi_transport_type.get()),
            status: Some("active".into()),
            supported_transactions: split_comma_values(edi_transactions.get()),
            validation_mode: Some(edi_validation_mode.get()),
        };

        pending_action.set(Some("edi-partner".into()));
        action_message.set(None);
        spawn_local(async move {
            match api::upsert_edi_partner_profile(&payload).await {
                Ok(response) => {
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                    action_message.set(Some(response.message));
                }
                Err(error) => action_message.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let validate_edi = move |ev: SubmitEvent| {
        ev.prevent_default();
        if pending_action.get().is_some() {
            return;
        }
        let payload = ValidateEdiMessageRequest {
            transaction_code: edi_transaction_code.get(),
            direction: edi_direction.get(),
            control_number: Some(edi_control_number.get()).filter(|value| !value.trim().is_empty()),
            business_key: Some(edi_business_key.get()).filter(|value| !value.trim().is_empty()),
            payload_excerpt: Some(edi_payload_excerpt.get())
                .filter(|value| !value.trim().is_empty()),
        };

        pending_action.set(Some("edi-validate".into()));
        action_message.set(None);
        spawn_local(async move {
            match api::validate_edi_message(&payload).await {
                Ok(response) => {
                    let missing = if response.missing_fields.is_empty() {
                        String::new()
                    } else {
                        format!(" Missing: {}.", response.missing_fields.join(", "))
                    };
                    action_message.set(Some(format!("{}{}", response.message, missing)));
                    refresh_nonce.update(|value| *value += 1);
                }
                Err(error) => action_message.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let replay_edi = move |message_id: u64| {
        if pending_action.get().is_some() {
            return;
        }
        pending_action.set(Some(format!("edi-replay-{message_id}")));
        action_message.set(None);
        spawn_local(async move {
            match api::replay_edi_message(message_id).await {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let queue_reset = move |sandbox_environment_id: u64| {
        if pending_action.get().is_some() {
            return;
        }
        let payload = QueueSandboxResetRequest {
            sandbox_environment_id,
            reset_reason: Some(sandbox_reset_reason.get()).filter(|value| !value.trim().is_empty()),
        };
        pending_action.set(Some(format!("sandbox-reset-{sandbox_environment_id}")));
        action_message.set(None);
        spawn_local(async move {
            match api::queue_sandbox_reset(&payload).await {
                Ok(response) => {
                    action_message.set(Some(response.message));
                    if response.success {
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => action_message.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(
                &auth,
                "Customer Integration Portal",
                &[
                    "access_admin_portal",
                    "manage_tms_operations",
                    "manage_dispatch_desk",
                ],
            ) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>"Customer Integration Portal"</h2>
                                <p style="margin:0;color:#64748b;">
                                    {move || screen
                                        .get()
                                        .map(|value| format!("Organization #{} | API {}", value.organization_id, value.api_version))
                                        .unwrap_or_else(|| "Self-serve API keys, webhook endpoints, sandbox access, and delivery logs.".into())}
                                </p>
                            </div>
                            <button
                                type="button"
                                class="shell-action secondary"
                                on:click=move |_| refresh_nonce.update(|value| *value += 1)
                                disabled=move || is_loading.get()
                            >
                                <i class="fas fa-sync-alt"></i>
                                <span>{move || if is_loading.get() { "Refreshing" } else { "Refresh" }}</span>
                            </button>
                        </section>

                        {move || error_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #fecaca;border-radius:0.5rem;background:#fff1f2;color:#be123c;">
                                {message}
                            </section>
                        })}

                        {move || action_message.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #bfdbfe;border-radius:0.5rem;background:#eff6ff;color:#1d4ed8;">
                                {message}
                            </section>
                        })}

                        {move || created_api_key.get().map(|key| view! {
                            <section style="display:grid;gap:0.5rem;padding:1rem;border:1px solid #bbf7d0;border-radius:0.5rem;background:#f0fdf4;">
                                <strong>"One-time API key"</strong>
                                <code style="word-break:break-all;background:white;padding:0.75rem;border-radius:0.4rem;border:1px solid #dcfce7;">{key}</code>
                            </section>
                        })}

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;align-items:start;">
                            <form on:submit=create_key style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;">
                                <h3 style="margin:0;">"Create API key"</h3>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Client name"</span>
                                    <input
                                        type="text"
                                        prop:value=move || api_client_name.get()
                                        on:input=move |ev| api_client_name.set(event_target_value(&ev))
                                        placeholder="Acme TMS production"
                                    />
                                </label>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Scopes"</span>
                                    <input
                                        type="text"
                                        prop:value=move || api_scopes.get()
                                        on:input=move |ev| api_scopes.set(event_target_value(&ev))
                                    />
                                </label>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Rate limit"</span>
                                        <input
                                            type="number"
                                            min="1"
                                            prop:value=move || api_rate_limit.get()
                                            on:input=move |ev| api_rate_limit.set(event_target_value(&ev))
                                        />
                                    </label>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Expires at"</span>
                                        <input
                                            type="datetime-local"
                                            prop:value=move || api_expires_at.get()
                                            on:input=move |ev| api_expires_at.set(event_target_value(&ev))
                                        />
                                    </label>
                                </div>
                                <label style="display:flex;gap:0.5rem;align-items:center;">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || api_requires_signature.get()
                                        on:change=move |ev| api_requires_signature.set(event_target_checked(&ev))
                                    />
                                    <span>"Require signed requests"</span>
                                </label>
                                <button
                                    type="submit"
                                    class="shell-action"
                                    disabled=move || pending_action.get().is_some()
                                >
                                    <i class="fas fa-key"></i>
                                    <span>"Create key"</span>
                                </button>
                            </form>

                            <form on:submit=save_webhook style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;">
                                <h3 style="margin:0;">"Webhook endpoint"</h3>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Endpoint name"</span>
                                    <input
                                        type="text"
                                        prop:value=move || webhook_name.get()
                                        on:input=move |ev| webhook_name.set(event_target_value(&ev))
                                        placeholder="Production dispatch events"
                                    />
                                </label>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"HTTPS URL"</span>
                                    <input
                                        type="url"
                                        prop:value=move || webhook_url.get()
                                        on:input=move |ev| webhook_url.set(event_target_value(&ev))
                                        placeholder="https://example.com/stloads/webhooks"
                                    />
                                </label>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Events"</span>
                                    <input
                                        type="text"
                                        prop:value=move || webhook_events.get()
                                        on:input=move |ev| webhook_events.set(event_target_value(&ev))
                                    />
                                </label>
                                <label style=FIELD_LABEL_STYLE>
                                    <span>"Status"</span>
                                    <select
                                        prop:value=move || webhook_status.get()
                                        on:change=move |ev| webhook_status.set(event_target_value(&ev))
                                    >
                                        <option value="active">"Active"</option>
                                        <option value="paused">"Paused"</option>
                                        <option value="disabled">"Disabled"</option>
                                    </select>
                                </label>
                                <button
                                    type="submit"
                                    class="shell-action"
                                    disabled=move || pending_action.get().is_some()
                                >
                                    <i class="fas fa-plug"></i>
                                    <span>"Save endpoint"</span>
                                </button>
                            </form>
                        </section>

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                            {move || screen.get().map(|value| view! {
                                <>
                                    <div style="border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;display:grid;gap:0.5rem;">
                                        <strong>"Sandbox"</strong>
                                        <code>{value.sandbox.base_url}</code>
                                        <small>{value.sandbox.production_safety}</small>
                                        <small>{value.sandbox.reset_policy}</small>
                                    </div>
                                    {value.docs.into_iter().map(|doc| view! {
                                        <a href=doc.href target="_blank" rel="noreferrer" style="border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;display:grid;gap:0.5rem;color:inherit;text-decoration:none;">
                                            <strong>{doc.label}</strong>
                                            <small>{doc.description}</small>
                                        </a>
                                    }).collect_view()}
                                </>
                            })}
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <h3 style="margin:0;">"API keys"</h3>
                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:760px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style=TABLE_CELL_STYLE>"Client"</th>
                                            <th style=TABLE_CELL_STYLE>"Prefix"</th>
                                            <th style=TABLE_CELL_STYLE>"Scopes"</th>
                                            <th style=TABLE_CELL_STYLE>"Status"</th>
                                            <th style=TABLE_CELL_STYLE>"Rate"</th>
                                            <th style=TABLE_CELL_STYLE>"Signature"</th>
                                            <th style=TABLE_CELL_STYLE>"Last used"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || screen.get().map(|value| {
                                            value.api_keys.into_iter().map(|row| {
                                                let status_style = tone_style(&row.status);
                                                view! {
                                                    <tr style=ROW_BORDER_STYLE>
                                                        <td style=TABLE_CELL_STYLE>{row.client_name}</td>
                                                        <td style=TABLE_CELL_STYLE><code>{row.key_prefix}</code></td>
                                                        <td style=TABLE_CELL_STYLE>{row.scopes.join(", ")}</td>
                                                        <td style=TABLE_CELL_STYLE><span style=status_style>{row.status}</span></td>
                                                        <td style=TABLE_CELL_STYLE>{format!("{}/min", row.rate_limit_per_minute)}</td>
                                                        <td style=TABLE_CELL_STYLE>{if row.require_request_signature { "Required" } else { "Optional" }}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.last_used_at.unwrap_or_else(|| "Never".into())}</td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <h3 style="margin:0;">"Webhook endpoints"</h3>
                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:720px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style=TABLE_CELL_STYLE>"Name"</th>
                                            <th style=TABLE_CELL_STYLE>"URL"</th>
                                            <th style=TABLE_CELL_STYLE>"Events"</th>
                                            <th style=TABLE_CELL_STYLE>"Status"</th>
                                            <th style=TABLE_CELL_STYLE>"Created"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || screen.get().map(|value| {
                                            value.webhook_endpoints.into_iter().map(|row| {
                                                let status_style = tone_style(&row.status);
                                                view! {
                                                    <tr style=ROW_BORDER_STYLE>
                                                        <td style=TABLE_CELL_STYLE>{row.endpoint_name}</td>
                                                        <td style="padding:0.75rem;word-break:break-all;">{row.target_url}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.event_types.join(", ")}</td>
                                                        <td style=TABLE_CELL_STYLE><span style=status_style>{row.status}</span></td>
                                                        <td style=TABLE_CELL_STYLE>{row.created_at}</td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <h3 style="margin:0;">"Recent delivery logs"</h3>
                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:820px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style=TABLE_CELL_STYLE>"Event"</th>
                                            <th style=TABLE_CELL_STYLE>"Event ID"</th>
                                            <th style=TABLE_CELL_STYLE>"Status"</th>
                                            <th style=TABLE_CELL_STYLE>"Attempts"</th>
                                            <th style=TABLE_CELL_STYLE>"HTTP"</th>
                                            <th style=TABLE_CELL_STYLE>"Created"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || screen.get().map(|value| {
                                            value.recent_deliveries.into_iter().map(|row| {
                                                let status_style = tone_style(&row.delivery_status);
                                                view! {
                                                    <tr style=ROW_BORDER_STYLE>
                                                        <td style=TABLE_CELL_STYLE>{row.event_type}</td>
                                                        <td style=TABLE_CELL_STYLE><code>{row.event_id}</code></td>
                                                        <td style=TABLE_CELL_STYLE><span style=status_style>{row.delivery_status}</span></td>
                                                        <td style=TABLE_CELL_STYLE>{row.attempt_count}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.response_status_code.map(|value| value.to_string()).unwrap_or_else(|| "-".into())}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.created_at}</td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <div>
                                <h3 style="margin:0;">"EDI integration track"</h3>
                                <p style="margin:0.25rem 0 0;color:#64748b;">
                                    {move || edi_screen
                                        .get()
                                        .map(|value| value.replay_policy)
                                        .unwrap_or_else(|| "EDI mapping, validation, acknowledgements, retry, replay, and partner rules.".into())}
                                </p>
                            </div>

                            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;align-items:start;">
                                <form on:submit=save_edi_partner style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;">
                                    <h4 style="margin:0;">"EDI partner profile"</h4>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Partner name"</span>
                                        <input type="text" prop:value=move || edi_partner_name.get() on:input=move |ev| edi_partner_name.set(event_target_value(&ev)) placeholder="Acme EDI production" />
                                    </label>
                                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"ISA ID"</span>
                                            <input type="text" prop:value=move || edi_isa_id.get() on:input=move |ev| edi_isa_id.set(event_target_value(&ev)) />
                                        </label>
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"GS ID"</span>
                                            <input type="text" prop:value=move || edi_gs_id.get() on:input=move |ev| edi_gs_id.set(event_target_value(&ev)) />
                                        </label>
                                    </div>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Transactions"</span>
                                        <input type="text" prop:value=move || edi_transactions.get() on:input=move |ev| edi_transactions.set(event_target_value(&ev)) />
                                    </label>
                                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Transport"</span>
                                            <select prop:value=move || edi_transport_type.get() on:change=move |ev| edi_transport_type.set(event_target_value(&ev))>
                                                <option value="sftp">"SFTP"</option>
                                                <option value="as2">"AS2"</option>
                                                <option value="api">"API"</option>
                                                <option value="manual">"Manual"</option>
                                            </select>
                                        </label>
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Validation"</span>
                                            <select prop:value=move || edi_validation_mode.get() on:change=move |ev| edi_validation_mode.set(event_target_value(&ev))>
                                                <option value="strict">"Strict"</option>
                                                <option value="warn">"Warn"</option>
                                                <option value="sandbox">"Sandbox"</option>
                                            </select>
                                        </label>
                                    </div>
                                    <button type="submit" class="shell-action" disabled=move || pending_action.get().is_some()>
                                        <i class="fas fa-save"></i>
                                        <span>"Save partner"</span>
                                    </button>
                                </form>

                                <form on:submit=validate_edi style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:#ffffff;">
                                    <h4 style="margin:0;">"Validate EDI message"</h4>
                                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Transaction"</span>
                                            <select prop:value=move || edi_transaction_code.get() on:change=move |ev| edi_transaction_code.set(event_target_value(&ev))>
                                                <option value="204">"204 load tender"</option>
                                                <option value="990">"990 tender response"</option>
                                                <option value="214">"214 shipment status"</option>
                                                <option value="210">"210 invoice"</option>
                                                <option value="997">"997 acknowledgement"</option>
                                            </select>
                                        </label>
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Direction"</span>
                                            <select prop:value=move || edi_direction.get() on:change=move |ev| edi_direction.set(event_target_value(&ev))>
                                                <option value="inbound">"Inbound"</option>
                                                <option value="outbound">"Outbound"</option>
                                            </select>
                                        </label>
                                    </div>
                                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Control number"</span>
                                            <input type="text" prop:value=move || edi_control_number.get() on:input=move |ev| edi_control_number.set(event_target_value(&ev)) />
                                        </label>
                                        <label style=FIELD_LABEL_STYLE>
                                            <span>"Business key"</span>
                                            <input type="text" prop:value=move || edi_business_key.get() on:input=move |ev| edi_business_key.set(event_target_value(&ev)) />
                                        </label>
                                    </div>
                                    <label style=FIELD_LABEL_STYLE>
                                        <span>"Payload excerpt"</span>
                                        <textarea rows="5" prop:value=move || edi_payload_excerpt.get() on:input=move |ev| edi_payload_excerpt.set(event_target_value(&ev))></textarea>
                                    </label>
                                    <button type="submit" class="shell-action" disabled=move || pending_action.get().is_some()>
                                        <i class="fas fa-check-circle"></i>
                                        <span>"Validate message"</span>
                                    </button>
                                </form>
                            </section>

                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:820px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style=TABLE_CELL_STYLE>"Transaction"</th>
                                            <th style=TABLE_CELL_STYLE>"Direction"</th>
                                            <th style=TABLE_CELL_STYLE>"STLoads model"</th>
                                            <th style=TABLE_CELL_STYLE>"Required fields"</th>
                                            <th style=TABLE_CELL_STYLE>"Status"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || edi_screen.get().map(|value| {
                                            value.supported_transactions.into_iter().map(|row| {
                                                let status_style = tone_style(&row.status);
                                                view! {
                                                    <tr style=ROW_BORDER_STYLE>
                                                        <td style=TABLE_CELL_STYLE><strong>{row.transaction_code}</strong></td>
                                                        <td style=TABLE_CELL_STYLE>{row.direction}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.stloads_model}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.required_fields.join(", ")}</td>
                                                        <td style=TABLE_CELL_STYLE><span style=status_style>{row.status}</span></td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        })}
                                    </tbody>
                                </table>
                            </div>

                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:880px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style=TABLE_CELL_STYLE>"EDI message"</th>
                                            <th style=TABLE_CELL_STYLE>"Business key"</th>
                                            <th style=TABLE_CELL_STYLE>"Status"</th>
                                            <th style=TABLE_CELL_STYLE>"Ack"</th>
                                            <th style=TABLE_CELL_STYLE>"Retries"</th>
                                            <th style=TABLE_CELL_STYLE>"Action"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || edi_screen.get().map(|value| {
                                            value.message_logs.into_iter().map(|row| {
                                                let status_style = tone_style(&row.message_status);
                                                let message_id = row.id;
                                                view! {
                                                    <tr style=ROW_BORDER_STYLE>
                                                        <td style=TABLE_CELL_STYLE>
                                                            <strong>{format!("{} {}", row.transaction_code, row.direction)}</strong>
                                                            <div><small>{row.control_number.unwrap_or_else(|| "No control number".into())}</small></div>
                                                        </td>
                                                        <td style=TABLE_CELL_STYLE>{row.business_key.unwrap_or_else(|| "-".into())}</td>
                                                        <td style=TABLE_CELL_STYLE><span style=status_style>{row.message_status}</span></td>
                                                        <td style=TABLE_CELL_STYLE>{row.ack_status}</td>
                                                        <td style=TABLE_CELL_STYLE>{row.retry_count}</td>
                                                        <td style=TABLE_CELL_STYLE>
                                                            <button type="button" class="shell-action secondary" on:click=move |_| replay_edi(message_id) disabled=move || pending_action.get().is_some()>
                                                                <i class="fas fa-redo"></i>
                                                                <span>"Replay"</span>
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        {render_sandbox_governance(sandbox_screen, sandbox_reset_reason, pending_action, queue_reset)}

                        {render_api_lifecycle(lifecycle_screen)}
                    </article>
                }
                .into_any()
            }
        }}
    }
}
