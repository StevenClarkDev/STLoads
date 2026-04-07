pub mod admin;
pub mod auth;
pub mod dispatch;
pub mod execution;
pub mod marketplace;
pub mod master_data;
pub mod payments;
pub mod realtime;
pub mod tms;

pub const GROUP_NAMES: &[&str] = &[
    "auth",
    "dispatch",
    "marketplace",
    "execution",
    "payments",
    "tms",
    "admin",
    "master-data",
    "realtime",
];
