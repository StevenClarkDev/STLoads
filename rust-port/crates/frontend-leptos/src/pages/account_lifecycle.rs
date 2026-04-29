use crate::{
    api,
    session::{self, use_auth},
};
use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{
    AdminCreateUserRequest, AdminUserDirectoryScreen, AdminUserDirectoryUser, OtpPurpose,
    ResendOtpRequest, ReviewOnboardingRequest,
};

use super::admin_guard_view;

#[component]
pub fn AccountLifecyclePage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminUserDirectoryScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let search_query = RwSignal::new(String::new());
    let refresh_nonce = RwSignal::new(0_u64);
    let action_loading_user_id = RwSignal::new(None::<u64>);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_users")
    });

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if !auth.session_ready.get() || !auth.session.get().authenticated || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_admin_user_directory().await {
                Ok(next) => {
                    screen.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(&auth, "Your session expired; sign in again.");
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Lifecycle Workspace", &["access_admin_portal", "manage_users"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1.1rem;">
                        {move || feedback.get().map(|message| view! {
                            <section style="padding:0.85rem 1rem;border:1px solid #dbeafe;border-radius:0.9rem;background:#eff6ff;color:#1d4ed8;white-space:pre-wrap;">
                                {message}
                            </section>
                        })}

                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div style="max-width:760px;color:#475569;">
                                "Search by name, email, role, company, or lifecycle status before resending OTPs or taking review actions."
                            </div>
                            <div style="min-width:280px;">
                                <input
                                    type="text"
                                    placeholder="Search lifecycle accounts"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                    style="width:100%;padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.9rem;"
                                />
                            </div>
                        </section>

                        {move || if loading.get() && screen.get().is_none() {
                            view! { <p>"Loading account lifecycle data..."</p> }.into_any()
                        } else if let Some(screen_data) = screen.get() {
                            render_workspace(
                                screen_data,
                                search_query.get(),
                                feedback,
                                action_loading_user_id,
                                refresh_nonce,
                            ).into_any()
                        } else {
                            view! { <p>"Lifecycle data is not available yet."</p> }.into_any()
                        }}
                    </article>
                }.into_any()
            }
        }}
    }
}

fn render_workspace(
    screen_data: AdminUserDirectoryScreen,
    search_query: String,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
) -> impl IntoView {
    let pending_otp_users = screen_data
        .users
        .iter()
        .filter(|user| user.status_key == "pending_otp")
        .filter(|user| lifecycle_user_matches_query(user, &search_query))
        .cloned()
        .collect::<Vec<_>>();
    let pending_review_users = screen_data
        .users
        .iter()
        .filter(|user| user.status_key == "pending_review")
        .filter(|user| lifecycle_user_matches_query(user, &search_query))
        .cloned()
        .collect::<Vec<_>>();
    let revision_users = screen_data
        .users
        .iter()
        .filter(|user| user.status_key == "revision_requested")
        .filter(|user| lifecycle_user_matches_query(user, &search_query))
        .cloned()
        .collect::<Vec<_>>();
    let rejected_users = screen_data
        .users
        .iter()
        .filter(|user| user.status_key == "rejected")
        .filter(|user| lifecycle_user_matches_query(user, &search_query))
        .cloned()
        .collect::<Vec<_>>();

    view! {
        <>
            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:0.75rem;">
                <LifecycleSummaryCard title="Pending OTP" value=pending_otp_users.len().to_string() tone="info" />
                <LifecycleSummaryCard title="Pending Review" value=pending_review_users.len().to_string() tone="warning" />
                <LifecycleSummaryCard title="Revision Requested" value=revision_users.len().to_string() tone="primary" />
                <LifecycleSummaryCard title="Rejected" value=rejected_users.len().to_string() tone="danger" />
            </section>

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;">
                <LifecycleCreateCard
                    title="Create Fresh Pending OTP"
                    status_key="pending_otp"
                    accent="#0369a1"
                    feedback=feedback
                    refresh_nonce=refresh_nonce
                />
                <LifecycleCreateCard
                    title="Create Fresh Pending Review"
                    status_key="pending_review"
                    accent="#b45309"
                    feedback=feedback
                    refresh_nonce=refresh_nonce
                />
            </section>

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                <LifecycleLane
                    title="Pending OTP Support Lane"
                    tone="info"
                >
                    {if pending_otp_users.is_empty() {
                        view! { <p style="margin:0;color:#64748b;">"No pending OTP accounts are queued right now."</p> }.into_any()
                    } else {
                        pending_otp_users
                            .into_iter()
                            .map(|user| view! {
                                <PendingOtpRow
                                    user=user
                                    feedback=feedback
                                    action_loading_user_id=action_loading_user_id
                                    refresh_nonce=refresh_nonce
                                />
                            })
                            .collect_view()
                            .into_any()
                    }}
                </LifecycleLane>

                <LifecycleLane
                    title="Pending Review Queue"
                    tone="warning"
                >
                    {if pending_review_users.is_empty() {
                        view! { <p style="margin:0;color:#64748b;">"No pending review accounts are queued right now."</p> }.into_any()
                    } else {
                        pending_review_users
                            .into_iter()
                            .map(|user| view! {
                                <PendingReviewRow
                                    user=user
                                    feedback=feedback
                                    action_loading_user_id=action_loading_user_id
                                    refresh_nonce=refresh_nonce
                                />
                            })
                            .collect_view()
                            .into_any()
                    }}
                </LifecycleLane>
            </section>

            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:1rem;align-items:start;">
                <LifecycleLane
                    title="Revision Requested Lane"
                    tone="primary"
                >
                    {if revision_users.is_empty() {
                        view! { <p style="margin:0;color:#64748b;">"No revision-requested accounts are in this queue right now."</p> }.into_any()
                    } else {
                        revision_users
                            .into_iter()
                            .map(|user| view! { <StatusHistoryRow user=user tone="primary" /> })
                            .collect_view()
                            .into_any()
                    }}
                </LifecycleLane>

                <LifecycleLane
                    title="Rejected Lane"
                    tone="danger"
                >
                    {if rejected_users.is_empty() {
                        view! { <p style="margin:0;color:#64748b;">"No rejected accounts are in this queue right now."</p> }.into_any()
                    } else {
                        rejected_users
                            .into_iter()
                            .map(|user| view! { <StatusHistoryRow user=user tone="danger" /> })
                            .collect_view()
                            .into_any()
                    }}
                </LifecycleLane>
            </section>
        </>
    }
}

#[component]
fn LifecycleSummaryCard(title: &'static str, value: String, tone: &'static str) -> impl IntoView {
    view! {
        <div style="padding:0.95rem 1rem;border:1px solid #e5e7eb;border-radius:1rem;background:white;display:grid;gap:0.35rem;">
            <div style="display:flex;justify-content:space-between;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                <strong>{title}</strong>
                <span style=badge_style(tone)>{tone.replace('_', " ")}</span>
            </div>
            <div style="font-size:1.35rem;font-weight:700;color:#111827;">{value}</div>
        </div>
    }
}

#[component]
fn LifecycleLane(title: &'static str, tone: &'static str, children: Children) -> impl IntoView {
    view! {
        <section style="display:grid;gap:0.75rem;padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#ffffff;">
            <div style="display:grid;gap:0.25rem;">
                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                    <strong>{title}</strong>
                    <span style=badge_style(tone)>{tone.replace('_', " ")}</span>
                </div>
            </div>
            <div style="display:grid;gap:0.7rem;">
                {children()}
            </div>
        </section>
    }
}

#[component]
fn LifecycleCreateCard(
    title: &'static str,
    status_key: &'static str,
    accent: &'static str,
    feedback: RwSignal<Option<String>>,
    refresh_nonce: RwSignal<u64>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let password_confirmation = RwSignal::new(String::new());
    let role = RwSignal::new(String::from("shipper"));
    let phone = RwSignal::new(String::new());
    let address = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);

    let submit_create = move |ev: SubmitEvent| {
        ev.prevent_default();
        let payload = AdminCreateUserRequest {
            name: name.get(),
            email: email.get(),
            password: password.get(),
            password_confirmation: password_confirmation.get(),
            role_key: role.get(),
            status_key: status_key.into(),
            phone_no: optional_string(phone.get()),
            address: optional_string(address.get()),
        };

        is_submitting.set(true);
        spawn_local(async move {
            match api::create_admin_user(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    if response.success {
                        name.set(String::new());
                        email.set(String::new());
                        password.set(String::new());
                        password_confirmation.set(String::new());
                        phone.set(String::new());
                        address.set(String::new());
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            is_submitting.set(false);
        });
    };

    view! {
        <form on:submit=submit_create style="display:grid;gap:0.8rem;padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:white;box-shadow:0 12px 30px rgba(15,23,42,0.04);">
            <div style="display:grid;gap:0.25rem;">
                <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:center;flex-wrap:wrap;">
                    <strong>{title}</strong>
                    <span style=format!("background:{}1A;color:{};padding:0.28rem 0.7rem;border-radius:999px;font-size:0.82rem;font-weight:700;", accent, accent)>{status_key.replace('_', " ")}</span>
                </div>
            </div>

            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.7rem;">
                <input type="text" placeholder="Full name" prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) />
                <input type="email" placeholder="Email" prop:value=move || email.get() on:input=move |ev| email.set(event_target_value(&ev)) />
                <select prop:value=move || role.get() on:change=move |ev| role.set(event_target_value(&ev))>
                    <option value="shipper">"Shipper"</option>
                    <option value="carrier">"Carrier"</option>
                    <option value="broker">"Broker"</option>
                    <option value="freight_forwarder">"Freight Forwarder"</option>
                </select>
                <input type="text" placeholder="Phone" prop:value=move || phone.get() on:input=move |ev| phone.set(event_target_value(&ev)) />
                <input type="text" placeholder="Address" prop:value=move || address.get() on:input=move |ev| address.set(event_target_value(&ev)) />
                <input type="password" placeholder="Password" prop:value=move || password.get() on:input=move |ev| password.set(event_target_value(&ev)) />
                <input type="password" placeholder="Confirm password" prop:value=move || password_confirmation.get() on:input=move |ev| password_confirmation.set(event_target_value(&ev)) />
            </div>

            <div style="display:flex;justify-content:flex-end;">
                <button type="submit" disabled=move || is_submitting.get() style=format!("padding:0.7rem 1rem;border:none;border-radius:0.85rem;background:{};color:white;cursor:pointer;font-weight:700;", accent)>
                    {move || if is_submitting.get() { "Creating..." } else { "Create account" }}
                </button>
            </div>
        </form>
    }
}

#[component]
fn PendingOtpRow(
    user: AdminUserDirectoryUser,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
) -> impl IntoView {
    let email = user.email.clone();

    view! {
        <article style="display:grid;gap:0.55rem;padding:0.9rem;border:1px solid #dbeafe;border-radius:0.95rem;background:#f8fbff;">
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>{user.name}</strong>
                    <small>{format!("{} | {}", user.role_label, user.email)}</small>
                    <small style="color:#64748b;">{format!("Joined {} | {} document(s)", user.joined_at_label, user.document_count)}</small>
                </div>
                <span style=badge_style("info")>{user.status_label}</span>
            </div>
            <small style="color:#475569;">"This user is still waiting on verification. Resend the registration OTP from here to help them continue."</small>
            <div style="display:flex;justify-content:flex-end;">
                <button
                    type="button"
                    disabled=move || action_loading_user_id.get() == Some(user.user_id)
                    on:click=move |_| {
                        action_loading_user_id.set(Some(user.user_id));
                        let email = email.clone();
                        spawn_local(async move {
                            match api::resend_otp(&ResendOtpRequest {
                                email,
                                purpose: OtpPurpose::Registration,
                            }).await {
                                Ok(response) => {
                                    feedback.set(Some(response.message));
                                    refresh_nonce.update(|value| *value += 1);
                                }
                                Err(error) => feedback.set(Some(error)),
                            }
                            action_loading_user_id.set(None);
                        });
                    }
                    style="padding:0.55rem 0.8rem;border:none;border-radius:0.75rem;background:#0369a1;color:white;cursor:pointer;"
                >
                    {move || if action_loading_user_id.get() == Some(user.user_id) { "Sending..." } else { "Resend OTP" }}
                </button>
            </div>
        </article>
    }
}

#[component]
fn PendingReviewRow(
    user: AdminUserDirectoryUser,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
) -> impl IntoView {
    let remarks = RwSignal::new(user.latest_review_note.clone().unwrap_or_default());

    view! {
        <article style="display:grid;gap:0.65rem;padding:0.9rem;border:1px solid #fde68a;border-radius:0.95rem;background:#fffbeb;">
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>{user.name}</strong>
                    <small>{format!("{} | {}", user.role_label, user.email)}</small>
                    <small style="color:#64748b;">{format!("Joined {} | {} document(s)", user.joined_at_label, user.document_count)}</small>
                </div>
                <span style=badge_style("warning")>{user.status_label}</span>
            </div>
            <small style="color:#92400e;">"This user is in the review decision stage. Keep the review note visible so the next action is clear."</small>
            <textarea
                rows="2"
                placeholder="Admin review note"
                prop:value=move || remarks.get()
                on:input=move |ev| remarks.set(event_target_value(&ev))
            ></textarea>
            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                <LifecycleActionButton
                    user_id=user.user_id
                    action_loading_user_id=action_loading_user_id
                    label="Approve"
                    accent="#166534"
                    on_click=move || {
                        run_review_action(
                            user.user_id,
                            "approve",
                            optional_string(remarks.get()),
                            feedback,
                            action_loading_user_id,
                            refresh_nonce,
                        );
                    }
                />
                <LifecycleActionButton
                    user_id=user.user_id
                    action_loading_user_id=action_loading_user_id
                    label="Request Revision"
                    accent="#b45309"
                    on_click=move || {
                        run_review_action(
                            user.user_id,
                            "revision",
                            optional_string(remarks.get()).or_else(|| Some("Please revise and resubmit from the account lifecycle workspace.".into())),
                            feedback,
                            action_loading_user_id,
                            refresh_nonce,
                        );
                    }
                />
                <LifecycleActionButton
                    user_id=user.user_id
                    action_loading_user_id=action_loading_user_id
                    label="Reject"
                    accent="#be123c"
                    on_click=move || {
                        run_review_action(
                            user.user_id,
                            "reject",
                            optional_string(remarks.get()).or_else(|| Some("Rejected from the account lifecycle workspace.".into())),
                            feedback,
                            action_loading_user_id,
                            refresh_nonce,
                        );
                    }
                />
            </div>
        </article>
    }
}

#[component]
fn LifecycleActionButton<F>(
    user_id: u64,
    action_loading_user_id: RwSignal<Option<u64>>,
    label: &'static str,
    accent: &'static str,
    on_click: F,
) -> impl IntoView
where
    F: Fn() + 'static + Copy,
{
    view! {
        <button
            type="button"
            disabled=move || action_loading_user_id.get() == Some(user_id)
            on:click=move |_| on_click()
            style=format!("padding:0.55rem 0.8rem;border:none;border-radius:0.75rem;background:{};color:white;cursor:pointer;", accent)
        >
            {move || if action_loading_user_id.get() == Some(user_id) { "Working..." } else { label }}
        </button>
    }
}

#[component]
fn StatusHistoryRow(user: AdminUserDirectoryUser, tone: &'static str) -> impl IntoView {
    view! {
        <article style="display:grid;gap:0.45rem;padding:0.85rem;border:1px solid #e5e7eb;border-radius:0.9rem;background:#fafaf9;">
            <div style="display:flex;justify-content:space-between;gap:0.75rem;align-items:start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>{user.name}</strong>
                    <small>{format!("{} | {}", user.role_label, user.email)}</small>
                    <small style="color:#64748b;">{format!("Joined {} | {} document(s)", user.joined_at_label, user.document_count)}</small>
                </div>
                <span style=badge_style(tone)>{user.status_label}</span>
            </div>
            {user.company_name.map(|company| view! {
                <small style="color:#475569;">{format!("Company: {}", company)}</small>
            })}
            {user.latest_review_note.map(|note| view! {
                <small style="color:#475569;white-space:pre-wrap;">{format!("Latest admin note: {}", note)}</small>
            }).unwrap_or_else(|| view! {
                <small style="color:#94a3b8;">{"No admin note was captured on the latest history row.".to_string()}</small>
            })}
        </article>
    }
}

fn run_review_action(
    user_id: u64,
    decision: &'static str,
    remarks: Option<String>,
    feedback: RwSignal<Option<String>>,
    action_loading_user_id: RwSignal<Option<u64>>,
    refresh_nonce: RwSignal<u64>,
) {
    action_loading_user_id.set(Some(user_id));
    spawn_local(async move {
        let result = api::review_onboarding_user(
            user_id,
            &ReviewOnboardingRequest {
                decision: decision.into(),
                remarks,
            },
        )
        .await;

        match result {
            Ok(response) => {
                feedback.set(Some(response.message));
                if response.success {
                    refresh_nonce.update(|value| *value += 1);
                }
            }
            Err(error) => feedback.set(Some(error)),
        }

        action_loading_user_id.set(None);
    });
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn badge_style(tone: &str) -> &'static str {
    match tone {
        "success" => "background:#e8fff3;padding:0.25rem 0.6rem;border-radius:999px;color:#0f766e;",
        "warning" => "background:#fff7dd;padding:0.25rem 0.6rem;border-radius:999px;color:#b45309;",
        "danger" => "background:#ffe4e6;padding:0.25rem 0.6rem;border-radius:999px;color:#be123c;",
        "info" => "background:#e0f2fe;padding:0.25rem 0.6rem;border-radius:999px;color:#0369a1;",
        "primary" => "background:#ede9fe;padding:0.25rem 0.6rem;border-radius:999px;color:#6d28d9;",
        _ => "background:#f1f5f9;padding:0.25rem 0.6rem;border-radius:999px;color:#475569;",
    }
}

fn lifecycle_user_matches_query(user: &AdminUserDirectoryUser, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return true;
    }

    user.name.to_ascii_lowercase().contains(&query)
        || user.email.to_ascii_lowercase().contains(&query)
        || user.role_label.to_ascii_lowercase().contains(&query)
        || user.status_label.to_ascii_lowercase().contains(&query)
        || user.status_key.to_ascii_lowercase().contains(&query)
        || user
            .company_name
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&query)
        || user
            .latest_review_note
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains(&query)
}
