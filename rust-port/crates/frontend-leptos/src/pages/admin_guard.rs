use leptos::{
    prelude::*,
    tachys::view::any_view::{AnyView, IntoAny},
};
use leptos_router::components::A;

use crate::session::{self, AuthContext};

pub fn admin_guard_view(
    auth: &AuthContext,
    title: &'static str,
    permissions: &[&str],
) -> Option<AnyView> {
    if !auth.session_ready.get() {
        Some(
            view! {
                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">
                    {format!("Loading access for {}...", title)}
                </section>
            }
            .into_any(),
        )
    } else if !auth.session.get().authenticated {
        Some(
            view! {
                <section style="display:grid;gap:0.5rem;padding:1rem;border:1px solid #fecaca;border-radius:1rem;background:#fff1f2;color:#be123c;">
                    <strong>{format!("{} requires a Rust session", title)}</strong>
                    <span>"Sign in before opening this admin route."</span>
                    <A href="/auth/login">"Open login"</A>
                </section>
            }
            .into_any(),
        )
    } else if !permissions
        .iter()
        .any(|permission| session::has_permission(auth, permission))
    {
        Some(
            view! {
                <section style="display:grid;gap:0.5rem;padding:1rem;border:1px solid #fde68a;border-radius:1rem;background:#fffbeb;color:#92400e;">
                    <strong>{format!("{} is restricted", title)}</strong>
                    <span>"The authenticated session does not have the required admin permission for this route."</span>
                    <A href="/dashboard">"Back to user dashboard"</A>
                </section>
            }
            .into_any(),
        )
    } else {
        None
    }
}
