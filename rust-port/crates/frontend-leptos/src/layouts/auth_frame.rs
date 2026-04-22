use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn AuthFrame(children: Children) -> impl IntoView {
    view! {
        <main class="auth-frame">
            <header>
                <div class="shell-title-row">
                    <div class="brand-mark" aria-hidden="true">"ST"</div>
                    <div class="shell-brand-copy">
                        <p class="shell-kicker">"STLoads Access"</p>
                        <h1 class="shell-title">"Secure Entry"</h1>
                    </div>
                </div>
                <A href="/" attr:class="shell-secondary">"Dashboard"</A>
            </header>
            <section>
                <h1>"Account Access"</h1>
                <p>
                    "Secure STLoads access."
                </p>
                {children()}
            </section>
        </main>
    }
}
