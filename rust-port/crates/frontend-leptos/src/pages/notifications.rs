use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use super::shared::tone_style;
use crate::{
    api::{
        self, CommunicationGovernanceScreen, MarkNotificationReadRequest,
        MessageTemplateTestSendRequest, NotificationCenterScreen,
        UpsertNotificationPreferenceRequest,
    },
    session::{self, use_auth},
};

#[component]
pub fn NotificationCenterPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<NotificationCenterScreen>);
    let governance = RwSignal::new(None::<CommunicationGovernanceScreen>);
    let is_loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);
    let pending_action = RwSignal::new(None::<String>);

    let event_key = RwSignal::new("*".to_string());
    let email_enabled = RwSignal::new(true);
    let in_app_enabled = RwSignal::new(true);
    let sms_enabled = RwSignal::new(false);
    let push_enabled = RwSignal::new(false);
    let quiet_start = RwSignal::new(String::new());
    let quiet_end = RwSignal::new(String::new());
    let timezone = RwSignal::new("UTC".to_string());
    let escalation_minutes = RwSignal::new(String::new());
    let test_template_key = RwSignal::new("otp.login".to_string());
    let test_channel = RwSignal::new("email".to_string());
    let test_recipient = RwSignal::new(String::new());

    let can_view = Signal::derive(move || auth.session.get().authenticated);

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if !auth.session_ready.get() || !can_view.get() {
            return;
        }

        is_loading.set(true);
        let auth = auth;
        spawn_local(async move {
            let center_result = api::fetch_notification_center().await;
            let governance_result = api::fetch_communication_governance().await;
            match (center_result, governance_result) {
                (Ok(next), Ok(next_governance)) => {
                    screen.set(Some(next));
                    governance.set(Some(next_governance));
                    feedback.set(None);
                }
                (Err(error), _) | (_, Err(error)) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            is_loading.set(false);
        });
    });

    let mark_read = move |notification_id: Option<u64>, mark_all: bool| {
        if pending_action.get().is_some() {
            return;
        }
        pending_action.set(Some("mark-read".into()));
        spawn_local(async move {
            let payload = MarkNotificationReadRequest {
                notification_id,
                mark_all: Some(mark_all),
            };
            match api::mark_notification_read(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    refresh_nonce.update(|value| *value += 1);
                }
                Err(error) => feedback.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let record_test_send = move |ev: SubmitEvent| {
        ev.prevent_default();
        if pending_action.get().is_some() {
            return;
        }
        let payload = MessageTemplateTestSendRequest {
            template_key: test_template_key.get(),
            channel: test_channel.get(),
            recipient: test_recipient.get(),
        };
        pending_action.set(Some("test-send".into()));
        spawn_local(async move {
            match api::record_message_template_test_send(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    refresh_nonce.update(|value| *value += 1);
                }
                Err(error) => feedback.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    let save_preferences = move |ev: SubmitEvent| {
        ev.prevent_default();
        if pending_action.get().is_some() {
            return;
        }
        let escalation = escalation_minutes.get().trim().parse::<i32>().ok();
        let payload = UpsertNotificationPreferenceRequest {
            event_key: event_key.get(),
            email_enabled: email_enabled.get(),
            in_app_enabled: in_app_enabled.get(),
            sms_enabled: sms_enabled.get(),
            push_enabled: push_enabled.get(),
            quiet_hours_start: Some(quiet_start.get()).filter(|value| !value.trim().is_empty()),
            quiet_hours_end: Some(quiet_end.get()).filter(|value| !value.trim().is_empty()),
            timezone: Some(timezone.get()).filter(|value| !value.trim().is_empty()),
            escalation_minutes: escalation,
        };
        pending_action.set(Some("preferences".into()));
        spawn_local(async move {
            match api::upsert_notification_preference(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    refresh_nonce.update(|value| *value += 1);
                }
                Err(error) => feedback.set(Some(error)),
            }
            pending_action.set(None);
        });
    };

    view! {
        {move || {
            if !auth.session_ready.get() {
                view! {
                    <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                        "Loading notification center..."
                    </section>
                }.into_any()
            } else if !can_view.get() {
                view! {
                    <section style="padding:1rem;border:1px solid #fecaca;border-radius:0.5rem;background:#fff1f2;color:#be123c;">
                        "Sign in before opening notifications."
                    </section>
                }.into_any()
            } else {
                view! {
                    <article style="display:grid;gap:1.25rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div>
                                <h2>"Notification Center"</h2>
                                <p style="margin:0;color:#64748b;">
                                    {move || screen.get().map(|value| format!("{} unread operational notification(s)", value.unread_count)).unwrap_or_else(|| "Operational notifications, preferences, and communication policy.".into())}
                                </p>
                            </div>
                            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                <button type="button" class="shell-action secondary" on:click=move |_| refresh_nonce.update(|value| *value += 1) disabled=move || is_loading.get()>
                                    <i class="fas fa-sync-alt"></i>
                                    <span>{move || if is_loading.get() { "Refreshing" } else { "Refresh" }}</span>
                                </button>
                                <button type="button" class="shell-action" on:click=move |_| mark_read(None, true) disabled=move || pending_action.get().is_some()>
                                    <i class="fas fa-check-double"></i>
                                    <span>"Mark all read"</span>
                                </button>
                            </div>
                        </section>

                        {move || feedback.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #bfdbfe;border-radius:0.5rem;background:#eff6ff;color:#1d4ed8;">
                                {message}
                            </section>
                        })}

                        <section style="display:grid;gap:0.75rem;">
                            {move || screen.get().map(|value| {
                                if value.notifications.is_empty() {
                                    view! {
                                        <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;color:#64748b;">
                                            "No operational notifications yet."
                                        </section>
                                    }.into_any()
                                } else {
                                    value.notifications.into_iter().map(|row| {
                                        let priority_style = tone_style(&row.priority);
                                        let notification_id = row.id;
                                        view! {
                                            <section style="display:grid;gap:0.5rem;padding:1rem;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                                                    <strong>{row.subject}</strong>
                                                    <span style=priority_style>{row.priority}</span>
                                                </div>
                                                <p style="margin:0;color:#475569;">{row.body}</p>
                                                <div style="display:flex;gap:0.75rem;flex-wrap:wrap;color:#64748b;">
                                                    <small>{row.event_key}</small>
                                                    <small>{row.category}</small>
                                                    <small>{row.created_at}</small>
                                                    <small>{format!("Channels: {}", row.channels.join(", "))}</small>
                                                </div>
                                                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                                    {row.action_href.map(|href| view! {
                                                        <a class="shell-action secondary" href=href>
                                                            <i class="fas fa-arrow-right"></i>
                                                            <span>"Open"</span>
                                                        </a>
                                                    })}
                                                    {row.read_at.is_none().then(|| view! {
                                                        <button type="button" class="shell-action secondary" on:click=move |_| mark_read(Some(notification_id), false) disabled=move || pending_action.get().is_some()>
                                                            <i class="fas fa-check"></i>
                                                            <span>"Mark read"</span>
                                                        </button>
                                                    })}
                                                </div>
                                            </section>
                                        }
                                    }).collect_view().into_any()
                                }
                            })}
                        </section>

                        <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:1rem;align-items:start;">
                            <form on:submit=save_preferences style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                <h3 style="margin:0;">"Preferences"</h3>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Event key"</span>
                                    <select prop:value=move || event_key.get() on:change=move |ev| event_key.set(event_target_value(&ev))>
                                        <option value="*">"All events"</option>
                                        {move || screen.get().map(|value| value.coverage_rules.into_iter().map(|rule| {
                                            let key = rule.event_key;
                                            view! { <option value=key.clone()>{key.clone()}</option> }
                                        }).collect_view())}
                                    </select>
                                </label>
                                <div style="display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:0.5rem;">
                                    <label><input type="checkbox" prop:checked=move || in_app_enabled.get() on:change=move |ev| in_app_enabled.set(event_target_checked(&ev)) />" In-app"</label>
                                    <label><input type="checkbox" prop:checked=move || email_enabled.get() on:change=move |ev| email_enabled.set(event_target_checked(&ev)) />" Email"</label>
                                    <label><input type="checkbox" prop:checked=move || sms_enabled.get() on:change=move |ev| sms_enabled.set(event_target_checked(&ev)) />" SMS"</label>
                                    <label><input type="checkbox" prop:checked=move || push_enabled.get() on:change=move |ev| push_enabled.set(event_target_checked(&ev)) />" Push"</label>
                                </div>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"Quiet start"</span><input type="time" prop:value=move || quiet_start.get() on:input=move |ev| quiet_start.set(event_target_value(&ev)) /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Quiet end"</span><input type="time" prop:value=move || quiet_end.get() on:input=move |ev| quiet_end.set(event_target_value(&ev)) /></label>
                                </div>
                                <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;"><span>"Timezone"</span><input type="text" prop:value=move || timezone.get() on:input=move |ev| timezone.set(event_target_value(&ev)) /></label>
                                    <label style="display:grid;gap:0.35rem;"><span>"Escalation minutes"</span><input type="number" min="0" prop:value=move || escalation_minutes.get() on:input=move |ev| escalation_minutes.set(event_target_value(&ev)) /></label>
                                </div>
                                <button type="submit" class="shell-action" disabled=move || pending_action.get().is_some()>
                                    <i class="fas fa-save"></i>
                                    <span>"Save preferences"</span>
                                </button>
                            </form>

                            <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                <h3 style="margin:0;">"Channel decisions"</h3>
                                {move || screen.get().map(|value| value.provider_decisions.into_iter().map(|row| {
                                    let status_style = tone_style(&row.decision_status);
                                    view! {
                                        <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                <strong>{format!("{} | {}", row.channel, row.provider_name)}</strong>
                                                <span style=status_style>{row.decision_status}</span>
                                            </div>
                                            <small style="color:#64748b;">{row.compliance_notes}</small>
                                        </div>
                                    }
                                }).collect_view())}
                            </section>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                <div>
                                    <h3 style="margin:0;">"Deliverability governance"</h3>
                                    <p style="margin:0.25rem 0 0;color:#64748b;">"Sender authentication, template approval, suppressions, and high-risk fallback monitoring."</p>
                                </div>
                                <form on:submit=record_test_send style="display:flex;gap:0.5rem;flex-wrap:wrap;align-items:flex-end;">
                                    <label style="display:grid;gap:0.25rem;">
                                        <span>"Template"</span>
                                        <select prop:value=move || test_template_key.get() on:change=move |ev| test_template_key.set(event_target_value(&ev))>
                                            {move || governance.get().map(|value| value.template_governance.into_iter().map(|row| {
                                                let key = row.template_key;
                                                view! { <option value=key.clone()>{key.clone()}</option> }
                                            }).collect_view())}
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.25rem;">
                                        <span>"Channel"</span>
                                        <select prop:value=move || test_channel.get() on:change=move |ev| test_channel.set(event_target_value(&ev))>
                                            <option value="email">"Email"</option>
                                            <option value="sms">"SMS"</option>
                                            <option value="push">"Push"</option>
                                            <option value="in_app">"In-app"</option>
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.25rem;">
                                        <span>"Recipient"</span>
                                        <input type="text" placeholder="ops@example.com" prop:value=move || test_recipient.get() on:input=move |ev| test_recipient.set(event_target_value(&ev)) />
                                    </label>
                                    <button type="submit" class="shell-action secondary" disabled=move || pending_action.get().is_some()>
                                        <i class="fas fa-paper-plane"></i>
                                        <span>"Record test-send"</span>
                                    </button>
                                </form>
                            </div>

                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:1rem;align-items:start;">
                                <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                    <h4 style="margin:0;">"Sender identities"</h4>
                                    {move || governance.get().map(|value| value.sender_identities.into_iter().map(|row| {
                                        let status_style = tone_style(&row.identity_status);
                                        view! {
                                            <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                    <strong>{row.from_email}</strong>
                                                    <span style=status_style>{row.identity_status}</span>
                                                </div>
                                                <small style="color:#64748b;">{format!("{} | {} | SPF {} | DKIM {} | DMARC {}", row.environment_key, row.sender_domain, row.spf_status, row.dkim_status, row.dmarc_status)}</small>
                                                {row.notes.map(|notes| view! { <small style="color:#64748b;">{notes}</small> })}
                                            </div>
                                        }
                                    }).collect_view())}
                                </section>

                                <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                    <h4 style="margin:0;">"Suppression list"</h4>
                                    {move || governance.get().map(|value| {
                                        if value.suppression_entries.is_empty() {
                                            view! { <small style="color:#64748b;">"No active suppressions recorded for this tenant."</small> }.into_any()
                                        } else {
                                            value.suppression_entries.into_iter().map(|row| {
                                                let status_style = tone_style(&row.status);
                                                view! {
                                                    <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                            <strong>{row.recipient}</strong>
                                                            <span style=status_style>{row.status}</span>
                                                        </div>
                                                        <small style="color:#64748b;">{format!("{} | {}", row.channel, row.suppression_reason)}</small>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    })}
                                </section>
                            </div>

                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:980px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style="padding:0.75rem;">"Template"</th>
                                            <th style="padding:0.75rem;">"Channel"</th>
                                            <th style="padding:0.75rem;">"Locale"</th>
                                            <th style="padding:0.75rem;">"Owner"</th>
                                            <th style="padding:0.75rem;">"Approval"</th>
                                            <th style="padding:0.75rem;">"Risk"</th>
                                            <th style="padding:0.75rem;">"Test-send"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || governance.get().map(|value| value.template_governance.into_iter().map(|row| {
                                            let approval_style = tone_style(&row.approval_status);
                                            view! {
                                                <tr style="border-top:1px solid #e5e7eb;">
                                                    <td style="padding:0.75rem;"><code>{row.template_key}</code></td>
                                                    <td style="padding:0.75rem;">{row.channel}</td>
                                                    <td style="padding:0.75rem;">{format!("{} v{}", row.locale, row.version)}</td>
                                                    <td style="padding:0.75rem;">{row.owner_team}</td>
                                                    <td style="padding:0.75rem;"><span style=approval_style>{row.approval_status}</span></td>
                                                    <td style="padding:0.75rem;">{if row.high_risk { "High" } else { "Standard" }}</td>
                                                    <td style="padding:0.75rem;">{if row.test_send_required { "Required" } else { "Optional" }}</td>
                                                </tr>
                                            }
                                        }).collect_view())}
                                    </tbody>
                                </table>
                            </div>

                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:920px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style="padding:0.75rem;">"Rule"</th>
                                            <th style="padding:0.75rem;">"Category"</th>
                                            <th style="padding:0.75rem;">"Priority"</th>
                                            <th style="padding:0.75rem;">"Fallback"</th>
                                            <th style="padding:0.75rem;">"Escalation"</th>
                                            <th style="padding:0.75rem;">"Sender auth"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || governance.get().map(|value| value.monitoring_rules.into_iter().map(|row| {
                                            let priority_style = tone_style(&row.priority);
                                            view! {
                                                <tr style="border-top:1px solid #e5e7eb;">
                                                    <td style="padding:0.75rem;"><code>{row.rule_key}</code></td>
                                                    <td style="padding:0.75rem;">{row.category}</td>
                                                    <td style="padding:0.75rem;"><span style=priority_style>{row.priority}</span></td>
                                                    <td style="padding:0.75rem;">{row.fallback_channel}</td>
                                                    <td style="padding:0.75rem;">{format!("{} min", row.escalation_minutes)}</td>
                                                    <td style="padding:0.75rem;">{if row.required_sender_identity { "Required" } else { "Not required" }}</td>
                                                </tr>
                                            }
                                        }).collect_view())}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <div>
                                <h3 style="margin:0;">"Tenant branding"</h3>
                                <p style="margin:0.25rem 0 0;color:#64748b;">"Portal, document, email, asset-review, and custom-domain status for customer-facing identity."</p>
                            </div>

                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:1rem;align-items:start;">
                                <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                    <h4 style="margin:0;">"Branding policy"</h4>
                                    {move || governance.get().map(|value| {
                                        if value.branding_policies.is_empty() {
                                            view! { <small style="color:#64748b;">"No tenant branding policy is configured yet; STLoads fallback branding remains in force."</small> }.into_any()
                                        } else {
                                            value.branding_policies.into_iter().map(|row| {
                                                let status_style = tone_style(&row.white_label_status);
                                                view! {
                                                    <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                            <strong>{row.fallback_brand_name}</strong>
                                                            <span style=status_style>{row.white_label_status}</span>
                                                        </div>
                                                        <small style="color:#64748b;">{format!("Portal {} | Documents {} | Email {} | Custom domains {} | Cache v{}",
                                                            if row.portal_branding_enabled { "on" } else { "off" },
                                                            if row.document_branding_enabled { "on" } else { "off" },
                                                            if row.email_branding_enabled { "on" } else { "off" },
                                                            if row.custom_domain_enabled { "on" } else { "off" },
                                                            row.cache_version)}</small>
                                                        <small style="color:#64748b;">{row.unsupported_message}</small>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    })}
                                </section>

                                <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                    <h4 style="margin:0;">"Brand assets"</h4>
                                    {move || governance.get().map(|value| {
                                        if value.brand_assets.is_empty() {
                                            view! { <small style="color:#64748b;">"No reviewed customer assets are configured."</small> }.into_any()
                                        } else {
                                            value.brand_assets.into_iter().map(|row| {
                                                let status_style = tone_style(&row.review_status);
                                                view! {
                                                    <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                            <strong>{row.asset_type}</strong>
                                                            <span style=status_style>{row.review_status}</span>
                                                        </div>
                                                        <small style="color:#64748b;">{format!("{} | {} bytes | {}", row.mime_type, row.file_size_bytes, row.cache_key)}</small>
                                                        {row.notes.map(|notes| view! { <small style="color:#64748b;">{notes}</small> })}
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    })}
                                </section>
                            </div>

                            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:1rem;align-items:start;">
                                <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                    <h4 style="margin:0;">"Custom domains"</h4>
                                    {move || governance.get().map(|value| {
                                        if value.custom_domains.is_empty() {
                                            view! { <small style="color:#64748b;">"No custom domains are configured. Sales cannot promise custom domains without product approval."</small> }.into_any()
                                        } else {
                                            value.custom_domains.into_iter().map(|row| {
                                                let status_style = tone_style(&row.verification_status);
                                                view! {
                                                    <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                            <strong>{row.domain}</strong>
                                                            <span style=status_style>{row.verification_status}</span>
                                                        </div>
                                                        <small style="color:#64748b;">{format!("{} | TLS {} | rollback {}", row.purpose, row.tls_status, row.rollback_status)}</small>
                                                        <small style="color:#64748b;">{format!("DNS: {} = {}", row.dns_txt_name, row.dns_txt_value)}</small>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    })}
                                </section>

                                <section style="display:grid;gap:0.75rem;border:1px solid #e5e7eb;border-radius:0.5rem;padding:1rem;background:white;">
                                    <h4 style="margin:0;">"Branded templates"</h4>
                                    {move || governance.get().map(|value| {
                                        if value.branded_template_rules.is_empty() {
                                            view! { <small style="color:#64748b;">"No tenant-branded document or email template overrides are configured."</small> }.into_any()
                                        } else {
                                            value.branded_template_rules.into_iter().map(|row| {
                                                let status_style = tone_style(&row.branding_status);
                                                view! {
                                                    <div style="display:grid;gap:0.35rem;padding-top:0.75rem;border-top:1px solid #f1f5f9;">
                                                        <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;">
                                                            <strong>{row.template_surface}</strong>
                                                            <span style=status_style>{row.branding_status}</span>
                                                        </div>
                                                        <small style="color:#64748b;">{format!("{} | fallback {}", row.template_key, if row.fallback_allowed { "allowed" } else { "blocked" })}</small>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    })}
                                </section>
                            </div>
                        </section>

                        <section style="display:grid;gap:1rem;">
                            <h3 style="margin:0;">"Coverage"</h3>
                            <div style="overflow:auto;border:1px solid #e5e7eb;border-radius:0.5rem;background:white;">
                                <table style="width:100%;border-collapse:collapse;min-width:820px;">
                                    <thead>
                                        <tr style="text-align:left;background:#f8fafc;">
                                            <th style="padding:0.75rem;">"Event"</th>
                                            <th style="padding:0.75rem;">"Category"</th>
                                            <th style="padding:0.75rem;">"Priority"</th>
                                            <th style="padding:0.75rem;">"Channels"</th>
                                            <th style="padding:0.75rem;">"Owner"</th>
                                            <th style="padding:0.75rem;">"Escalation"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {move || screen.get().map(|value| value.coverage_rules.into_iter().map(|row| {
                                            let priority_style = tone_style(&row.default_priority);
                                            view! {
                                                <tr style="border-top:1px solid #e5e7eb;">
                                                    <td style="padding:0.75rem;"><code>{row.event_key}</code></td>
                                                    <td style="padding:0.75rem;">{row.category}</td>
                                                    <td style="padding:0.75rem;"><span style=priority_style>{row.default_priority}</span></td>
                                                    <td style="padding:0.75rem;">{row.default_channels.join(", ")}</td>
                                                    <td style="padding:0.75rem;">{row.responsible_party}</td>
                                                    <td style="padding:0.75rem;">{row.escalation_minutes.map(|value| format!("{} min", value)).unwrap_or_else(|| "-".into())}</td>
                                                </tr>
                                            }
                                        }).collect_view())}
                                    </tbody>
                                </table>
                            </div>
                        </section>
                    </article>
                }.into_any()
            }
        }}
    }
}
