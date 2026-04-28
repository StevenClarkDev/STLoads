use leptos::prelude::*;
use leptos_router::components::A;

use crate::session::use_auth;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let auth = use_auth();
    let today = "Thursday, 23 Apr 2026";
    // Keep dashboard-only admin surfaces aligned with the user shell so non-admin
    // roles do not see STLOADS operations or reconciliation shortcuts.
    let show_admin_surfaces =
        Signal::derive(move || crate::session::has_permission(&auth, "access_admin_portal"));

    view! {
        <article class="php-grid">
            <section class="php-page-title">
                <div>
                    <h4>"Dashboard"</h4>
                    <p>"Overview"</p>
                </div>
                <ol class="php-breadcrumb">
                    <li><i class="fas fa-home"></i></li>
                    <li>"Dashboard"</li>
                    <li><strong>"Overview"</strong></li>
                </ol>
            </section>

            <section class="php-grid columns-3">
                <div class="php-card php-welcome-card">
                    <div class="php-card-body">
                        <div class="php-welcome-copy">
                            <h4>{move || format!("Welcome Back, {}!", auth.session.get().user.as_ref().map(|user| user.name.clone()).unwrap_or_else(|| "User".into()))}</h4>
                            <p>"Here's what's happening in your account today"</p>
                        </div>
                        <div class="php-clockbox">
                            <i class="fas fa-clock" style="font-size:2.4rem;"></i>
                            <span class="date">{today}</span>
                        </div>
                    </div>
                </div>

                <div class="php-card php-widget-card">
                    <div class="php-card-body">
                        <span class="label">"Active Views"</span>
                        <span class="value">
                            {move || if show_admin_surfaces.get() { "4" } else { "2" }}
                        </span>
                        <span class="sub">"Open workspaces"</span>
                        <div class="icon php-icon-primary"><i class="fas fa-layer-group"></i></div>
                    </div>
                </div>

                <div class="php-grid">
                    <div class="php-card php-widget-card">
                        <div class="php-card-body">
                            <span class="label">"Messages"</span>
                            <span class="value">"Live"</span>
                            <span class="sub">"Chat"</span>
                            <div class="icon php-icon-success"><i class="fas fa-comments"></i></div>
                        </div>
                    </div>
                    <div class="php-card php-widget-card">
                        <div class="php-card-body">
                            <span class="label">"Profile"</span>
                            <span class="value">"Open"</span>
                            <span class="sub">"Account"</span>
                            <div class="icon php-icon-warning"><i class="fas fa-id-badge"></i></div>
                        </div>
                    </div>
                </div>
            </section>

            {move || show_admin_surfaces.get().then(|| view! {
                <section class="php-card">
                    <div class="php-card-body php-grid">
                        <div style="display:flex;justify-content:space-between;align-items:center;gap:1rem;flex-wrap:wrap;">
                            <h5 style="margin:0;">"STLOADS Board Status"</h5>
                            <A href="/admin/stloads" attr:class="shell-action secondary">"View All"</A>
                        </div>
                        <div class="php-stloads-grid">
                            <div class="php-mini-status">
                                <i class="fas fa-clock" style="color:#f59e0b;"></i>
                                <div>
                                    <div class="count">"1"</div>
                                    <div class="label">"Queued"</div>
                                </div>
                            </div>
                            <div class="php-mini-status">
                                <i class="fas fa-check-circle" style="color:#22c55e;"></i>
                                <div>
                                    <div class="count">"2"</div>
                                    <div class="label">"Published"</div>
                                </div>
                            </div>
                            <div class="php-mini-status">
                                <i class="fas fa-triangle-exclamation" style="color:#ef4444;"></i>
                                <div>
                                    <div class="count">"0"</div>
                                    <div class="label">"Failed"</div>
                                </div>
                            </div>
                            <div class="php-mini-status">
                                <i class="fas fa-times-circle" style="color:#64748b;"></i>
                                <div>
                                    <div class="count">"0"</div>
                                    <div class="label">"Withdrawn"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            })}

            <section class="php-grid columns-2">
                <div class="php-card">
                    <div class="php-card-body">
                        <h5 style="margin:0 0 1rem;">"Recent Activity"</h5>
                        <ul class="php-list">
                            {move || {
                                let mut items = vec![
                                    ("Load workspace", "Available"),
                                    ("Profile workspace", "Available"),
                                ];
                                if show_admin_surfaces.get() {
                                    items.push(("STLOADS routes", "Available"));
                                } else {
                                    items.push(("Workspace scope", "Role-safe"));
                                }
                                items
                            }
                                .into_iter()
                                .map(|(title, copy)| view! {
                                    <li class="php-list-item">
                                        <i class="fas fa-circle-check" style="color:#7366ff;margin-top:0.15rem;"></i>
                                        <div class="copy">
                                            <h6>{title}</h6>
                                            <span>{copy}</span>
                                        </div>
                                    </li>
                                })
                                .collect_view()}
                        </ul>
                    </div>
                </div>

                <div class="php-card">
                    <div class="php-card-body">
                        <h5 style="margin:0 0 1rem;">"Today's Workspaces"</h5>
                        <div class="php-grid">
                            {move || {
                                let mut items = vec![
                                    (
                                        "Load Board",
                                        "/loads",
                                        "Loads and booking",
                                    ),
                                    (
                                        "Private Chat",
                                        "/chat",
                                        "Messages",
                                    ),
                                ];
                                if show_admin_surfaces.get() {
                                    items.push((
                                        "STLOADS Ops",
                                        "/admin/stloads",
                                        "Operations",
                                    ));
                                    items.push((
                                        "Reconciliation",
                                        "/admin/stloads/reconciliation",
                                        "Audit",
                                    ));
                                }
                                items
                            }
                                .into_iter()
                                .map(|(label, href, copy)| view! {
                                    <A href=href attr:class="dashboard-card">
                                        <span class="status-pill secondary">"Open"</span>
                                        <strong class="dashboard-card-title">{label}</strong>
                                        <p class="dashboard-card-copy">{copy}</p>
                                    </A>
                                })
                                .collect_view()}
                        </div>
                    </div>
                </div>
            </section>
        </article>
    }
}
