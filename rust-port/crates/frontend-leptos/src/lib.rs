pub mod api;
pub mod layouts;
pub mod pages;
pub mod realtime;
pub mod session;

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use layouts::{AdminFrame, AuthFrame, UserFrame};
use pages::{
    AdminDashboardPage, ChatWorkspacePage, DashboardPage, EscrowOperationsPage, LoadBoardPage,
    LoadBuilderPage, LoginPage, MasterDataPage, NotFoundPage, StloadsOperationsPage,
    StloadsReconciliationPage,
};
use session::AuthProvider;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <Router>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <Route
                        path=path!("")
                        view=|| view! {
                            <UserFrame>
                                <DashboardPage />
                            </UserFrame>
                        }
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
                        path=path!("chat")
                        view=|| view! {
                            <UserFrame>
                                <ChatWorkspacePage />
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
