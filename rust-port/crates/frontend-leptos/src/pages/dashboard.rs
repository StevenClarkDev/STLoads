use leptos::prelude::*;
use leptos_router::components::A;

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
        _ => "background:#e5e7eb;padding:0.25rem 0.6rem;border-radius:999px;color:#111827;",
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let destinations = vec![
        ("Load Board", "/loads", "Dispatch and booking workspace"),
        ("Private Chat", "/chat", "Conversation and offer workflow"),
        (
            "STLOADS Ops",
            "/admin/stloads",
            "Publish, retry, and sync monitoring",
        ),
        (
            "Reconciliation",
            "/admin/stloads/reconciliation",
            "Mismatch cleanup and audit log",
        ),
    ];

    view! {
        <article style="display:grid;gap:1.25rem;">
            <section>
                <p style=tone_style("info")>"Rust + Leptos Workspace"</p>
                <h2>"Operational Port Progress"</h2>
                <p>
                    "The dashboard is now a navigation hub for the first real screen ports instead of a single placeholder."
                </p>
            </section>
            <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:1rem;">
                {destinations
                    .into_iter()
                    .map(|(label, href, detail)| {
                        view! {
                            <A href=href attr:style="display:block;padding:1rem;border:1px solid #dbeafe;border-radius:1rem;text-decoration:none;color:inherit;background:#f8fbff;">
                                <strong>{label}</strong>
                                <p style="margin:0.5rem 0 0;">{detail}</p>
                            </A>
                        }
                    })
                    .collect_view()}
            </section>
        </article>
    }
}
