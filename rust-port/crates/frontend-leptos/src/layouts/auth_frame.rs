use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn AuthFrame(children: Children) -> impl IntoView {
    view! {
        <main class="auth-frame">
            <header>
                <p>"STLoads Auth"</p>
                <A href="/">"Back to dashboard shell"</A>
            </header>
            <section>
                <h1>"Auth Shell"</h1>
                <p>
                    "This frame will host login, OTP, register, forgot-password, and reset flows."
                </p>
                {children()}
            </section>
        </main>
    }
}
