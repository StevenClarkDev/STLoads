use leptos::prelude::*;

#[component]
pub fn AuthFrame(children: Children) -> impl IntoView {
    view! {
        <main class="auth-shell">
            <div class="auth-card">
                {children()}
            </div>
        </main>
    }
}
