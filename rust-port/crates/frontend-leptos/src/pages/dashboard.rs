use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let destinations = vec![
        (
            "Market",
            "Load Board",
            "/loads",
            "Dispatch and booking.",
            "accent-teal",
        ),
        (
            "Desk",
            "Private Chat",
            "/chat",
            "Offers and messages.",
            "accent-gold",
        ),
        (
            "Sync",
            "STLOADS Ops",
            "/admin/stloads",
            "Sync and retries.",
            "accent-copper",
        ),
        (
            "Audit",
            "Reconciliation",
            "/admin/stloads/reconciliation",
            "Exceptions and audit.",
            "accent-coral",
        ),
        (
            "Account",
            "My Profile",
            "/profile",
            "Company and credentials.",
            "accent-violet",
        ),
    ];

    view! {
        <article class="dashboard-page">
            <section class="dashboard-hero">
                <div class="hero-copy">
                    <p class="eyebrow">"Rust + Leptos Workspace"</p>
                    <h2>"Freight Command Center"</h2>
                    <p>
                        "Live freight operations."
                    </p>
                    <div class="hero-actions">
                        <A href="/loads" attr:class="shell-cta">"Open Load Board"</A>
                        <A href="/admin/stloads/operations" attr:class="shell-secondary">"Monitor STLOADS"</A>
                    </div>
                </div>
                <div class="route-visual" aria-label="Lane telemetry snapshot">
                    <div class="route-map">
                        <div class="route-node origin">
                            <strong>"DAL"</strong>
                            <small>"pickup"</small>
                        </div>
                        <div class="route-node crossdock">
                            <strong>"MEM"</strong>
                            <small>"handoff"</small>
                        </div>
                        <div class="route-node dest">
                            <strong>"ATL"</strong>
                            <small>"delivery"</small>
                        </div>
                        <div class="route-node audit">
                            <strong>"QA"</strong>
                            <small>"audit"</small>
                        </div>
                    </div>
                    <div class="signal-strip">
                        <div class="signal-card">
                            <strong>"45"</strong>
                            <small>"tracked tables"</small>
                        </div>
                        <div class="signal-card">
                            <strong>"9"</strong>
                            <small>"route groups"</small>
                        </div>
                        <div class="signal-card">
                            <strong>"Rust"</strong>
                            <small>"API surface"</small>
                        </div>
                    </div>
                </div>
            </section>

            <section class="command-grid" aria-label="Workspace destinations">
                {destinations
                    .into_iter()
                    .map(|(kicker, label, href, detail, accent)| {
                        view! {
                            <A href=href attr:class=format!("command-card {}", accent)>
                                <span class="command-card-header">
                                    <span class="command-card-kicker">{kicker}</span>
                                    <strong>{label}</strong>
                                </span>
                                <p>{detail}</p>
                            </A>
                        }
                    })
                    .collect_view()}
            </section>

        </article>
    }
}
