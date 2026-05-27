use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use shared::{
    AdminCheckIdentityDomainDnsRequest, AdminIdentityScreen, AdminUpsertIdentityDomainRequest,
    AdminUpsertIdentityProviderRequest, AdminVerifyIdentityDomainRequest,
};

use crate::{
    api,
    session::{self, use_auth},
};

use super::admin_guard_view;

#[component]
pub fn EnterpriseIdentityPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminIdentityScreen>);
    let feedback = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);
    let target_organization_id = RwSignal::new(String::new());
    let domain = RwSignal::new(String::new());
    let domain_token = RwSignal::new(String::new());
    let login_routing_enabled = RwSignal::new(false);
    let provider_type = RwSignal::new(String::from("oidc"));
    let provider_status = RwSignal::new(String::from("draft"));
    let provider_name = RwSignal::new(String::new());
    let issuer = RwSignal::new(String::new());
    let sso_url = RwSignal::new(String::new());
    let jwks_url = RwSignal::new(String::new());
    let metadata_url = RwSignal::new(String::new());
    let client_id = RwSignal::new(String::new());
    let default_role_key = RwSignal::new(String::from("member"));
    let jit_enabled = RwSignal::new(false);

    let can_manage = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_roles")
    });

    let load_screen = move || {
        loading.set(true);
        let auth = auth;
        let target_org = target_organization_id.get().trim().parse::<u64>().ok();
        spawn_local(async move {
            match api::fetch_admin_identity_screen(target_org).await {
                Ok(next) => {
                    screen.set(Some(next));
                    feedback.set(None);
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let submit_domain = move |ev: SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        let auth = auth;
        let payload = AdminUpsertIdentityDomainRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            domain: domain.get(),
            login_routing_enabled: login_routing_enabled.get(),
        };
        spawn_local(async move {
            match api::upsert_admin_identity_domain(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
                    domain.set(String::new());
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let verify_domain = move |ev: SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        let auth = auth;
        let payload = AdminVerifyIdentityDomainRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            domain: domain.get(),
            verification_token: domain_token.get(),
        };
        spawn_local(async move {
            match api::verify_admin_identity_domain(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
                    domain_token.set(String::new());
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let verify_domain_dns = move |ev: SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        let auth = auth;
        let payload = AdminCheckIdentityDomainDnsRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            domain: domain.get(),
        };
        spawn_local(async move {
            match api::check_admin_identity_domain_dns(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    let submit_provider = move |ev: SubmitEvent| {
        ev.prevent_default();
        loading.set(true);
        let auth = auth;
        let payload = AdminUpsertIdentityProviderRequest {
            target_organization_id: target_organization_id.get().trim().parse::<u64>().ok(),
            provider_id: None,
            provider_type: provider_type.get(),
            status: provider_status.get(),
            display_name: provider_name.get(),
            issuer: optional_string(issuer.get()),
            sso_url: optional_string(sso_url.get()),
            jwks_url: optional_string(jwks_url.get()),
            metadata_url: optional_string(metadata_url.get()),
            client_id: optional_string(client_id.get()),
            jit_enabled: jit_enabled.get(),
            default_role_key: default_role_key.get(),
        };
        spawn_local(async move {
            match api::upsert_admin_identity_provider(&payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    screen.set(Some(response.screen));
                }
                Err(error) => {
                    if error.contains("returned 401") {
                        session::invalidate_session(
                            &auth,
                            "Your Rust session expired; sign in again.",
                        );
                    }
                    feedback.set(Some(error));
                }
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| {
        if can_manage.get() && screen.get_untracked().is_none() {
            load_screen();
        }
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Enterprise Identity", &["access_admin_portal", "manage_roles"]) {
                guard
            } else if !can_manage.get() {
                view! { <section class="php-alert php-alert-warning">"Enterprise identity is restricted for this session."</section> }.into_any()
            } else {
                view! {
                    <main class="php-page-shell">
                        <section class="php-dashboard-header">
                            <div>
                                <p class="eyebrow">"Security"</p>
                                <h1>"Enterprise Identity"</h1>
                                <p>"Manage domain verification, SSO routing metadata, and SCIM deprovisioning evidence."</p>
                            </div>
                            <button type="button" class="php-btn php-btn-light" on:click=move |_| load_screen() disabled=move || loading.get()>
                                {move || if loading.get() { "Refreshing..." } else { "Refresh" }}
                            </button>
                        </section>

                        <section class="php-card" style="border-radius:8px;">
                            <label style="display:grid;gap:0.35rem;max-width:18rem;">
                                <span>"Organization ID"</span>
                                <input class="php-input" inputmode="numeric" placeholder="Current org" prop:value=move || target_organization_id.get() on:input=move |ev| target_organization_id.set(event_target_value(&ev)) />
                            </label>
                        </section>

                        {move || feedback.get().map(|message| view! {
                            <section class="php-alert php-alert-warning">{message}</section>
                        })}

                        <section class="php-card" style="border-radius:8px;">
                            <p class="eyebrow">"Domain Verification"</p>
                            <form on:submit=submit_domain style="display:grid;grid-template-columns:minmax(14rem,1fr) auto auto;gap:0.75rem;align-items:end;">
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Domain"</span>
                                    <input class="php-input" placeholder="customer.com" prop:value=move || domain.get() on:input=move |ev| domain.set(event_target_value(&ev)) />
                                </label>
                                <label style="display:flex;gap:0.45rem;align-items:center;">
                                    <input type="checkbox" prop:checked=move || login_routing_enabled.get() on:change=move |ev| login_routing_enabled.set(event_target_checked(&ev)) />
                                    <span>"Enforce routing"</span>
                                </label>
                                <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>"Save Domain"</button>
                            </form>
                            <form on:submit=verify_domain style="display:grid;grid-template-columns:minmax(14rem,1fr) minmax(18rem,1fr) auto;gap:0.75rem;align-items:end;margin-top:0.8rem;">
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Domain"</span>
                                    <input class="php-input" placeholder="customer.com" prop:value=move || domain.get() on:input=move |ev| domain.set(event_target_value(&ev)) />
                                </label>
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"Verification Token"</span>
                                    <input class="php-input" placeholder="stloads-domain-..." prop:value=move || domain_token.get() on:input=move |ev| domain_token.set(event_target_value(&ev)) />
                                </label>
                                <button class="php-btn php-btn-light" type="submit" disabled=move || loading.get()>"Verify"</button>
                            </form>
                            <form on:submit=verify_domain_dns style="display:grid;grid-template-columns:minmax(14rem,1fr) auto;gap:0.75rem;align-items:end;margin-top:0.8rem;">
                                <label style="display:grid;gap:0.35rem;">
                                    <span>"DNS Domain"</span>
                                    <input class="php-input" placeholder="customer.com" prop:value=move || domain.get() on:input=move |ev| domain.set(event_target_value(&ev)) />
                                </label>
                                <button class="php-btn php-btn-light" type="submit" disabled=move || loading.get()>"Check DNS TXT"</button>
                            </form>
                        </section>

                        <section class="php-card" style="border-radius:8px;">
                            <p class="eyebrow">"Provider Metadata"</p>
                            <form on:submit=submit_provider style="display:grid;gap:0.75rem;">
                                <div style="display:grid;grid-template-columns:repeat(4,minmax(10rem,1fr));gap:0.75rem;">
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Type"</span>
                                        <select class="php-input" prop:value=move || provider_type.get() on:change=move |ev| provider_type.set(event_target_value(&ev))>
                                            <option value="oidc">"OIDC"</option>
                                            <option value="saml">"SAML"</option>
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Status"</span>
                                        <select class="php-input" prop:value=move || provider_status.get() on:change=move |ev| provider_status.set(event_target_value(&ev))>
                                            <option value="draft">"Draft"</option>
                                            <option value="active">"Active"</option>
                                            <option value="disabled">"Disabled"</option>
                                        </select>
                                    </label>
                                    <label style="display:grid;gap:0.35rem;">
                                        <span>"Default Role"</span>
                                        <input class="php-input" prop:value=move || default_role_key.get() on:input=move |ev| default_role_key.set(event_target_value(&ev)) />
                                    </label>
                                    <label style="display:flex;gap:0.45rem;align-items:center;">
                                        <input type="checkbox" prop:checked=move || jit_enabled.get() on:change=move |ev| jit_enabled.set(event_target_checked(&ev)) />
                                        <span>"JIT users"</span>
                                    </label>
                                </div>
                                <div style="display:grid;grid-template-columns:repeat(2,minmax(14rem,1fr));gap:0.75rem;">
                                    <input class="php-input" placeholder="Display name" prop:value=move || provider_name.get() on:input=move |ev| provider_name.set(event_target_value(&ev)) />
                                    <input class="php-input" placeholder="Issuer" prop:value=move || issuer.get() on:input=move |ev| issuer.set(event_target_value(&ev)) />
                                    <input class="php-input" placeholder="SSO URL" prop:value=move || sso_url.get() on:input=move |ev| sso_url.set(event_target_value(&ev)) />
                                    <input class="php-input" placeholder="JWKS URL" prop:value=move || jwks_url.get() on:input=move |ev| jwks_url.set(event_target_value(&ev)) />
                                    <input class="php-input" placeholder="Metadata URL" prop:value=move || metadata_url.get() on:input=move |ev| metadata_url.set(event_target_value(&ev)) />
                                    <input class="php-input" placeholder="Client ID" prop:value=move || client_id.get() on:input=move |ev| client_id.set(event_target_value(&ev)) />
                                </div>
                                <button class="php-btn php-btn-primary" type="submit" disabled=move || loading.get()>"Save Provider"</button>
                            </form>
                        </section>

                        {move || screen.get().map(|screen| view! {
                            <>
                                <section class="php-card" style="border-radius:8px;">
                                    <p class="eyebrow">"Domains"</p>
                                    <div style="display:grid;gap:0.65rem;">
                                        <For each=move || screen.domains.clone() key=|row| row.id children=move |row| view! {
                                            <article style="border:1px solid #e5e7eb;border-radius:8px;padding:0.8rem;">
                                                <strong>{row.domain}</strong>
                                                <p style="margin:0.25rem 0;color:#64748b;">{format!("{} / routing {}", row.verification_status, if row.login_routing_enabled { "enabled" } else { "disabled" })}</p>
                                                <small style="color:#64748b;">{row.verification_token}</small>
                                            </article>
                                        } />
                                    </div>
                                </section>
                                <section class="php-card" style="border-radius:8px;">
                                    <p class="eyebrow">"Providers"</p>
                                    <div style="display:grid;gap:0.65rem;">
                                        <For each=move || screen.providers.clone() key=|row| row.id children=move |row| view! {
                                            <article style="border:1px solid #e5e7eb;border-radius:8px;padding:0.8rem;">
                                                <strong>{row.display_name}</strong>
                                                <p style="margin:0.25rem 0;color:#64748b;">{format!("{} / {} / JIT {}", row.provider_type, row.status, if row.jit_enabled { "enabled" } else { "disabled" })}</p>
                                                <small style="color:#64748b;">{row.sso_url.unwrap_or_else(|| "No SSO URL".into())}</small>
                                            </article>
                                        } />
                                    </div>
                                </section>
                                <section class="php-card" style="border-radius:8px;">
                                    <p class="eyebrow">"SCIM Events"</p>
                                    <div style="display:grid;gap:0.65rem;">
                                        <For each=move || screen.scim_events.clone() key=|row| row.id children=move |row| view! {
                                            <article style="border:1px solid #e5e7eb;border-radius:8px;padding:0.8rem;">
                                                <strong>{row.action}</strong>
                                                <p style="margin:0.25rem 0;color:#64748b;">{format!("{} / {}", row.outcome, row.created_at)}</p>
                                                <small style="color:#64748b;">{row.reason.unwrap_or_else(|| "No reason".into())}</small>
                                            </article>
                                        } />
                                    </div>
                                </section>
                            </>
                        })}
                    </main>
                }.into_any()
            }
        }}
    }
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
