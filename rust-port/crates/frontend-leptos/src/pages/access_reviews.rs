use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{
    AdminAccessElevationDecisionRequest, AdminAccessReviewDecisionRequest, AdminAccessReviewScreen,
    AdminCreateAccessElevationRequest, AdminStartAccessReviewRequest,
};

use crate::{
    api,
    session::{self, use_auth},
};

use super::admin_guard_view;

#[component]
pub fn AccessReviewsPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminAccessReviewScreen>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let target_organization_id = RwSignal::new(String::new());
    let title = RwSignal::new(String::from("Quarterly privileged access review"));
    let due_days = RwSignal::new(String::from("14"));
    let elevation_target_user_id = RwSignal::new(String::new());
    let elevation_role_key = RwSignal::new(String::from("admin"));
    let elevation_reason = RwSignal::new(String::new());

    let can_manage = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_roles")
    });

    let load_screen = move || {
        loading.set(true);
        let auth = auth;
        let target_org = target_organization_id.get().trim().parse::<u64>().ok();
        spawn_local(async move {
            match api::fetch_admin_access_reviews(target_org).await {
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

    let start_review = move |ev: SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        let auth = auth;
        let payload = AdminStartAccessReviewRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            title: title.get(),
            due_days: due_days.get().trim().parse::<u64>().ok(),
        };
        spawn_local(async move {
            match api::start_admin_access_review(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
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

    let decide = move |item_id: u64, decision: &'static str| {
        loading.set(true);
        let auth = auth;
        let payload = AdminAccessReviewDecisionRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            decision: decision.into(),
            reason: Some(match decision {
                "approve" => "Approved during privileged access recertification.".into(),
                "exception" => "Exception accepted for this review period.".into(),
                "revoke" => "Privileged access revoked during recertification.".into(),
                _ => "Access review decision recorded.".into(),
            }),
        };
        spawn_local(async move {
            match api::decide_admin_access_review_item(item_id, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
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

    let request_elevation = move |ev: SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        let auth = auth;
        let payload = AdminCreateAccessElevationRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            target_user_id: elevation_target_user_id
                .get()
                .trim()
                .parse::<u64>()
                .unwrap_or_default(),
            requested_role_key: elevation_role_key.get(),
            business_justification: elevation_reason.get(),
            expires_in_days: Some(30),
        };
        spawn_local(async move {
            match api::create_admin_access_elevation_request(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
                    elevation_reason.set(String::new());
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

    let decide_elevation = move |request_id: u64, decision: &'static str| {
        loading.set(true);
        let auth = auth;
        let payload = AdminAccessElevationDecisionRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            decision: decision.into(),
            reason: Some(match decision {
                "approve" => "Privilege elevation approved after business review.".into(),
                "reject" => "Privilege elevation rejected during access review.".into(),
                _ => "Privilege elevation decision recorded.".into(),
            }),
        };
        spawn_local(async move {
            match api::decide_admin_access_elevation_request(request_id, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
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

    Effect::new(move |_| {
        if can_manage.get() && screen.get_untracked().is_none() {
            load_screen();
        }
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Access Reviews", &["access_admin_portal", "manage_roles"]) {
                guard
            } else if !can_manage.get() {
                view! { <section class="php-alert php-alert-warning">"Access reviews are restricted for this session."</section> }.into_any()
            } else {
                view! {
                    <main class="php-page-shell">
                        <section class="php-dashboard-header">
                            <div>
                                <p class="eyebrow">"Security"</p>
                                <h1>"Access Reviews"</h1>
                                <p>"Recertify privileged roles, stale accounts, exceptions, and emergency access evidence."</p>
                            </div>
                            <button type="button" class="php-btn php-btn-light" on:click=move |_| load_screen() disabled=move || loading.get()>
                                {move || if loading.get() { "Refreshing..." } else { "Refresh" }}
                            </button>
                        </section>

                        <form class="php-card php-form-grid" style="border-radius:8px;" on:submit=start_review>
                            <label>
                                <span>"Organization ID"</span>
                                <input class="php-input" inputmode="numeric" placeholder="Current org" prop:value=move || target_organization_id.get() on:input=move |ev| target_organization_id.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Review title"</span>
                                <input class="php-input" prop:value=move || title.get() on:input=move |ev| title.set(event_target_value(&ev)) />
                            </label>
                            <label>
                                <span>"Due days"</span>
                                <input class="php-input" inputmode="numeric" prop:value=move || due_days.get() on:input=move |ev| due_days.set(event_target_value(&ev)) />
                            </label>
                            <div style="display:flex;align-items:end;">
                                <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>"Start Review"</button>
                            </div>
                        </form>

                        {move || feedback.get().map(|message| view! { <section class="php-alert php-alert-info">{message}</section> })}

                        {move || screen.get().map(|screen| view! {
                            <form class="php-card php-form-grid" style="border-radius:8px;" on:submit=request_elevation>
                                <label>
                                    <span>"Target user ID"</span>
                                    <input class="php-input" inputmode="numeric" prop:value=move || elevation_target_user_id.get() on:input=move |ev| elevation_target_user_id.set(event_target_value(&ev)) />
                                </label>
                                <label>
                                    <span>"Requested role"</span>
                                    <select class="php-input" prop:value=move || elevation_role_key.get() on:change=move |ev| elevation_role_key.set(event_target_value(&ev))>
                                        <option value="admin">"Admin"</option>
                                        <option value="finance">"Finance"</option>
                                        <option value="operator">"Operator"</option>
                                        <option value="auditor">"Auditor"</option>
                                    </select>
                                </label>
                                <label>
                                    <span>"Justification"</span>
                                    <input class="php-input" prop:value=move || elevation_reason.get() on:input=move |ev| elevation_reason.set(event_target_value(&ev)) />
                                </label>
                                <div style="display:flex;align-items:end;">
                                    <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>"Request Elevation"</button>
                                </div>
                            </form>

                            <section class="php-card" style="border-radius:8px;">
                                <div class="php-section-heading">
                                    <div>
                                        <p class="eyebrow">"Evidence"</p>
                                        <h2>{screen.title.clone()}</h2>
                                    </div>
                                    <p>{screen.summary.clone()}</p>
                                </div>
                                <div class="php-grid php-grid-4">
                                    {screen.reviews.iter().map(|review| view! {
                                        <article class="php-metric-card">
                                            <span>{review.status.clone()}</span>
                                            <strong>{review.title.clone()}</strong>
                                            <small>{format!("Pending {} / Approved {} / Exceptions {} / Revoked {}", review.pending_count, review.approved_count, review.exception_count, review.revoke_count)}</small>
                                        </article>
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="php-card" style="border-radius:8px;">
                                <div class="php-section-heading">
                                    <div>
                                        <p class="eyebrow">"Approvals"</p>
                                        <h2>"Privilege Elevation Requests"</h2>
                                    </div>
                                </div>
                                <div class="php-table-wrap">
                                    <table class="php-table">
                                        <thead>
                                            <tr>
                                                <th>"Target"</th>
                                                <th>"Role"</th>
                                                <th>"Status"</th>
                                                <th>"Reason"</th>
                                                <th>"Action"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {screen.elevation_requests.iter().map(|request| {
                                                let approve_id = request.id;
                                                let reject_id = request.id;
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <strong>{request.target_email.clone()}</strong>
                                                            <span>{format!("Requested by {}", request.requester_email)}</span>
                                                        </td>
                                                        <td>{format!("{} -> {}", request.current_role_key.clone().unwrap_or_else(|| "none".into()), request.requested_role_key)}</td>
                                                        <td>{request.status.clone()}</td>
                                                        <td>{request.business_justification.clone()}</td>
                                                        <td>
                                                            <div style="display:flex;gap:0.35rem;flex-wrap:wrap;">
                                                                <button type="button" class="php-btn php-btn-light" on:click=move |_| decide_elevation(approve_id, "approve") disabled=move || loading.get()>"Approve"</button>
                                                                <button type="button" class="php-btn php-btn-danger" on:click=move |_| decide_elevation(reject_id, "reject") disabled=move || loading.get()>"Reject"</button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </section>

                            <section class="php-card" style="border-radius:8px;">
                                <div class="php-section-heading">
                                    <div>
                                        <p class="eyebrow">"Latest Review"</p>
                                        <h2>"Privileged Access Items"</h2>
                                    </div>
                                </div>
                                <div class="php-table-wrap">
                                    <table class="php-table">
                                        <thead>
                                            <tr>
                                                <th>"User"</th>
                                                <th>"Role"</th>
                                                <th>"Risk"</th>
                                                <th>"Decision"</th>
                                                <th>"Action"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {screen.items.iter().map(|item| {
                                                let approve_id = item.id;
                                                let exception_id = item.id;
                                                let revoke_id = item.id;
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <strong>{item.user_name.clone()}</strong>
                                                            <span>{item.user_email.clone()}</span>
                                                        </td>
                                                        <td>{format!("{} / {}", item.role_label, item.account_status_label)}</td>
                                                        <td>{if item.risk_flags.is_empty() { "none".into() } else { item.risk_flags.join(", ") }}</td>
                                                        <td>{item.decision.clone()}</td>
                                                        <td>
                                                            <div style="display:flex;gap:0.35rem;flex-wrap:wrap;">
                                                                <button type="button" class="php-btn php-btn-light" on:click=move |_| decide(approve_id, "approve") disabled=move || loading.get()>"Approve"</button>
                                                                <button type="button" class="php-btn php-btn-light" on:click=move |_| decide(exception_id, "exception") disabled=move || loading.get()>"Exception"</button>
                                                                <button type="button" class="php-btn php-btn-danger" on:click=move |_| decide(revoke_id, "revoke") disabled=move || loading.get()>"Revoke"</button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            </section>
                        })}
                    </main>
                }.into_any()
            }
        }}
    }
}
