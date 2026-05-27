use leptos::prelude::*;

#[component]
pub fn AuthFrame(children: Children) -> impl IntoView {
    view! {
        <main class="auth-shell">
            <a class="skip-link" href="#main-content">"Skip to main content"</a>
            <div id="main-content" class="auth-card" tabindex="-1">
                {children()}
            </div>
        </main>
    }
}
