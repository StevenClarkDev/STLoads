use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <article>
            <h2>"Route Not Found"</h2>
            <p>"The frontend scaffold currently exposes dashboard, loads, auth/login, and admin."</p>
            <p>
                <A href="/">"Return to dashboard"</A>
            </p>
        </article>
    }
}
