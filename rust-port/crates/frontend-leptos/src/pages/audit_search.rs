use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{AdminAuditExportResponse, AdminAuditSearchFilters, AdminAuditSearchScreen};

use crate::{
    api,
    session::{self, use_auth},
};

use super::admin_guard_view;

#[component]
pub fn AuditSearchPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminAuditSearchScreen>);
    let export_result = RwSignal::new(None::<AdminAuditExportResponse>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let exporting = RwSignal::new(false);

    let q = RwSignal::new(String::new());
    let target_organization_id = RwSignal::new(String::new());
    let actor_user_id = RwSignal::new(String::new());
    let entity_type = RwSignal::new(String::new());
    let entity_id = RwSignal::new(String::new());
    let action = RwSignal::new(String::new());
    let request_id = RwSignal::new(String::new());
    let date_from = RwSignal::new(String::new());
    let date_to = RwSignal::new(String::new());

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "view_audit_events")
            || session::has_permission(&auth, "manage_users")
    });

    let collect_filters = move || AdminAuditSearchFilters {
        q: optional_string(q.get()),
        target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
        actor_user_id: actor_user_id.get().trim().parse::<u64>().ok(),
        entity_type: optional_string(entity_type.get()),
        entity_id: optional_string(entity_id.get()),
        action: optional_string(action.get()),
        request_id: optional_string(request_id.get()),
        date_from: optional_string(date_from.get()),
        date_to: optional_string(date_to.get()),
    };

    let search = move || {
        loading.set(true);
        let auth = auth;
        let filters = collect_filters();
        spawn_local(async move {
            match api::fetch_admin_audit_search(&filters).await {
                Ok(next) => {
                    screen.set(Some(next));
                    export_result.set(None);
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

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        search();
    };

    let export_csv = move |_| {
        exporting.set(true);
        let auth = auth;
        let filters = collect_filters();
        spawn_local(async move {
            match api::export_admin_audit_search(&filters).await {
                Ok(response) => {
                    feedback.set(Some(format!(
                        "Prepared {} with {} rows.",
                        response.filename, response.row_count
                    )));
                    export_result.set(Some(response));
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
            exporting.set(false);
        });
    };

    Effect::new(move |_| {
        if can_view.get() && screen.get_untracked().is_none() {
            search();
        }
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Audit Search", &["access_admin_portal", "view_audit_events", "manage_users"]) {
                guard
            } else if !can_view.get() {
                view! { <section class="php-alert php-alert-warning">"Audit search is restricted for this session."</section> }.into_any()
            } else {
                view! {
                    <main class="php-page-shell">
                        <section class="php-dashboard-header">
                            <div>
                                <p class="eyebrow">"Compliance"</p>
                                <h1>"Audit Search"</h1>
                                <p>"Find who changed what across users, freight, documents, payments, TMS handoffs, and admin workflows."</p>
                            </div>
                            <button type="button" class="php-btn php-btn-light" on:click=move |_| search() disabled=move || loading.get()>
                                {move || if loading.get() { "Refreshing..." } else { "Refresh" }}
                            </button>
                        </section>

                        <form class="php-card php-form-grid" style="border-radius:8px;" on:submit=submit>
                            <label>
                                <span>"Search"</span>
                                <input class="php-input" type="search" placeholder="Email, request ID, reason, metadata" prop:value=move || q.get() on:input=move |ev| q.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Organization ID"</span>
                                <input class="php-input" inputmode="numeric" placeholder="Current org" prop:value=move || target_organization_id.get() on:input=move |ev| target_organization_id.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Actor user ID"</span>
                                <input class="php-input" inputmode="numeric" placeholder="Any actor" prop:value=move || actor_user_id.get() on:input=move |ev| actor_user_id.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Entity type"</span>
                                <select class="php-input" prop:value=move || entity_type.get() on:change=move |ev| entity_type.set(event_target_value(&ev))>
                                    <option value="">"Any entity"</option>
                                    <option value="user">"User"</option>
                                    <option value="organization">"Organization"</option>
                                    <option value="load">"Load"</option>
                                    <option value="document">"Document"</option>
                                    <option value="payment">"Payment"</option>
                                    <option value="tms_handoff">"TMS handoff"</option>
                                    <option value="support_search">"Support search"</option>
                                </select>
                            </label>
                            <label>
                                <span>"Entity ID"</span>
                                <input class="php-input" placeholder="Load, document, payment, handoff" prop:value=move || entity_id.get() on:input=move |ev| entity_id.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Action"</span>
                                <input class="php-input" placeholder="approved, revoked, webhook" prop:value=move || action.get() on:input=move |ev| action.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Request ID"</span>
                                <input class="php-input" placeholder="req_..." prop:value=move || request_id.get() on:input=move |ev| request_id.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"From"</span>
                                <input class="php-input" type="date" prop:value=move || date_from.get() on:input=move |ev| date_from.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"To"</span>
                                <input class="php-input" type="date" prop:value=move || date_to.get() on:input=move |ev| date_to.set(event_target_value(&ev)) />
                            </label>
                            <div style="display:flex;gap:0.5rem;align-items:end;flex-wrap:wrap;">
                                <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>
                                    {move || if loading.get() { "Searching..." } else { "Search" }}
                                </button>
                                <button class="php-btn php-btn-light" type="button" on:click=export_csv disabled=move || exporting.get()>
                                    {move || if exporting.get() { "Exporting..." } else { "Export CSV" }}
                                </button>
                            </div>
                        </form>

                        {move || feedback.get().map(|message| view! {
                            <section class="php-alert php-alert-warning">{message}</section>
                        })}

                        {move || export_result.get().map(|response| view! {
                            <section class="php-card" style="border-radius:8px;">
                                <p class="eyebrow">"Compliance Export"</p>
                                <h2 style="margin:0;">{response.filename}</h2>
                                <p style="color:#64748b;">{format!("{} rows prepared as {}.", response.row_count, response.content_type)}</p>
                                <textarea class="php-input" rows="8" readonly prop:value=response.csv></textarea>
                            </section>
                        })}

                        {move || screen.get().map(|screen| {
                            view! {
                                <section class="php-card" style="border-radius:8px;overflow:auto;">
                                    <div style="display:flex;justify-content:space-between;gap:1rem;align-items:center;flex-wrap:wrap;">
                                        <div>
                                            <p class="eyebrow">"Results"</p>
                                            <h2 style="margin:0;">{format!("{} audit events", screen.result_count)}</h2>
                                        </div>
                                        <span style="color:#64748b;font-size:0.9rem;">{format!("Organization {}", screen.target_organization_id)}</span>
                                    </div>
                                    <table class="php-table" style="margin-top:1rem;min-width:980px;">
                                        <thead>
                                            <tr>
                                                <th>"Time"</th>
                                                <th>"Actor"</th>
                                                <th>"Action"</th>
                                                <th>"Entity"</th>
                                                <th>"Request"</th>
                                                <th>"Evidence"</th>
                                                <th>"Reason"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || screen.rows.clone()
                                                key=|row| row.id
                                                children=move |row| {
                                                    let entity = format!(
                                                        "{} {}",
                                                        row.entity_type,
                                                        row.entity_id.clone().unwrap_or_else(|| "-".into())
                                                    );
                                                    view! {
                                                        <tr>
                                                            <td>{row.created_at}</td>
                                                            <td>{row.actor_label}</td>
                                                            <td>{row.action}</td>
                                                            <td>{entity}</td>
                                                            <td>{row.request_id.unwrap_or_else(|| "-".into())}</td>
                                                            <td>{row.before_after_label}</td>
                                                            <td>{row.reason.unwrap_or_else(|| row.metadata_preview.unwrap_or_else(|| "-".into()))}</td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
                                    {screen.notes.into_iter().map(|note| view! {
                                        <p style="margin:0.75rem 0 0;color:#64748b;font-size:0.88rem;">{note}</p>
                                    }).collect_view()}
                                </section>
                            }
                        })}
                    </main>
                }.into_any()
            }
        }}
    }
}

fn optional_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}
