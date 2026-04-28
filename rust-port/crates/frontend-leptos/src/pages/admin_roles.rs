use leptos::{ev::SubmitEvent, prelude::*, tachys::view::any_view::IntoAny, task::spawn_local};

use crate::{
    api,
    session::{self, use_auth},
};
use shared::{AdminRolePermissionOption, AdminRolePermissionRow, AdminRolePermissionScreen};

use super::admin_guard_view;

#[component]
pub fn AdminRolesPage() -> impl IntoView {
    let auth = use_auth();
    let screen = RwSignal::new(None::<AdminRolePermissionScreen>);
    let loading = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);
    let action_loading_role_key = RwSignal::new(None::<String>);
    let refresh_nonce = RwSignal::new(0_u64);

    let can_view = Signal::derive(move || {
        session::has_permission(&auth, "access_admin_portal")
            || session::has_permission(&auth, "manage_roles")
    });

    Effect::new(move |_| {
        let _refresh = refresh_nonce.get();
        if !auth.session_ready.get() || !auth.session.get().authenticated || !can_view.get() {
            return;
        }

        loading.set(true);
        let auth = auth.clone();
        spawn_local(async move {
            match api::fetch_admin_role_permissions().await {
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
            if let Some(guard) = admin_guard_view(&auth, "Role Permissions", &["access_admin_portal", "manage_roles"]) {
                guard
            } else {
                view! {
                    <article style="display:grid;gap:1rem;">
                        <section style="display:grid;gap:0.35rem;">
                            <h2>"Role Permissions"</h2>
                            <p>"Edit the live role-to-permission matrix that Rust sessions now resolve from at runtime."</p>
                        </section>

                        {move || if loading.get() && screen.get().is_none() {
                            view! { <p>"Loading role-permission matrix from the Rust backend..."</p> }.into_any()
                        } else if let Some(screen_data) = screen.get() {
                            let permissions = screen_data.permissions.clone();
                            view! {
                                <>
                                    <section style="display:grid;gap:1rem;">
                                        {screen_data.roles.into_iter().map(|role| {
                                            render_role_card(
                                                role,
                                                permissions.clone(),
                                                feedback,
                                                action_loading_role_key,
                                                refresh_nonce,
                                            )
                                        }).collect_view()}
                                    </section>
                                </>
                            }.into_any()
                        } else {
                            view! { <p>"No role-permission data is available yet."</p> }.into_any()
                        }}
                    </article>
                }.into_any()
            }
        }}
    }
}

fn render_role_card(
    role: AdminRolePermissionRow,
    permissions: Vec<AdminRolePermissionOption>,
    feedback: RwSignal<Option<String>>,
    action_loading_role_key: RwSignal<Option<String>>,
    refresh_nonce: RwSignal<u64>,
) -> impl IntoView {
    let role_key = role.role_key.clone();
    let role_label = role.role_label.clone();
    let initial_permission_count = role.assigned_permission_keys.len();
    let selected_permissions = RwSignal::new(role.assigned_permission_keys.clone());
    let submit_role_key = role_key.clone();
    let disabled_role_key = role_key.clone();
    let label_role_key = role_key.clone();

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        action_loading_role_key.set(Some(submit_role_key.clone()));

        let payload = shared::AdminUpdateRolePermissionsRequest {
            permission_keys: selected_permissions.get(),
        };
        let request_role_key = submit_role_key.clone();

        spawn_local(async move {
            match api::update_admin_role_permissions(&request_role_key, &payload).await {
                Ok(response) => {
                    feedback.set(Some(response.message));
                    if response.success {
                        selected_permissions.set(response.assigned_permission_keys);
                        refresh_nonce.update(|value| *value += 1);
                    }
                }
                Err(error) => feedback.set(Some(error)),
            }
            action_loading_role_key.set(None);
        });
    };

    view! {
        <article style="padding:1rem;border:1px solid #d6d3d1;border-radius:1rem;background:#fcfcfb;display:grid;gap:0.85rem;">
            <div style="display:flex;justify-content:space-between;gap:1rem;flex-wrap:wrap;align-items:flex-start;">
                <div style="display:grid;gap:0.2rem;">
                    <strong>{role_label.clone()}</strong>
                    <small>{format!("{} permission(s) assigned", initial_permission_count)}</small>
                </div>
                <span style="padding:0.25rem 0.6rem;border-radius:999px;background:#f1f5f9;color:#475569;">{role_key.clone()}</span>
            </div>

            <form on:submit=submit style="display:grid;gap:0.85rem;">
                <section style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.75rem;">
                    {permissions.into_iter().map(|permission| {
                        let permission_key = permission.key.clone();
                        let checked_permission_key = permission_key.clone();
                        let change_permission_key = permission_key.clone();
                        view! {
                            <label style="display:grid;gap:0.2rem;padding:0.75rem;border:1px solid #e7e5e4;border-radius:0.85rem;background:white;">
                                <span style="display:flex;gap:0.6rem;align-items:flex-start;">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_permissions.get().iter().any(|value| value == &checked_permission_key)
                                        on:change=move |ev| {
                                            let checked = event_target_checked(&ev);
                                            selected_permissions.update(|values| {
                                                if checked {
                                                    if !values.iter().any(|value| value == &change_permission_key) {
                                                        values.push(change_permission_key.clone());
                                                    }
                                                } else {
                                                    values.retain(|value| value != &change_permission_key);
                                                }
                                                values.sort();
                                            });
                                        }
                                    />
                                    <strong>{permission.label}</strong>
                                </span>
                                <small>{permission.description}</small>
                            </label>
                        }
                    }).collect_view()}
                </section>
                <div style="display:flex;justify-content:flex-end;">
                    <button
                        type="submit"
                        disabled=move || action_loading_role_key.get() == Some(disabled_role_key.clone())
                        style="padding:0.6rem 0.9rem;border:none;border-radius:0.8rem;background:#111827;color:white;cursor:pointer;"
                    >
                        {move || if action_loading_role_key.get() == Some(label_role_key.clone()) { "Saving..." } else { "Save role permissions" }}
                    </button>
                </div>
            </form>
        </article>
    }
}
