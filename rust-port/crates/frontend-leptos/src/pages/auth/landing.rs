use leptos::{prelude::*, task::spawn_local};
use leptos_router::components::A;

use crate::api;

#[component]
pub fn PortalLandingPage() -> impl IntoView {
    let shipper_count = RwSignal::new("Count --".to_string());
    let carrier_count = RwSignal::new("Count --".to_string());
    let broker_count = RwSignal::new("Count --".to_string());
    let freight_forwarder_count = RwSignal::new("Count --".to_string());

    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(counts) = api::fetch_portal_role_counts().await {
                shipper_count.set(format!("Count {}", counts.shipper_total));
                carrier_count.set(format!("Count {}", counts.carrier_total));
                broker_count.set(format!("Count {}", counts.broker_total));
                freight_forwarder_count.set(format!("Count {}", counts.freight_forwarder_total));
            }
        });
    });

    view! {
        <section class="portal-home">
            <div class="portal-topbar">
                <img
                    class="portal-logo"
                    src="/assets/images/stloads/logo-bg_none-small.png"
                    alt="LoadBoard Logo"
                />
                <nav class="portal-nav">
                    <A href="/" attr:class="portal-nav-link is-active">"Home"</A>
                    <A href="https://stloads.com/about-us" attr:class="portal-nav-link">"About"</A>
                    <A href="https://stloads.com/services" attr:class="portal-nav-link">"Services"</A>
                    <A href="https://stloads.com/contact-us" attr:class="portal-nav-link">"Contact"</A>
                    <A href="/auth/login" attr:class="portal-nav-link portal-admin-link">"Customer Login"</A>
                </nav>
            </div>

            <div class="portal-heading">
                <h2 class="portal-title">"Welcome to LoadBoard - Where Smart Logistics Begin."</h2>
                <h5 class="portal-subtitle">"Select your role"</h5>
                <p class="portal-description">"To start your project we need to customize your preferences."</p>
            </div>

            <section class="portal-role-grid">
                <RoleSignupCard
                    signup_href="/auth/register?role=shipper"
                    icon_class="fas fa-boxes"
                    title="Shipper"
                    role_count=Signal::derive(move || shipper_count.get())
                    description="Get your shipper account set up"
                />
                <RoleSignupCard
                    signup_href="/auth/register?role=carrier"
                    icon_class="fas fa-truck-fast"
                    title="Carrier"
                    role_count=Signal::derive(move || carrier_count.get())
                    description="Start carrier signup"
                />
                <RoleSignupCard
                    signup_href="/auth/register?role=broker"
                    icon_class="fas fa-handshake-angle"
                    title="Broker"
                    role_count=Signal::derive(move || broker_count.get())
                    description="Start broker signup"
                />
                <RoleSignupCard
                    signup_href="/auth/register?role=freight_forwarder"
                    icon_class="fas fa-ship"
                    title="Freight Forwarder"
                    role_count=Signal::derive(move || freight_forwarder_count.get())
                    description="Start forwarder signup"
                />
            </section>
        </section>
    }
}

#[component]
fn RoleSignupCard(
    signup_href: &'static str,
    icon_class: &'static str,
    title: &'static str,
    role_count: Signal<String>,
    description: &'static str,
) -> impl IntoView {
    view! {
        <article class="portal-role-card">
            <div class="portal-role-content">
                <i class=format!("portal-role-icon {}", icon_class)></i>
                <h3 class="portal-role-title">{title}</h3>
                <p class="portal-role-count">{move || role_count.get()}</p>
                <p class="portal-role-copy">{description}</p>
                <div class="portal-role-actions">
                    <A href=signup_href attr:class="portal-role-cta">"Start Signup"</A>
                </div>
            </div>
        </article>
    }
}
