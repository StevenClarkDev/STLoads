use leptos::{prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};
use leptos_router::{components::A, hooks::use_params_map};

use crate::{
    api, document_upload,
    session::{self, use_auth},
};
use shared::{AdminUserDirectoryScreen, AdminUserDirectoryUser, AdminUserProfileScreen};

use super::admin_guard_view;

#[component]
pub fn AdminUsersByRolePage() -> impl IntoView {
    let auth = use_auth();
    let params = use_params_map();
    let role_key = Memo::new(move |_| params.with(|map| map.get("role_key")));

    let screen = RwSignal::new(None::<AdminUserDirectoryScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let search_query = RwSignal::new(String::new());
    let show_all_statuses = RwSignal::new(false);
    let selected_profile = RwSignal::new(None::<AdminUserProfileScreen>);
    let profile_loading = RwSignal::new(false);
    let active_profile_user_id = RwSignal::new(None::<u64>);
    let refresh_nonce = RwSignal::new(0_u64);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_users")
    });

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();

        if !auth.session_ready.get() || !auth.session.get().authenticated || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_admin_user_directory().await {
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
    });

    view! {
        {move || {
            if let Some(guard) = admin_guard_view(&auth, "Users By Role", &["access_admin_portal", "manage_users"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1rem;">
                        <section style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                            <div style="display:grid;gap:0.3rem;">
                                <h2>{move || role_heading(screen.get(), role_key.get())}</h2>
                                <p>{move || role_subtitle(role_key.get(), show_all_statuses.get())}</p>
                            </div>
                            <div style="display:flex;gap:0.6rem;flex-wrap:wrap;align-items:center;">
                                <A href="/admin/users" attr:style="padding:0.6rem 0.85rem;border-radius:0.8rem;background:#f4f4f5;color:#111827;text-decoration:none;">
                                    "Open full directory"
                                </A>
                                <label style="display:flex;gap:0.45rem;align-items:center;padding:0.6rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.8rem;background:#fff;">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || show_all_statuses.get()
                                        on:change=move |ev| show_all_statuses.set(event_target_checked(&ev))
                                    />
                                    <span>"Show all statuses"</span>
                                </label>
                            </div>
                        </section>

                        <section style="display:grid;grid-template-columns:minmax(240px,360px) minmax(0,1fr);gap:1rem;align-items:start;">
                            <div style="display:grid;gap:0.75rem;">
                                <input
                                    type="text"
                                    placeholder="Search by name, email, company, or phone"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                    style="padding:0.75rem 0.85rem;border:1px solid #d6d3d1;border-radius:0.9rem;"
                                />

                            </div>

                            <div style="display:grid;gap:1rem;">
                                {move || if profile_loading.get() {
                                    view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading role profile..."</section> }.into_any()
                                } else if let Some(profile) = selected_profile.get() {
                                    render_role_profile_panel(profile, feedback).into_any()
                                } else {
                                    view! { <></> }.into_any()
                                }}

                                {move || {
                                    if loading.get() && screen.get().is_none() {
                                        view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"Loading role-filtered users from the Rust backend..."</section> }.into_any()
                                    } else if let Some(screen_data) = screen.get() {
                                        let current_role_key = role_key.get().unwrap_or_else(|| "carrier".into());
                                        let query = search_query.get().to_ascii_lowercase();
                                        let show_all = show_all_statuses.get();
                                        let users = screen_data
                                            .users
                                            .into_iter()
                                            .filter(|user| user.role_key == current_role_key)
                                            .filter(|user| show_all || user.status_key == "approved")
                                            .filter(|user| role_user_matches_query(user, &query))
                                            .collect::<Vec<_>>();

                                        if users.is_empty() {
                                            view! {
                                                <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fff;">
                                                    "No users match this role filter yet."
                                                </section>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <section style="display:grid;gap:0.85rem;">
                                                    {users.into_iter().map(|user| {
                                                        render_role_user_card(
                                                            user,
                                                            feedback,
                                                            selected_profile,
                                                            profile_loading,
                                                            active_profile_user_id,
                                                        )
                                                    }).collect_view()}
                                                </section>
                                            }.into_any()
                                        }
                                    } else {
                                        view! { <section style="padding:1rem;border:1px solid #e5e7eb;border-radius:1rem;background:#fafaf9;">"No role-filtered directory data is available yet."</section> }.into_any()
                                    }
                                }}
                            </div>
                        </section>
                    </article>
                }.into_any()
            }
        }}
    }
}

fn render_role_user_card(
    user: AdminUserDirectoryUser,
    feedback: RwSignal<Option<String>>,
    selected_profile: RwSignal<Option<AdminUserProfileScreen>>,
    profile_loading: RwSignal<bool>,
    active_profile_user_id: RwSignal<Option<u64>>,
) -> impl IntoView {
    let user_id = user.user_id;
    view! {
        <article style="padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fff;display:grid;gap:0.85rem;">
            <div style="display:flex;justify-content:space-between;gap:1rem;align-items:flex-start;flex-wrap:wrap;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>{user.name.clone()}</strong>
                    <span>{user.email.clone()}</span>
                    <small>{format!("{} | Joined {} | {} KYC document(s)", user.status_label, user.joined_at_label, user.document_count)}</small>
                    {user.company_name.clone().map(|value| view! { <small>{format!("Company: {}", value)}</small> })}
                    {user.phone_no.clone().map(|value| view! { <small>{format!("Phone: {}", value)}</small> })}
                </div>
                <div style="display:flex;gap:0.5rem;flex-wrap:wrap;align-items:center;">
                    <button
                        type="button"
                        style="padding:0.55rem 0.8rem;border:none;border-radius:0.8rem;background:#1d4ed8;color:white;cursor:pointer;"
                        disabled=move || active_profile_user_id.get() == Some(user_id)
                        on:click=move |_| {
                            active_profile_user_id.set(Some(user_id));
                            profile_loading.set(true);
                            spawn_local(async move {
                                match api::fetch_admin_user_profile(user_id).await {
                                    Ok(profile) => selected_profile.set(Some(profile)),
                                    Err(error) => feedback.set(Some(error)),
                                }
                                active_profile_user_id.set(None);
                                profile_loading.set(false);
                            });
                        }
                    >
                        {move || if active_profile_user_id.get() == Some(user_id) { "Loading..." } else { "Profile" }}
                    </button>
                    <A href="/admin/users" attr:style="padding:0.55rem 0.8rem;border-radius:0.8rem;background:#f8fafc;color:#111827;text-decoration:none;border:1px solid #d1d5db;">
                        "Manage"
                    </A>
                    <a href=format!("mailto:{}", user.email) style="padding:0.55rem 0.8rem;border-radius:0.8rem;background:#eef2ff;color:#312e81;text-decoration:none;">
                        "Email"
                    </a>
                    {user.phone_no.clone().map(|phone| view! {
                        <a href=format!("tel:{}", phone) style="padding:0.55rem 0.8rem;border-radius:0.8rem;background:#ecfeff;color:#155e75;text-decoration:none;">
                            "Call"
                        </a>
                    })}
                </div>
            </div>
        </article>
    }
}

fn render_role_profile_panel(
    profile: AdminUserProfileScreen,
    feedback: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section style="padding:1rem;border:1px solid #cbd5e1;border-radius:1rem;background:#f8fafc;display:grid;gap:0.75rem;">
            <div>
                <h3 style="margin:0;">{profile.name.clone()}</h3>
                <p style="margin:0.2rem 0;">{format!("{} | {} | Joined {}", profile.email, profile.role_label, profile.joined_at_label)}</p>
                <small>{profile.status_label.clone()}</small>
            </div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:1rem;">
                <div>
                    <strong>"Personal facts"</strong>
                    {profile.personal_facts.into_iter().map(|fact| view! { <p style="margin:0.2rem 0;"><strong>{fact.label}</strong>" : "{fact.value}</p> }).collect_view()}
                </div>
                <div>
                    <strong>"Company facts"</strong>
                    {profile.company_facts.into_iter().map(|fact| view! { <p style="margin:0.2rem 0;"><strong>{fact.label}</strong>" : "{fact.value}</p> }).collect_view()}
                </div>
            </div>
            <div>
                <strong>"KYC documents"</strong>
                {if profile.documents.is_empty() {
                    view! { <p>"No KYC documents are attached yet."</p> }.into_any()
                } else {
                    profile.documents.into_iter().map(|document| {
                        let download_path = document.download_path.clone();
                        view! {
                            <div style="display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;padding:0.65rem 0;border-bottom:1px solid #e7e5e4;">
                                <div>
                                    <strong>{document.document_name}</strong>
                                    <p style="margin:0.15rem 0;">{document.original_name.unwrap_or_else(|| "Unnamed file".into())}</p>
                                    <small>{format!("{} | {}", document.document_type, document.uploaded_at_label)}</small>
                                </div>
                                {download_path.map(|path| view! {
                                    <button type="button" on:click=move |_| {
                                        let path = path.clone();
                                        spawn_local(async move {
                                            if let Err(error) = document_upload::open_protected_document(&path).await {
                                                feedback.set(Some(error));
                                            }
                                        });
                                    }>"View file"</button>
                                })}
                            </div>
                        }
                    }).collect_view().into_any()
                }}
            </div>
            <div>
                <strong>"History"</strong>
                {profile.history.into_iter().map(|item| view! {
                    <div style="padding:0.65rem 0;border-bottom:1px solid #e7e5e4;">
                        <strong>{item.status_label}</strong>
                        <small style="display:block;">{item.created_at_label}</small>
                        {item.admin_name.map(|name| view! { <small style="display:block;">{format!("By {}", name)}</small> })}
                        {item.remarks.map(|remarks| view! { <p style="margin:0.25rem 0 0;">{remarks}</p> })}
                    </div>
                }).collect_view()}
            </div>
        </section>
    }
}

fn role_heading(screen: Option<AdminUserDirectoryScreen>, role_key: Option<String>) -> String {
    let current_role_key = role_key.unwrap_or_else(|| "carrier".into());
    screen
        .and_then(|value| {
            value
                .role_options
                .into_iter()
                .find(|option| option.key == current_role_key)
                .map(|option| option.label)
        })
        .unwrap_or_else(|| fallback_role_label(&current_role_key))
        + " Directory"
}

fn role_subtitle(role_key: Option<String>, show_all_statuses: bool) -> String {
    let role_label = fallback_role_label(&role_key.unwrap_or_else(|| "carrier".into()));
    if show_all_statuses {
        format!(
            "Rust replacement for the Laravel {} list, widened to show all account statuses.",
            role_label
        )
    } else {
        format!(
            "Rust replacement for the Laravel {} list, starting with approved accounts only just like the Blade screen.",
            role_label
        )
    }
}

fn fallback_role_label(role_key: &str) -> String {
    match role_key {
        "carrier" => "Carrier".into(),
        "shipper" => "Shipper".into(),
        "broker" => "Broker".into(),
        "freight_forwarder" => "Freight Forwarder".into(),
        "admin" => "Admin".into(),
        _ => "User".into(),
    }
}

fn role_user_matches_query(user: &AdminUserDirectoryUser, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }

    [
        user.name.as_str(),
        user.email.as_str(),
        user.role_label.as_str(),
        user.status_label.as_str(),
        user.company_name.as_deref().unwrap_or_default(),
        user.phone_no.as_deref().unwrap_or_default(),
    ]
    .into_iter()
    .any(|value| value.to_ascii_lowercase().contains(query))
}
