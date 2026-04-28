pub mod api;
pub mod device_location;
pub mod document_upload;
pub mod google_places;
pub mod layouts;
pub mod pages;
pub mod realtime;
pub mod runtime_config;
pub mod session;

use leptos::{prelude::*, tachys::view::any_view::IntoAny};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use layouts::{AdminFrame, AuthFrame, UserFrame};
use pages::{
    AccountLifecyclePage, AdminChangePasswordPage, AdminDashboardPage, AdminLoadsPage,
    AdminRolesPage, AdminUsersByRolePage, AdminUsersPage, ChatWorkspacePage, DashboardPage,
    DispatchDeskPage, EscrowOperationsPage, ExecutionLegPage, ForgotPasswordPage, LoadBoardPage,
    LoadBuilderPage, LoadProfilePage, LoginPage, MasterDataPage, NotFoundPage, OnboardingPage,
    OnboardingReviewPage, PortalLandingPage, ProfilePage, RegisterPage, ResetPasswordPage,
    StloadsOperationsPage, StloadsReconciliationPage, VerifyOtpPage,
};
use session::{AuthProvider, use_auth};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <Router>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <Route
                        path=path!("")
                        view=|| view! { <HomePage /> }
                    />
                    <Route
                        path=path!("dashboard")
                        view=|| view! { <DashboardRoutePage /> }
                    />
                    <Route
                        path=path!("loads")
                        view=|| view! {
                            <UserFrame>
                                <LoadBoardPage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("loads/new")
                        view=|| view! {
                            <UserFrame>
                                <LoadBuilderPage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("loads/:load_id/edit")
                        view=|| view! {
                            <UserFrame>
                                <LoadBuilderPage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("loads/:load_id")
                        view=|| view! {
                            <UserFrame>
                                <LoadProfilePage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("execution/legs/:leg_id")
                        view=|| view! {
                            <UserFrame>
                                <ExecutionLegPage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("chat")
                        view=|| view! {
                            <UserFrame>
                                <ChatWorkspacePage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("profile")
                        view=|| view! {
                            <UserFrame>
                                <ProfilePage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("desk/:desk_key")
                        view=|| view! {
                            <UserFrame>
                                <DispatchDeskPage />
                            </UserFrame>
                        }
                    />
                    <Route
                        path=path!("auth/login")
                        view=|| view! {
                            <AuthFrame>
                                <LoginPage />
                            </AuthFrame>
                        }
                    />
                    <Route
                        path=path!("auth/register")
                        view=|| view! {
                            <AuthFrame>
                                <RegisterPage />
                            </AuthFrame>
                        }
                    />
                    <Route
                        path=path!("auth/verify-otp")
                        view=|| view! {
                            <AuthFrame>
                                <VerifyOtpPage />
                            </AuthFrame>
                        }
                    />
                    <Route
                        path=path!("auth/forgot-password")
                        view=|| view! {
                            <AuthFrame>
                                <ForgotPasswordPage />
                            </AuthFrame>
                        }
                    />
                    <Route
                        path=path!("auth/reset-password")
                        view=|| view! {
                            <AuthFrame>
                                <ResetPasswordPage />
                            </AuthFrame>
                        }
                    />
                    <Route
                        path=path!("auth/onboarding")
                        view=|| view! {
                            <AuthFrame>
                                <OnboardingPage />
                            </AuthFrame>
                        }
                    />
                    <Route
                        path=path!("admin")
                        view=|| view! {
                            <AdminFrame>
                                <AdminDashboardPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/payments")
                        view=|| view! {
                            <AdminFrame>
                                <EscrowOperationsPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/master-data")
                        view=|| view! {
                            <AdminFrame>
                                <MasterDataPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/onboarding-reviews")
                        view=|| view! {
                            <AdminFrame>
                                <OnboardingReviewPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/account-lifecycle")
                        view=|| view! {
                            <AdminFrame>
                                <AccountLifecyclePage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/users")
                        view=|| view! {
                            <AdminFrame>
                                <AdminUsersPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/users/role/:role_key")
                        view=|| view! {
                            <AdminFrame>
                                <AdminUsersByRolePage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/change-password")
                        view=|| view! {
                            <AdminFrame>
                                <AdminChangePasswordPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/loads")
                        view=|| view! {
                            <AdminFrame>
                                <AdminLoadsPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/loads/:load_id")
                        view=|| view! {
                            <AdminFrame>
                                <LoadProfilePage admin_mode=true />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/roles")
                        view=|| view! {
                            <AdminFrame>
                                <AdminRolesPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/stloads")
                        view=|| view! {
                            <AdminFrame>
                                <StloadsOperationsPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/stloads/operations")
                        view=|| view! {
                            <AdminFrame>
                                <StloadsOperationsPage />
                            </AdminFrame>
                        }
                    />
                    <Route
                        path=path!("admin/stloads/reconciliation")
                        view=|| view! {
                            <AdminFrame>
                                <StloadsReconciliationPage />
                            </AdminFrame>
                        }
                    />
                </Routes>
            </Router>
        </AuthProvider>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let auth = use_auth();

    view! {
        {move || {
            if !auth.session_ready.get() {
                view! {
                    <AuthFrame>
                        <PortalLandingPage />
                    </AuthFrame>
                }
                .into_any()
            } else if auth
                .session
                .get()
                .user
                .as_ref()
                .map(|user| user.dashboard_href.as_str() == "/admin")
                .unwrap_or(false)
            {
                view! {
                    <AdminFrame>
                        <AdminDashboardPage />
                    </AdminFrame>
                }
                .into_any()
            } else if auth.session.get().authenticated {
                view! {
                    <UserFrame>
                        <DashboardPage />
                    </UserFrame>
                }
                .into_any()
            } else {
                view! {
                    <AuthFrame>
                        <PortalLandingPage />
                    </AuthFrame>
                }
                .into_any()
            }
        }}
    }
}

#[component]
fn DashboardRoutePage() -> impl IntoView {
    let auth = use_auth();

    view! {
        {move || {
            if !auth.session_ready.get() {
                view! {
                    <AuthFrame>
                        <section class="auth-shell" style="min-height:55vh;align-items:center;">
                            <div class="auth-card" style="max-width:32rem;text-align:center;">
                                <span class="auth-kicker">"Checking session"</span>
                                <h1>"Opening the correct workspace"</h1>
                                <p>
                                    "The Rust app is resolving whether this session belongs in the admin portal or the user workspace before loading the dashboard."
                                </p>
                            </div>
                        </section>
                    </AuthFrame>
                }
                .into_any()
            } else if auth
                .session
                .get()
                .user
                .as_ref()
                .map(|user| user.dashboard_href.as_str() == "/admin")
                .unwrap_or(false)
            {
                view! {
                    <AdminFrame>
                        <AdminDashboardPage />
                    </AdminFrame>
                }
                .into_any()
            } else {
                view! {
                    <UserFrame>
                        <DashboardPage />
                    </UserFrame>
                }
                .into_any()
            }
        }}
    }
}
