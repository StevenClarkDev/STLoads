use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::components::A;
use shared::{
    AdminCreateSupportCaseRequest, AdminCreateSupportNoteRequest, AdminSupportCaseFeedbackRequest,
    AdminSupportCaseScreen, AdminSupportSearchScreen, AdminSupportTimelineScreen,
    AdminUpdateSupportCaseRequest,
};

use crate::{
    api,
    session::{self, use_auth},
};

use super::admin_guard_view;

#[component]
pub fn SupportSearchPage() -> impl IntoView {
    let auth = use_auth();
    let query = RwSignal::new(String::new());
    let target_organization_id = RwSignal::new(String::new());
    let screen = RwSignal::new(None::<AdminSupportSearchScreen>);
    let cases = RwSignal::new(None::<AdminSupportCaseScreen>);
    let timeline = RwSignal::new(None::<AdminSupportTimelineScreen>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let timeline_loading = RwSignal::new(false);
    let selected_entity_type = RwSignal::new(String::from("general"));
    let selected_entity_id = RwSignal::new(String::new());
    let support_ticket_ref = RwSignal::new(String::new());
    let support_visibility = RwSignal::new(String::from("internal"));
    let support_note = RwSignal::new(String::new());
    let case_title = RwSignal::new(String::new());
    let case_severity = RwSignal::new(String::from("sev3"));
    let case_category = RwSignal::new(String::from("general"));
    let case_customer_impact = RwSignal::new(String::new());
    let case_description = RwSignal::new(String::new());

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_users")
            || session::has_permission(&auth, "view_audit_events")
    });

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let search = query.get().trim().to_string();
        if search.len() < 2 {
            feedback.set(Some("Enter at least two characters.".into()));
            return;
        }
        let target_org = target_organization_id.get().trim().parse::<u64>().ok();
        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::fetch_admin_support_search(&search, target_org).await {
                Ok(next) => {
                    screen.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let load_cases = move || {
        let target_org = target_organization_id.get().trim().parse::<u64>().ok();
        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::fetch_admin_support_cases(target_org).await {
                Ok(next) => {
                    cases.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let submit_case = move |ev: SubmitEvent| {
        ev.prevent_default();
        let target_org = target_organization_id.get().trim().parse::<u64>().ok();
        let payload = AdminCreateSupportCaseRequest {
            target_organization_id: target_org,
            reporter_user_id: None,
            affected_user_id: None,
            related_entity_type: Some(selected_entity_type.get()),
            related_entity_id: optional_string(selected_entity_id.get()),
            channel: "portal".into(),
            severity: case_severity.get(),
            category: case_category.get(),
            owner_team: "Support".into(),
            title: case_title.get(),
            description: case_description.get(),
            customer_impact: case_customer_impact.get(),
            customer_update: Some("Support case opened and is being triaged.".into()),
            internal_note: Some("Created from Rust support console.".into()),
        };
        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::create_admin_support_case(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    cases.set(Some(response.screen));
                    if response.success {
                        case_title.set(String::new());
                        case_customer_impact.set(String::new());
                        case_description.set(String::new());
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let resolve_case = move |case_id: u64| {
        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            let payload = AdminUpdateSupportCaseRequest {
                status: Some("resolved".into()),
                severity: None,
                owner_team: None,
                owner_user_id: None,
                escalation_owner_user_id: None,
                customer_update: Some("Support case resolved.".into()),
                internal_note: Some("Resolved from Rust support console.".into()),
                resolution_reason: Some("resolved_by_support".into()),
                root_cause_category: Some("support_request".into()),
                follow_up_action: Some(
                    "Review recurring patterns during support reporting.".into(),
                ),
            };
            match api::update_admin_support_case(case_id, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    cases.set(Some(response.screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let record_positive_feedback = move |case_id: u64| {
        loading.set(true);
        let auth = auth;
        spawn_local(async move {
            let payload = AdminSupportCaseFeedbackRequest {
                feedback_score: 5,
                feedback_comment: Some("Positive customer feedback recorded by support.".into()),
            };
            match api::record_admin_support_case_feedback(case_id, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    cases.set(Some(response.screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let load_timeline =
        move |entity_type: String, entity_id: Option<String>, organization_id: Option<u64>| {
            selected_entity_type.set(entity_type.clone());
            selected_entity_id.set(entity_id.clone().unwrap_or_default());
            timeline_loading.set(true);
            let auth = auth;
            spawn_local(async move {
                match api::fetch_admin_support_timeline(
                    &entity_type,
                    entity_id.as_deref(),
                    organization_id,
                )
                .await
                {
                    Ok(next) => {
                        timeline.set(Some(next));
                        feedback.set(None);
                    }
                    Err(error) => {
                        if error.contains("returned 401") {
                            session::invalidate_session(
                                &auth,
                                "Your Rust session expired; sign in again.",
                            );
                        }
                        feedback.set(Some(error));
                    }
                }
                timeline_loading.set(false);
            });
        };

    let submit_note = move |ev: SubmitEvent| {
        ev.prevent_default();
        let entity_type = selected_entity_type.get();
        let entity_id = optional_string(selected_entity_id.get());
        let target_org = timeline
            .get()
            .map(|value| value.target_organization_id)
            .or_else(|| target_organization_id.get().trim().parse::<u64>().ok());
        let payload = AdminCreateSupportNoteRequest {
            target_organization_id: target_org,
            entity_type,
            entity_id,
            visibility: support_visibility.get(),
            ticket_ref: optional_string(support_ticket_ref.get()),
            note: support_note.get(),
        };
        timeline_loading.set(true);
        let auth = auth;
        spawn_local(async move {
            match api::create_admin_support_note(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message.clone()));
                    timeline.set(Some(response.timeline));
                    if response.success {
                        support_note.set(String::new());
                    }
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            timeline_loading.set(false);
        });
    };

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Support Search", &["access_admin_portal", "manage_users", "view_audit_events"]) {
                guard
            } else if !can_view.get() {
                view! { <section class="php-alert php-alert-warning">"Support search is restricted for this session."</section> }.into_any()
            } else {
                view! {
                <main class="php-page-shell">
                    <section class="php-dashboard-header">
                        <div>
                            <p class="eyebrow">"Support"</p>
                            <h1>"Support Search"</h1>
                            <p>"Audited lookup for customer support across accounts, freight, documents, payments, and TMS handoffs."</p>
                        </div>
                    </section>

                    <section class="php-card" style="border-radius:8px;">
                        <form on:submit=submit style="display:grid;grid-template-columns:minmax(16rem,1fr) minmax(12rem,16rem) auto;gap:0.75rem;align-items:end;">
                            <label style="display:grid;gap:0.35rem;">
                                <span>"Search"</span>
                                <input
                                    class="php-input"
                                    type="search"
                                    placeholder="Email, load number, document, payment, TMS ID"
                                    prop:value=move || query.get()
                                    on:input=move |ev| query.set(event_target_value(&ev))
                                />
                            </label>
                            <label style="display:grid;gap:0.35rem;">
                                <span>"Organization ID"</span>
                                <input
                                    class="php-input"
                                    inputmode="numeric"
                                    placeholder="Current org"
                                    prop:value=move || target_organization_id.get()
                                    on:input=move |ev| target_organization_id.set(event_target_value(&ev))
                                />
                            </label>
                            <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>
                                {move || if loading.get() { "Searching..." } else { "Search" }}
                            </button>
                        </form>
                        <p style="margin:0.75rem 0 0;color:#64748b;font-size:0.88rem;">
                            "Cross-organization searches require an active break-glass session and are written to the audit ledger."
                        </p>
                    </section>

                    <section class="php-card" style="border-radius:8px;">
                        <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                            <div>
                                <p class="eyebrow">"Support Cases"</p>
                                <h2 style="margin:0;">"Case Management"</h2>
                            </div>
                            <button type="button" class="php-btn php-btn-light" on:click=move |_| load_cases() disabled=move || loading.get()>
                                "Load Cases"
                            </button>
                        </div>

                        <form on:submit=submit_case style="display:grid;gap:0.75rem;margin-top:1rem;border:1px solid #e5e7eb;border-radius:8px;padding:0.9rem;">
                            <div style="display:grid;grid-template-columns:minmax(14rem,1fr) minmax(8rem,10rem) minmax(10rem,12rem);gap:0.75rem;">
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Title"</span>
                                    <input class="php-input" placeholder="Customer cannot reconcile invoice" prop:value=move || case_title.get() on:input=move |ev| case_title.set(event_target_value(&ev)) />
                                </label>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Severity"</span>
                                    <select class="php-input" prop:value=move || case_severity.get() on:change=move |ev| case_severity.set(event_target_value(&ev))>
                                        <option value="sev1">"SEV-1"</option>
                                        <option value="sev2">"SEV-2"</option>
                                        <option value="sev3">"SEV-3"</option>
                                        <option value="sev4">"SEV-4"</option>
                                    </select>
                                </label>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Category"</span>
                                    <input class="php-input" placeholder="payments" prop:value=move || case_category.get() on:input=move |ev| case_category.set(event_target_value(&ev)) />
                                </label>
                            </div>
                            <label style="display:grid;gap:0.35rem;">
                                <span>"Customer Impact"</span>
                                <input class="php-input" placeholder="Who is blocked and why" prop:value=move || case_customer_impact.get() on:input=move |ev| case_customer_impact.set(event_target_value(&ev)) />
                            </label>
                            <label style="display:grid;gap:0.35rem;">
                                <span>"Description"</span>
                                <textarea class="php-input" rows="3" placeholder="Case details and current customer state" prop:value=move || case_description.get() on:input=move |ev| case_description.set(event_target_value(&ev))></textarea>
                            </label>
                            <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>
                                {move || if loading.get() { "Saving..." } else { "Create Case" }}
                            </button>
                        </form>

                        {move || cases.get().map(|cases| view! {
                            <div style="display:grid;gap:0.75rem;margin-top:1rem;">
                                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;color:#475569;">
                                    <span>{format!("Open: {}", cases.open_count)}</span>
                                    <span>{format!("At risk: {}", cases.breach_risk_count)}</span>
                                    <span>{format!("Breached: {}", cases.breached_count)}</span>
                                </div>
                                <For
                                    each=move || cases.rows.clone()
                                    key=|case| case.id
                                    children=move |case| view! {
                                        <article style="border:1px solid #e5e7eb;border-radius:8px;padding:0.9rem;display:grid;gap:0.55rem;">
                                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                                                <div>
                                                    <span style="display:inline-flex;padding:0.2rem 0.55rem;border-radius:999px;background:#f1f5f9;color:#334155;font-size:0.78rem;">{case.case_number.clone()}</span>
                                                    <h3 style="margin:0.45rem 0 0;font-size:1rem;">{case.title.clone()}</h3>
                                                    <p style="margin:0.15rem 0 0;color:#64748b;">{case.customer_impact.clone()}</p>
                                                </div>
                                                <div style="display:flex;gap:0.4rem;flex-wrap:wrap;">
                                                    <button type="button" class="php-btn php-btn-light" on:click={
                                                        let case_id = case.id;
                                                        move |_| resolve_case(case_id)
                                                    }>
                                                        "Resolve"
                                                    </button>
                                                    <button type="button" class="php-btn php-btn-light" on:click={
                                                        let case_id = case.id;
                                                        move |_| record_positive_feedback(case_id)
                                                    }>
                                                        "Record CSAT"
                                                    </button>
                                                </div>
                                            </div>
                                            <div style="display:flex;gap:0.45rem;flex-wrap:wrap;">
                                                <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.25rem 0.55rem;">{case.severity.clone()}</span>
                                                <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.25rem 0.55rem;">{case.status.clone()}</span>
                                                <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.25rem 0.55rem;">{case.breach_state.clone()}</span>
                                                <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.25rem 0.55rem;">{format!("Due {}", case.resolution_due_at)}</span>
                                                {case.feedback_score.map(|score| view! {
                                                    <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.25rem 0.55rem;">{format!("CSAT {score}/5")}</span>
                                                })}
                                            </div>
                                        </article>
                                    }
                                />
                            </div>
                        })}
                    </section>

                    {move || feedback.get().map(|message| view! {
                        <section class="php-alert php-alert-warning">{message}</section>
                    })}

                    {move || screen.get().map(|screen| {
                        let result_count_label = format!("{} matches", screen.result_count);
                        let organization_label = screen.target_organization_id.map(|id| id.to_string()).unwrap_or_else(|| "current".into());
                        view! {
                        <section class="php-card" style="border-radius:8px;">
                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                <div>
                                    <p class="eyebrow">"Results"</p>
                                    <h2 style="margin:0;">{result_count_label}</h2>
                                </div>
                                <span style="font-size:0.9rem;color:#64748b;">{format!("Organization {organization_label}")}</span>
                            </div>
                            <div style="display:grid;gap:0.75rem;margin-top:1rem;">
                                <For
                                    each=move || screen.results.clone()
                                    key=|result| format!("{}-{}", result.category, result.id)
                                    children=move |result| {
                                        view! {
                                            <article style="border:1px solid #e5e7eb;border-radius:8px;padding:0.9rem;display:grid;gap:0.55rem;">
                                                <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;">
                                                    <div>
                                                        <span style="display:inline-flex;padding:0.2rem 0.55rem;border-radius:999px;background:#eff6ff;color:#1d4ed8;font-size:0.78rem;text-transform:uppercase;">{result.category.clone()}</span>
                                                        <h3 style="margin:0.45rem 0 0;font-size:1rem;">{result.label.clone()}</h3>
                                                        <p style="margin:0.15rem 0 0;color:#64748b;">{result.detail.clone()}</p>
                                                    </div>
                                                    {result.href.clone().map(|href| view! {
                                                        <A href=href attr:class="php-btn php-btn-light">"Open"</A>
                                                    })}
                                                    <button
                                                        type="button"
                                                        class="php-btn php-btn-light"
                                                        on:click={
                                                            let category = result.category.clone();
                                                            let id = result.id.to_string();
                                                            let organization_id = result.organization_id;
                                                            move |_| load_timeline(category.clone(), Some(id.clone()), organization_id)
                                                        }
                                                    >
                                                        "Timeline"
                                                    </button>
                                                </div>
                                                <div style="display:flex;gap:0.45rem;flex-wrap:wrap;">
                                                    <For
                                                        each=move || result.facts.clone()
                                                        key=|fact| format!("{}-{}", fact.label, fact.value)
                                                        children=move |fact| view! {
                                                            <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.25rem 0.55rem;color:#475569;font-size:0.82rem;">
                                                                <strong>{fact.label}</strong> ": " {fact.value}
                                                            </span>
                                                        }
                                                    />
                                                </div>
                                            </article>
                                        }
                                    }
                                />
                            </div>
                            <ul style="margin:1rem 0 0;color:#64748b;">
                                <For
                                    each=move || screen.notes.clone()
                                    key=|note| note.clone()
                                    children=move |note| view! { <li>{note}</li> }
                                />
                            </ul>
                        </section>
                    }})}

                    {move || timeline.get().map(|timeline| view! {
                        <section class="php-card" style="border-radius:8px;">
                            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                <div>
                                    <p class="eyebrow">"Issue Timeline"</p>
                                    <h2 style="margin:0;">{timeline.entity_type.clone()} {timeline.entity_id.clone().unwrap_or_else(|| "general".into())}</h2>
                                </div>
                                <span style="font-size:0.9rem;color:#64748b;">{format!("Organization {}", timeline.target_organization_id)}</span>
                            </div>

                            <form on:submit=submit_note style="display:grid;gap:0.75rem;margin-top:1rem;border:1px solid #e5e7eb;border-radius:8px;padding:0.9rem;">
                                <div style="display:grid;grid-template-columns:minmax(10rem,1fr) minmax(10rem,1fr) minmax(10rem,1fr);gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Ticket"</span>
                                        <input class="php-input" placeholder="SUP-1234" prop:value=move || support_ticket_ref.get() on:input=move |ev| support_ticket_ref.set(event_target_value(&ev)) />
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Visibility"</span>
                                        <select class="php-input" prop:value=move || support_visibility.get() on:change=move |ev| support_visibility.set(event_target_value(&ev))>
                                            <option value="internal">"Internal"</option>
                                            <option value="customer_visible">"Customer visible"</option>
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Entity"</span>
                                        <input class="php-input" prop:value=move || format!("{} {}", selected_entity_type.get(), selected_entity_id.get()) readonly=true />
                                    </label>
                                </div>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Note"</span>
                                    <textarea class="php-input" rows="3" placeholder="Add a support note" prop:value=move || support_note.get() on:input=move |ev| support_note.set(event_target_value(&ev))></textarea>
                                </label>
                                <button class="php-btn php-btn-primary" type="submit" disabled=move || timeline_loading.get()>
                                    {move || if timeline_loading.get() { "Saving..." } else { "Save Note" }}
                                </button>
                            </form>

                            <div style="display:grid;gap:0.7rem;margin-top:1rem;">
                                <For
                                    each=move || timeline.entries.clone()
                                    key=|entry| format!("{}-{}-{}", entry.source, entry.action, entry.created_at)
                                    children=move |entry| view! {
                                        <article style="border-left:3px solid #2563eb;padding:0.2rem 0 0.2rem 0.8rem;">
                                            <div style="display:flex;gap:0.5rem;flex-wrap:wrap;align-items:center;">
                                                <strong>{entry.action.clone()}</strong>
                                                <span style="color:#64748b;font-size:0.85rem;">{entry.source.clone()}</span>
                                                <span style="color:#64748b;font-size:0.85rem;">{entry.created_at.clone()}</span>
                                                <span style="border:1px solid #e5e7eb;border-radius:999px;padding:0.15rem 0.45rem;font-size:0.78rem;">{entry.visibility.clone()}</span>
                                            </div>
                                            <p style="margin:0.3rem 0;color:#334155;white-space:pre-wrap;">{entry.summary.clone()}</p>
                                            <small style="color:#64748b;">{entry.ticket_ref.clone().unwrap_or_else(|| "No ticket reference".into())}</small>
                                        </article>
                                    }
                                />
                            </div>
                        </section>
                    })}
                </main>
            }
            .into_any()
            }
        }}
    }
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
