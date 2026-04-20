use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use bcrypt::hash;
use db::{
    auth::{
        CreateAdminUserInput, UpdateAdminUserProfileInput, create_admin_user_account,
        delete_admin_user_account, find_user_by_id, find_user_detail_by_user_id, list_admin_users,
        list_kyc_documents_by_user_id, list_pending_onboarding_users,
        list_permission_names_for_role, list_user_history_entries, list_user_ids_for_role,
        replace_role_permissions, review_onboarding_user, update_admin_user_account,
        update_admin_user_profile,
    },
    dispatch::{count_admin_load_legs_filtered, list_admin_load_legs_filtered, review_load_status},
    tms::resolve_sync_error,
};
use domain::auth::{
    AccountStatus, UserRole, permission_descriptors, role_descriptors, role_permission_contracts,
};
use serde::{Deserialize, Serialize};
use shared::{
    AdminCreateUserRequest, AdminCreateUserResponse, AdminDeleteUserResponse, AdminLoadListScreen,
    AdminLoadRow, AdminLoadTab, AdminOnboardingReviewScreen, AdminOnboardingReviewUser,
    AdminReviewLoadRequest, AdminReviewLoadResponse, AdminRolePermissionOption,
    AdminRolePermissionRow, AdminRolePermissionScreen, AdminUpdateRolePermissionsRequest,
    AdminUpdateRolePermissionsResponse, AdminUpdateUserProfileRequest,
    AdminUpdateUserProfileResponse, AdminUpdateUserRequest, AdminUpdateUserResponse,
    AdminUserDirectoryRoleOption, AdminUserDirectoryScreen, AdminUserDirectoryStatusOption,
    AdminUserDirectoryUser, AdminUserHistoryItem, AdminUserProfileFact, AdminUserProfileScreen,
    ApiResponse, KycDocumentItem, RealtimeEvent, RealtimeEventKind, RealtimeTopic,
    ResolveSyncErrorRequest, ResolveSyncErrorResponse, ReviewOnboardingRequest,
    ReviewOnboardingResponse, StloadsOperationsScreen, StloadsReconciliationScreen,
};

use crate::{
    auth_session, auth_session::ResolvedSession, realtime_bus::RoutedRealtimeEvent, screen_data,
    state::AppState,
};

#[derive(Debug, Serialize)]
struct AdminOverview {
    screen_routes: Vec<&'static str>,
    operational_views: usize,
    notes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct OperationsQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReconciliationQuery {
    action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdminLoadsQuery {
    tab: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/stloads/operations", get(stloads_operations))
        .route("/stloads/reconciliation", get(stloads_reconciliation))
        .route("/users", get(user_directory))
        .route("/loads", get(load_list))
        .route("/loads/{load_id}/review", post(review_load_handler))
        .route("/users", post(create_user_handler))
        .route("/users/{user_id}/profile", get(user_profile))
        .route(
            "/users/{user_id}/profile",
            post(update_user_profile_handler),
        )
        .route("/users/{user_id}/delete", post(delete_user_handler))
        .route("/roles/permissions", get(role_permissions))
        .route(
            "/users/{user_id}/account",
            post(update_user_account_handler),
        )
        .route(
            "/roles/{role_key}/permissions",
            post(update_role_permissions_handler),
        )
        .route("/onboarding-reviews", get(onboarding_reviews))
        .route("/users/{user_id}/review", post(review_user_handler))
        .route(
            "/stloads/sync-errors/{sync_error_id}/resolve",
            post(resolve_sync_error_handler),
        )
}

async fn index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminOverview>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &[
            "access_admin_portal",
            "manage_tms_operations",
            "manage_payments",
            "manage_master_data",
            "manage_users",
        ],
    )
    .await?;

    Ok(Json(ApiResponse::ok(AdminOverview {
        screen_routes: vec![
            "/admin/users",
            "/admin/roles",
            "/admin/onboarding-reviews",
            "/admin/stloads/operations",
            "/admin/stloads/reconciliation",
            "/admin/payments",
            "/admin/master-data",
            "/admin/loads",
        ],
        operational_views: 8,
        notes: vec![
            "Admin is now the route home for ops dashboards rather than a single placeholder.",
            "Master-data visibility now lives alongside payments and TMS so load-builder dependencies can be migrated in sequence.",
            "Role-permission editing now uses the same database-backed permission source that live Rust sessions resolve against.",
            "Admin loads now mirrors the Laravel approval and operations tabs inside the Rust admin shell.",
            "IBM-targeted runtime config is environment-driven so these routes can boot on fresh servers without code edits.",
        ],
    })))
}

async fn load_list(
    State(state): State<AppState>,
    Query(query): Query<AdminLoadsQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminLoadListScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_loads"]).await?;

    let active_tab = normalize_admin_load_tab(query.tab.as_deref());
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminLoadListScreen {
            title: "Admin Loads".into(),
            subtitle:
                "Load oversight is unavailable because the Rust database connection is not ready."
                    .into(),
            active_tab: active_tab.clone(),
            tabs: admin_load_tabs(&active_tab, 0, 0, 0, 0, 0),
            rows: Vec::new(),
            notes: vec![format!(
                "Admin load visibility is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
            pagination: shared::Pagination {
                page: 1,
                per_page: 30,
                total: 0,
            },
        })));
    };

    let all_count = count_admin_load_legs_filtered(pool, None)
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let pending_count = count_admin_load_legs_filtered(pool, Some(&[1]))
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let approved_count = count_admin_load_legs_filtered(pool, Some(&[2, 3, 4, 5, 6, 8, 9]))
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let release_count = count_admin_load_legs_filtered(pool, Some(&[10]))
        .await
        .unwrap_or_default()
        .max(0) as u64;
    let completed_count = count_admin_load_legs_filtered(pool, Some(&[11]))
        .await
        .unwrap_or_default()
        .max(0) as u64;

    let filter_statuses = admin_load_tab_statuses(&active_tab);
    let rows = list_admin_load_legs_filtered(pool, filter_statuses, 30)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(map_admin_load_row)
        .collect::<Vec<_>>();

    Ok(Json(ApiResponse::ok(AdminLoadListScreen {
        title: "Admin Loads".into(),
        subtitle: "Mirror of the Laravel admin load tabs for approval, active execution, completion, and release-readiness.".into(),
        active_tab: active_tab.clone(),
        tabs: admin_load_tabs(
            &active_tab,
            all_count,
            pending_count,
            approved_count,
            completed_count,
            release_count,
        ),
        rows,
        notes: vec![
            "Pending mirrors PHP status 1, approved-active mirrors 2,3,4,5,6,8,9, fund release mirrors 10, and completed mirrors 11.".into(),
            "Profile links now stay inside the Rust admin shell instead of bouncing back to Blade.".into(),
            "Release-ready rows deep-link into the Rust payments console until the final in-row finance actions land.".into(),
        ],
        pagination: shared::Pagination {
            page: 1,
            per_page: 30,
            total: match active_tab.as_str() {
                "pending" => pending_count,
                "approved" => approved_count,
                "completed" => completed_count,
                "release-funds" => release_count,
                _ => all_count,
            },
        },
    })))
}

async fn review_load_handler(
    State(state): State<AppState>,
    Path(load_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminReviewLoadRequest>,
) -> Result<Json<ApiResponse<AdminReviewLoadResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_loads"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
            success: false,
            load_id,
            status_label: "Unavailable".into(),
            message: format!(
                "Load review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let decision = payload.decision.trim().to_ascii_lowercase();
    let (status_id, status_label) = match decision.as_str() {
        "approve" | "approved" => (2_i16, "Approved"),
        "reject" | "rejected" => (0_i16, "Rejected"),
        "revision" | "revise" | "send_back" | "send-back" => (7_i16, "Revision Requested"),
        _ => {
            return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
                success: false,
                load_id,
                status_label: "Invalid".into(),
                message: "Choose approve, reject, or revision for the Rust load review action."
                    .into(),
            })));
        }
    };

    let remarks = payload
        .remarks
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if matches!(status_id, 0 | 7) && remarks.is_none() {
        return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
            success: false,
            load_id,
            status_label: status_label.into(),
            message: "Remarks are required when rejecting a load or sending it back for revision."
                .into(),
        })));
    }

    let reviewed = review_load_status(pool, load_id, status_id, remarks, Some(session.user.id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(scope) = reviewed else {
        return Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
            success: false,
            load_id,
            status_label: "Missing".into(),
            message: "The selected load was not found in the Rust admin review flow.".into(),
        })));
    };

    let summary = match status_id {
        2 => format!(
            "{} approved load #{} from the Rust admin loads page.",
            session.user.name, load_id
        ),
        0 => format!(
            "{} rejected load #{} from the Rust admin loads page.",
            session.user.name, load_id
        ),
        _ => format!(
            "{} returned load #{} for revision from the Rust admin loads page.",
            session.user.name, load_id
        ),
    };

    let mail_note = match scope.owner_user_id {
        Some(owner_user_id) => match find_user_by_id(pool, owner_user_id).await {
            Ok(Some(owner)) => match state
                .email
                .send_load_review_status(&owner.email, &owner.name, load_id, status_id, remarks)
                .await
            {
                Ok(outcome) => outcome.status_note(),
                Err(error) => Some(format!("Load review email delivery failed: {}", error)),
            },
            Ok(None) => Some("Load owner was not found for email notification.".into()),
            Err(error) => Some(format!("Load owner lookup for email failed: {}", error)),
        },
        None => Some("Load owner was unavailable for email notification.".into()),
    };

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: scope.owner_user_id.map(|value| value.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys([
            "access_admin_portal",
            "manage_loads",
            "manage_dispatch_desk",
        ])
        .with_topics([
            RealtimeTopic::AdminDashboard.as_key(),
            RealtimeTopic::LoadBoard.as_key(),
        ]),
    );

    Ok(Json(ApiResponse::ok(AdminReviewLoadResponse {
        success: true,
        load_id,
        status_label: status_label.into(),
        message: append_optional_note(summary, mail_note),
    })))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("admin route group ready"))
}

async fn onboarding_reviews(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminOnboardingReviewScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminOnboardingReviewScreen {
            title: "Onboarding Reviews".into(),
            summary: "Database unavailable".into(),
            users: Vec::new(),
            notes: vec![format!(
                "Admin review is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let users = list_pending_onboarding_users(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| async move {
            let documents = list_kyc_documents_by_user_id(pool, row.user_id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|document| KycDocumentItem {
                    blockchain_label: if document.document_type.eq_ignore_ascii_case("blockchain") {
                        Some("Anchored to blockchain".into())
                    } else {
                        None
                    },
                    blockchain_tone: if document.document_type.eq_ignore_ascii_case("blockchain") {
                        Some("success".into())
                    } else {
                        None
                    },
                    blockchain_hash_preview: document.hash.as_ref().map(|hash| {
                        if hash.len() > 24 {
                            format!("{}...", &hash[..24])
                        } else {
                            hash.clone()
                        }
                    }),
                    blockchain_hash: document.hash.clone(),
                    id: document.id.max(0) as u64,
                    document_name: document.document_name,
                    document_type: document.document_type,
                    file_label: document
                        .original_name
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| "KYC file".into()),
                    original_name: document.original_name,
                    mime_type: document.mime_type,
                    file_size_bytes: document
                        .file_size
                        .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
                    uploaded_at_label: format_datetime(&document.created_at),
                    download_path: Some(format!(
                        "/auth/onboarding/documents/{}/file",
                        document.id.max(0) as u64
                    )),
                    can_view_file: true,
                    can_edit: false,
                    can_verify_blockchain: false,
                    can_delete: false,
                })
                .collect::<Vec<_>>();

            AdminOnboardingReviewUser {
                user_id: row.user_id.max(0) as u64,
                name: row.name,
                email: row.email,
                role_label: row
                    .role_id
                    .and_then(UserRole::from_legacy_id)
                    .map(|role| role.label().to_string())
                    .unwrap_or_else(|| "Unknown".into()),
                status_label: AccountStatus::from_legacy_code(row.status)
                    .map(account_status_label)
                    .unwrap_or_else(|| format!("Status {}", row.status)),
                company_name: row.company_name,
                company_address: row.company_address,
                submitted_at_label: row
                    .submitted_at
                    .as_ref()
                    .map(format_datetime)
                    .unwrap_or_else(|| "Not submitted".into()),
                document_count: row.document_count.max(0) as u64,
                documents,
            }
        });

    let mut review_users = Vec::new();
    for future in users {
        review_users.push(future.await);
    }

    Ok(Json(ApiResponse::ok(AdminOnboardingReviewScreen {
        title: "Onboarding Reviews".into(),
        summary: format!("{} account(s) currently need review or revision handling.", review_users.len()),
        users: review_users,
        notes: vec![
            "Uploaded KYC files are protected by the same Rust document access rule: admin users and the uploader can open them.".into(),
            "Approve moves the account to Approved, Request Revision returns it to Revision Requested, and Reject moves it to Rejected.".into(),
        ],
    })))
}

async fn user_directory(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminUserDirectoryScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUserDirectoryScreen {
            title: "User Directory".into(),
            summary: "Database unavailable".into(),
            role_options: admin_role_options(),
            status_options: admin_status_options(),
            users: Vec::new(),
            notes: vec![format!(
                "User management is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let users = list_admin_users(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|row| AdminUserDirectoryUser {
            user_id: row.user_id.max(0) as u64,
            name: row.name,
            email: row.email,
            role_key: row
                .role_id
                .and_then(UserRole::from_legacy_id)
                .map(admin_role_key)
                .unwrap_or_else(|| "unknown".into()),
            role_label: row
                .role_id
                .and_then(UserRole::from_legacy_id)
                .map(|role| role.label().to_string())
                .unwrap_or_else(|| "Unknown".into()),
            status_key: AccountStatus::from_legacy_code(row.status)
                .map(admin_status_key)
                .unwrap_or_else(|| "unknown".into()),
            status_label: AccountStatus::from_legacy_code(row.status)
                .map(account_status_label)
                .unwrap_or_else(|| format!("Status {}", row.status)),
            company_name: row.company_name,
            phone_no: row.phone_no,
            joined_at_label: format_datetime(&row.joined_at),
            document_count: row.document_count.max(0) as u64,
            latest_review_note: row.latest_review_note,
        })
        .collect::<Vec<_>>();

    Ok(Json(ApiResponse::ok(AdminUserDirectoryScreen {
        title: "User Directory".into(),
        summary: format!(
            "{} user account(s) are currently available in the Rust admin directory.",
            users.len()
        ),
        role_options: admin_role_options(),
        status_options: admin_status_options(),
        users,
        notes: vec![
            "Role changes update both users.role_id and the Spatie role pivot so Rust session permissions stay consistent.".into(),
            "Status changes are written to user_history so account review context is preserved alongside onboarding actions.".into(),
        ],
    })))
}

async fn user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminUserProfileScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUserProfileScreen {
            user_id: user_id.max(0) as u64,
            name: "Unavailable".into(),
            email: String::new(),
            role_key: "unknown".into(),
            role_label: "Unknown".into(),
            status_key: "unknown".into(),
            status_label: "Unavailable".into(),
            phone_no: None,
            address: None,
            dob_label: None,
            gender: None,
            joined_at_label: "Unavailable".into(),
            company_name: None,
            company_address: None,
            image_path: None,
            personal_facts: Vec::new(),
            company_facts: Vec::new(),
            documents: Vec::new(),
            history: Vec::new(),
            notes: vec![format!(
                "User profile is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let Some(user) = find_user_by_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let details = find_user_detail_by_user_id(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let documents = list_kyc_documents_by_user_id(pool, user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|document| KycDocumentItem {
            blockchain_label: if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("Anchored to blockchain".into())
            } else {
                None
            },
            blockchain_tone: if document.document_type.eq_ignore_ascii_case("blockchain") {
                Some("success".into())
            } else {
                None
            },
            blockchain_hash_preview: document.hash.as_ref().map(|hash| {
                if hash.len() > 24 {
                    format!("{}...", &hash[..24])
                } else {
                    hash.clone()
                }
            }),
            blockchain_hash: document.hash.clone(),
            id: document.id.max(0) as u64,
            document_name: document.document_name,
            document_type: document.document_type,
            file_label: document
                .original_name
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "KYC file".into()),
            original_name: document.original_name,
            mime_type: document.mime_type,
            file_size_bytes: document
                .file_size
                .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
            uploaded_at_label: format_datetime(&document.created_at),
            download_path: Some(format!(
                "/auth/onboarding/documents/{}/file",
                document.id.max(0) as u64
            )),
            can_view_file: true,
            can_edit: false,
            can_verify_blockchain: false,
            can_delete: false,
        })
        .collect::<Vec<_>>();

    let history = list_user_history_entries(pool, user_id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|entry| AdminUserHistoryItem {
            status_label: AccountStatus::from_legacy_code(entry.status)
                .map(account_status_label)
                .unwrap_or_else(|| format!("Status {}", entry.status)),
            remarks: entry.remarks,
            created_at_label: format_datetime(&entry.created_at),
            admin_name: entry.admin_name,
        })
        .collect::<Vec<_>>();

    let role = user.primary_role();
    let status = user.account_status();

    Ok(Json(ApiResponse::ok(AdminUserProfileScreen {
        user_id: user.id.max(0) as u64,
        name: user.name.clone(),
        email: user.email.clone(),
        role_key: role.map(admin_role_key).unwrap_or_else(|| "unknown".into()),
        role_label: role
            .map(|value| value.label().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        status_key: status
            .map(admin_status_key)
            .unwrap_or_else(|| "unknown".into()),
        status_label: status
            .map(account_status_label)
            .unwrap_or_else(|| format!("Status {}", user.status)),
        phone_no: user.phone_no.clone(),
        address: user.address.clone(),
        dob_label: user.dob.map(|value| value.format("%b %d, %Y").to_string()),
        gender: user.gender.clone(),
        joined_at_label: format_datetime(&user.created_at),
        company_name: details
            .as_ref()
            .and_then(|detail| detail.company_name.clone())
            .or(user.company_name.clone()),
        company_address: details
            .as_ref()
            .and_then(|detail| detail.company_address.clone())
            .or(user.company_address.clone()),
        image_path: user.image.clone(),
        personal_facts: admin_personal_facts(&user),
        company_facts: admin_company_facts(&user, details.as_ref()),
        documents,
        history,
        notes: vec![
            "This Rust profile folds the old modal profile, user detail page, and KYC visibility into one admin surface.".into(),
            "KYC downloads still use the protected uploader-plus-admin access rule.".into(),
        ],
    })))
}

async fn create_user_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<Json<ApiResponse<AdminCreateUserResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: format!(
                "User creation is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Name and email are required before creating a user.".into(),
        })));
    }

    if payload.password != payload.password_confirmation {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Password confirmation does not match.".into(),
        })));
    }

    if payload.password.trim().len() < 8 {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Choose a password with at least 8 characters.".into(),
        })));
    }

    let Some(role) = parse_admin_role_key(&payload.role_key) else {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Choose a valid role before creating the user.".into(),
        })));
    };

    let Some(status) = parse_admin_status_key(&payload.status_key) else {
        return Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
            success: false,
            user_id: None,
            message: "Choose a valid account status before creating the user.".into(),
        })));
    };

    let password_hash =
        hash(&payload.password, 12).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let created = create_admin_user_account(
        pool,
        session.user.id,
        &CreateAdminUserInput {
            name: payload.name.trim().to_string(),
            email: payload.email.trim().to_ascii_lowercase(),
            password_hash,
            role_id: role.legacy_id(),
            status: status.legacy_code(),
            phone_no: payload.phone_no.clone(),
            address: payload.address.clone(),
        },
    )
    .await;

    match created {
        Ok(user) => {
            let summary = format!(
                "{} created {} in the Rust admin directory.",
                session.user.name, user.email
            );
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::AdminDashboardUpdated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: summary.clone(),
                })
                .for_permission_keys(["manage_users", "access_admin_portal"])
                .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
            );

            Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
                success: true,
                user_id: Some(user.id.max(0) as u64),
                message: summary,
            })))
        }
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
            Ok(Json(ApiResponse::ok(AdminCreateUserResponse {
                success: false,
                user_id: None,
                message: "That email address is already in use.".into(),
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_user_profile_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateUserProfileRequest>,
) -> Result<Json<ApiResponse<AdminUpdateUserProfileResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: format!(
                "Profile update is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "Name and email are required before saving the profile.".into(),
        })));
    }

    let password_hash = match (
        payload
            .password
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        payload
            .password_confirmation
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(password), Some(password_confirmation)) => {
            if password != password_confirmation {
                return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                    success: false,
                    user_id: user_id.max(0) as u64,
                    message: "Password confirmation does not match.".into(),
                })));
            }
            if password.len() < 8 {
                return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                    success: false,
                    user_id: user_id.max(0) as u64,
                    message: "Choose a password with at least 8 characters.".into(),
                })));
            }
            Some(hash(password, 12).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        }
        (Some(_), None) | (None, Some(_)) => {
            return Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                success: false,
                user_id: user_id.max(0) as u64,
                message: "Provide both password fields before changing the password.".into(),
            })));
        }
        (None, None) => None,
    };

    let updated = update_admin_user_profile(
        pool,
        &UpdateAdminUserProfileInput {
            user_id,
            admin_id: session.user.id,
            name: payload.name.trim().to_string(),
            email: payload.email.trim().to_ascii_lowercase(),
            password_hash,
            phone_no: payload.phone_no.clone(),
            address: payload.address.clone(),
            remarks: payload.remarks.clone(),
        },
    )
    .await;

    match updated {
        Ok(Some(user)) => {
            let summary = format!(
                "{} updated the Rust profile for {}.",
                session.user.name, user.email
            );
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::AdminDashboardUpdated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id: Some(session.user.id.max(0) as u64),
                    subject_user_id: Some(user.id.max(0) as u64),
                    presence_state: None,
                    last_read_message_id: None,
                    summary: summary.clone(),
                })
                .for_permission_keys(["manage_users", "access_admin_portal"])
                .for_user_ids([user.id.max(0) as u64])
                .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
            );
            Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                success: true,
                user_id: user.id.max(0) as u64,
                message: summary,
            })))
        }
        Ok(None) => Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The selected user account was not found.".into(),
        }))),
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
            Ok(Json(ApiResponse::ok(AdminUpdateUserProfileResponse {
                success: false,
                user_id: user_id.max(0) as u64,
                message: "That email address is already in use.".into(),
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminDeleteUserResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: format!(
                "Delete action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    if session.user.id == user_id {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The Rust admin portal will not let you delete your own account.".into(),
        })));
    }

    let deleted = delete_admin_user_account(pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !deleted {
        return Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            message: "The selected user account was not found.".into(),
        })));
    }

    let summary = format!(
        "{} deleted user #{} from the Rust admin directory.",
        session.user.name, user_id
    );
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(user_id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    Ok(Json(ApiResponse::ok(AdminDeleteUserResponse {
        success: true,
        user_id: user_id.max(0) as u64,
        message: summary,
    })))
}

async fn role_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AdminRolePermissionScreen>>, StatusCode> {
    let _session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminRolePermissionScreen {
            title: "Role Permissions".into(),
            summary: "Database unavailable".into(),
            permissions: permission_option_catalog(),
            roles: Vec::new(),
            notes: vec![format!(
                "Role-permission management is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        })));
    };

    let mut roles = Vec::new();
    for role in role_descriptors() {
        let assigned_permission_keys =
            list_permission_names_for_role(pool, i64::from(role.legacy_id))
                .await
                .unwrap_or_default();
        let assigned_permission_keys = if assigned_permission_keys.is_empty() {
            role_permission_contracts()
                .iter()
                .find(|contract| contract.role == role.role)
                .map(|contract| {
                    contract
                        .permissions
                        .iter()
                        .map(|permission| admin_permission_key(*permission).to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            assigned_permission_keys
        };

        roles.push(AdminRolePermissionRow {
            role_key: admin_role_key(role.role),
            role_label: role.label.into(),
            assigned_permission_keys,
        });
    }

    Ok(Json(ApiResponse::ok(AdminRolePermissionScreen {
        title: "Role Permissions".into(),
        summary: format!(
            "{} role profiles currently have editable permission sets in the Rust admin surface.",
            roles.len()
        ),
        permissions: permission_option_catalog(),
        roles,
        notes: vec![
            "Rust sessions now resolve permission keys from role_has_permissions first, with the static role contract as a safe fallback if the database has no assignment yet.".into(),
            "Saving a role permission set invalidates active sessions for users on that role so websocket scopes and route guards reconnect with the updated permissions.".into(),
        ],
    })))
}

async fn review_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ReviewOnboardingRequest>,
) -> Result<Json<ApiResponse<ReviewOnboardingResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            status_label: None,
            message: format!(
                "Review action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let next_status = match payload.decision.trim().to_ascii_lowercase().as_str() {
        "approve" | "approved" => AccountStatus::Approved,
        "reject" | "rejected" => AccountStatus::Rejected,
        "revision" | "revise" | "request_revision" => AccountStatus::RevisionRequested,
        _ => {
            return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
                success: false,
                user_id: user_id.max(0) as u64,
                status_label: None,
                message: "Choose approve, reject, or revision for the Rust review action.".into(),
            })));
        }
    };

    let updated = review_onboarding_user(
        pool,
        user_id,
        session.user.id,
        next_status.legacy_code(),
        payload.remarks.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(updated_user) = updated else {
        return Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            status_label: None,
            message: "The selected onboarding account was not found.".into(),
        })));
    };

    let mail_note = send_account_review_notification(
        &state,
        &updated_user,
        next_status,
        payload.remarks.as_deref(),
    )
    .await;

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(updated_user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} changed {} to {}.",
                session.user.name,
                updated_user.email,
                account_status_label(next_status)
            ),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"])
        .for_user_ids([updated_user.id.max(0) as u64])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ReviewOnboardingResponse {
        success: true,
        user_id: updated_user.id.max(0) as u64,
        status_label: Some(account_status_label(next_status)),
        message: append_optional_note(
            format!(
                "Rust review updated {} to {}.",
                updated_user.email,
                account_status_label(next_status)
            ),
            mail_note,
        ),
    })))
}

async fn update_user_account_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<Json<ApiResponse<AdminUpdateUserResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_users"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: format!(
                "User update is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(role) = parse_admin_role_key(&payload.role_key) else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "Choose a valid role before saving this user.".into(),
        })));
    };

    let Some(next_status) = parse_admin_status_key(&payload.status_key) else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "Choose a valid account status before saving this user.".into(),
        })));
    };

    let updated = update_admin_user_account(
        pool,
        user_id,
        session.user.id,
        role.legacy_id(),
        next_status.legacy_code(),
        payload.remarks.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(updated_user) = updated else {
        return Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
            success: false,
            user_id: user_id.max(0) as u64,
            role_label: None,
            status_label: None,
            message: "The selected user account was not found.".into(),
        })));
    };

    let updated_role = updated_user.primary_role();
    let updated_status = updated_user.account_status();
    let mail_note = match updated_status {
        Some(
            AccountStatus::Approved | AccountStatus::Rejected | AccountStatus::RevisionRequested,
        ) => {
            send_account_review_notification(
                &state,
                &updated_user,
                updated_status.unwrap(),
                payload.remarks.as_deref(),
            )
            .await
        }
        _ => None,
    };
    let summary = format!(
        "{} updated {} to role {} and status {}.",
        session.user.name,
        updated_user.email,
        updated_role
            .map(|value| value.label().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        account_status_label(updated_status.unwrap_or(AccountStatus::PendingReview))
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: Some(updated_user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"])
        .for_user_ids([updated_user.id.max(0) as u64])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::SessionInvalidated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(updated_user.id.max(0) as u64),
            subject_user_id: Some(updated_user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: "Your role or account status changed in the Rust admin directory. Refresh your session.".into(),
        })
        .for_user_ids([updated_user.id.max(0) as u64]),
    );

    Ok(Json(ApiResponse::ok(AdminUpdateUserResponse {
        success: true,
        user_id: updated_user.id.max(0) as u64,
        role_label: updated_role.map(|value| value.label().to_string()),
        status_label: updated_status.map(account_status_label),
        message: append_optional_note(summary, mail_note),
    })))
}

async fn update_role_permissions_handler(
    State(state): State<AppState>,
    Path(role_key): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AdminUpdateRolePermissionsRequest>,
) -> Result<Json<ApiResponse<AdminUpdateRolePermissionsResponse>>, StatusCode> {
    let session =
        require_any_permission(&state, &headers, &["access_admin_portal", "manage_roles"]).await?;
    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
            success: false,
            role_key,
            assigned_permission_keys: Vec::new(),
            message: format!(
                "Role permission updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let Some(role) = parse_admin_role_key(&role_key) else {
        return Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
            success: false,
            role_key,
            assigned_permission_keys: Vec::new(),
            message: "Choose a valid role before saving permissions.".into(),
        })));
    };

    let valid_permission_keys = permission_descriptors()
        .iter()
        .map(|descriptor| admin_permission_key(descriptor.permission).to_string())
        .collect::<Vec<_>>();

    let filtered_permission_keys = payload
        .permission_keys
        .into_iter()
        .map(|permission| permission.trim().to_string())
        .filter(|permission| {
            valid_permission_keys
                .iter()
                .any(|candidate| candidate == permission)
        })
        .collect::<Vec<_>>();

    if filtered_permission_keys.is_empty() {
        return Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
            success: false,
            role_key: admin_role_key(role),
            assigned_permission_keys: Vec::new(),
            message: "Assign at least one permission before saving a Rust role.".into(),
        })));
    }

    let mut filtered_permission_keys = filtered_permission_keys;
    if role == UserRole::Admin {
        for required_permission in ["access_admin_portal", "manage_roles"] {
            if !filtered_permission_keys
                .iter()
                .any(|permission| permission == required_permission)
            {
                filtered_permission_keys.push(required_permission.into());
            }
        }
        filtered_permission_keys.sort();
        filtered_permission_keys.dedup();
    }

    let assigned_permission_keys =
        replace_role_permissions(pool, i64::from(role.legacy_id()), &filtered_permission_keys)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let affected_user_ids = list_user_ids_for_role(pool, i64::from(role.legacy_id()))
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.max(0) as u64)
        .collect::<Vec<_>>();

    let summary = format!(
        "{} updated the {} role permission set in the Rust admin portal.",
        session.user.name,
        role.label()
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_roles", "access_admin_portal"])
        .with_topics([RealtimeTopic::AdminDashboard.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::SessionInvalidated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "Your role permissions changed for the {} role. Refresh your Rust session.",
                role.label()
            ),
        })
        .for_user_ids(affected_user_ids),
    );

    Ok(Json(ApiResponse::ok(AdminUpdateRolePermissionsResponse {
        success: true,
        role_key: admin_role_key(role),
        assigned_permission_keys,
        message: summary,
    })))
}

async fn stloads_operations(
    State(state): State<AppState>,
    Query(query): Query<OperationsQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StloadsOperationsScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        screen_data::stloads_operations_screen(&state, query.status).await,
    )))
}

async fn stloads_reconciliation(
    State(state): State<AppState>,
    Query(query): Query<ReconciliationQuery>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<StloadsReconciliationScreen>>, StatusCode> {
    let _session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    Ok(Json(ApiResponse::ok(
        screen_data::stloads_reconciliation_screen(&state, query.action).await,
    )))
}

async fn resolve_sync_error_handler(
    State(state): State<AppState>,
    Path(sync_error_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<ResolveSyncErrorRequest>,
) -> Result<Json<ApiResponse<ResolveSyncErrorResponse>>, StatusCode> {
    let session = require_any_permission(
        &state,
        &headers,
        &["access_admin_portal", "manage_tms_operations"],
    )
    .await?;

    let Some(pool) = state.pool.as_ref() else {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: None,
            message: format!(
                "Resolve action is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        })));
    };

    let resolved_by = format!("{} <{}>", session.user.name, session.user.email);
    let resolved_record = resolve_sync_error(
        pool,
        sync_error_id,
        &resolved_by,
        payload.resolution_note.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(sync_error) = resolved_record else {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: None,
            message: "The requested sync error was not found.".into(),
        })));
    };

    if !sync_error.resolved {
        return Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
            success: false,
            sync_error_id,
            handoff_id: sync_error.handoff_id,
            message: "The selected sync error could not be resolved yet.".into(),
        })));
    }

    let summary = format!(
        "{} resolved {} on sync error #{}.",
        session.user.name, sync_error.error_class, sync_error_id
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::TmsOperationsUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary: summary.clone(),
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([RealtimeTopic::AdminTmsOperations.as_key()]),
    );

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::TmsReconciliationUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(session.user.id.max(0) as u64),
            subject_user_id: None,
            presence_state: None,
            last_read_message_id: None,
            summary,
        })
        .for_permission_keys(["manage_tms_operations"])
        .with_topics([RealtimeTopic::AdminTmsReconciliation.as_key()]),
    );

    Ok(Json(ApiResponse::ok(ResolveSyncErrorResponse {
        success: true,
        sync_error_id,
        handoff_id: sync_error.handoff_id,
        message: "Sync error resolved from the Rust STLOADS admin route.".into(),
    })))
}

async fn require_any_permission(
    state: &AppState,
    headers: &HeaderMap,
    permission_keys: &[&str],
) -> Result<ResolvedSession, StatusCode> {
    let Some(session) = auth_session::resolve_session_from_headers(state, headers)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let allowed = session.session.permissions.iter().any(|permission| {
        permission_keys
            .iter()
            .any(|expected| permission == expected)
    });

    if allowed {
        Ok(session)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

async fn send_account_review_notification(
    state: &AppState,
    user: &db::auth::UserRecord,
    status: AccountStatus,
    remarks: Option<&str>,
) -> Option<String> {
    let role_label = user
        .primary_role()
        .map(|role| role.label().to_string())
        .unwrap_or_else(|| "STLoads".into());

    match state
        .email
        .send_account_review_status(&user.email, &user.name, &role_label, status, remarks)
        .await
    {
        Ok(outcome) => outcome.status_note(),
        Err(error) => Some(format!("Account review email delivery failed: {}", error)),
    }
}

fn append_optional_note(base: impl Into<String>, note: Option<String>) -> String {
    let base = base.into();
    match note {
        Some(note) if !note.trim().is_empty() => format!("{} {}", base, note),
        _ => base,
    }
}

fn account_status_label(status: AccountStatus) -> String {
    match status {
        AccountStatus::EmailVerifiedPendingOnboarding => "Email Verified".into(),
        AccountStatus::Approved => "Approved".into(),
        AccountStatus::Rejected => "Rejected".into(),
        AccountStatus::PendingReview => "Pending Review".into(),
        AccountStatus::PendingOtp => "Pending OTP".into(),
        AccountStatus::RevisionRequested => "Revision Requested".into(),
    }
}

fn admin_role_options() -> Vec<AdminUserDirectoryRoleOption> {
    vec![
        AdminUserDirectoryRoleOption {
            key: "admin".into(),
            label: "Admin".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "shipper".into(),
            label: "Shipper".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "carrier".into(),
            label: "Carrier".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "broker".into(),
            label: "Broker".into(),
        },
        AdminUserDirectoryRoleOption {
            key: "freight_forwarder".into(),
            label: "Freight Forwarder".into(),
        },
    ]
}

fn permission_option_catalog() -> Vec<AdminRolePermissionOption> {
    permission_descriptors()
        .iter()
        .map(|descriptor| AdminRolePermissionOption {
            key: admin_permission_key(descriptor.permission).into(),
            label: descriptor.label.into(),
            description: descriptor.description.into(),
        })
        .collect()
}

fn admin_status_options() -> Vec<AdminUserDirectoryStatusOption> {
    vec![
        AdminUserDirectoryStatusOption {
            key: "email_verified".into(),
            label: "Email Verified".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "approved".into(),
            label: "Approved".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "rejected".into(),
            label: "Rejected".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "pending_review".into(),
            label: "Pending Review".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "pending_otp".into(),
            label: "Pending OTP".into(),
        },
        AdminUserDirectoryStatusOption {
            key: "revision_requested".into(),
            label: "Revision Requested".into(),
        },
    ]
}

fn normalize_admin_load_tab(value: Option<&str>) -> String {
    match value.unwrap_or("all").trim().to_ascii_lowercase().as_str() {
        "pending" => "pending".into(),
        "approved" => "approved".into(),
        "completed" => "completed".into(),
        "release-funds" | "release_funds" => "release-funds".into(),
        _ => "all".into(),
    }
}

fn admin_load_tab_statuses(active_tab: &str) -> Option<&'static [i16]> {
    match active_tab {
        "pending" => Some(&[1]),
        "approved" => Some(&[2, 3, 4, 5, 6, 8, 9]),
        "completed" => Some(&[11]),
        "release-funds" => Some(&[10]),
        _ => None,
    }
}

fn admin_load_tabs(
    active_tab: &str,
    all_count: u64,
    pending_count: u64,
    approved_count: u64,
    completed_count: u64,
    release_count: u64,
) -> Vec<AdminLoadTab> {
    vec![
        AdminLoadTab {
            key: "all".into(),
            label: "All Loads".into(),
            count: all_count,
            is_active: active_tab == "all",
        },
        AdminLoadTab {
            key: "pending".into(),
            label: "Pending Approval".into(),
            count: pending_count,
            is_active: active_tab == "pending",
        },
        AdminLoadTab {
            key: "approved".into(),
            label: "Approved / Active".into(),
            count: approved_count,
            is_active: active_tab == "approved",
        },
        AdminLoadTab {
            key: "completed".into(),
            label: "Completed".into(),
            count: completed_count,
            is_active: active_tab == "completed",
        },
        AdminLoadTab {
            key: "release-funds".into(),
            label: "Fund Release".into(),
            count: release_count,
            is_active: active_tab == "release-funds",
        },
    ]
}

fn map_admin_load_row(row: db::dispatch::AdminLoadLegRecord) -> AdminLoadRow {
    let amount = row.booked_amount.or(row.price);
    let payment_label = row
        .escrow_status
        .as_ref()
        .map(|status| match status.as_str() {
            "released" | "paid_out" => "Released".to_string(),
            "funded" => "Funded".to_string(),
            "pending" | "hold" => "Pending".to_string(),
            "unfunded" => "Unfunded".to_string(),
            other => other.replace('_', " "),
        });
    let (finance_action_key, finance_action_label, finance_action_enabled, finance_note) =
        admin_load_finance_action(
            row.status_id,
            row.escrow_status.as_deref(),
            row.carrier_name.is_some(),
        );
    let payments_action = finance_action_key
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "release".into());

    AdminLoadRow {
        load_id: row.load_id.max(0) as u64,
        leg_id: row.leg_id.max(0) as u64,
        status_code: row.status_id,
        leg_code: row
            .leg_code
            .unwrap_or_else(|| format!("LEG-{}", row.leg_id.max(0))),
        owner_label: row.owner_name.unwrap_or_else(|| "Unknown owner".into()),
        carrier_label: row.carrier_name,
        origin_label: row
            .pickup_location_name
            .unwrap_or_else(|| "Unknown pickup".into()),
        destination_label: row
            .delivery_location_name
            .unwrap_or_else(|| "Unknown delivery".into()),
        pickup_date_label: row
            .pickup_date
            .map(|value| value.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "TBD".into()),
        delivery_date_label: row
            .delivery_date
            .map(|value| value.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "TBD".into()),
        status_label: admin_load_status_label(row.status_id),
        status_tone: admin_load_status_tone(row.status_id).into(),
        bid_status_label: row
            .bid_status
            .map(|value| value.replace('_', " "))
            .unwrap_or_else(|| "Open".into()),
        amount_label: amount
            .map(|value| format!("${:.0}", value))
            .unwrap_or_else(|| "Rate unavailable".into()),
        payment_label,
        finance_action_key,
        finance_action_label,
        finance_action_enabled,
        finance_note,
        can_review: row.status_id == 1,
        primary_action_label: if row.status_id == 10 {
            Some("Open Payments".into())
        } else if matches!(row.status_id, 2 | 3 | 4 | 5 | 6 | 8 | 9 | 11) {
            Some("Track".into())
        } else {
            Some("Profile".into())
        },
        load_href: format!("/admin/loads/{}", row.load_id.max(0) as u64),
        track_href: matches!(row.status_id, 2 | 3 | 4 | 5 | 6 | 8 | 9 | 10 | 11)
            .then(|| format!("/execution/legs/{}", row.leg_id.max(0) as u64)),
        payments_href: (row.status_id == 10 || row.escrow_id.is_some()).then(|| {
            format!(
                "/admin/payments?leg_id={}&action={}&source=admin-loads&load_id={}",
                row.leg_id.max(0) as u64,
                payments_action,
                row.load_id.max(0) as u64
            )
        }),
    }
}

fn admin_load_finance_action(
    status_id: i16,
    escrow_status: Option<&str>,
    has_carrier: bool,
) -> (Option<String>, Option<String>, bool, Option<String>) {
    if !has_carrier {
        return (
            None,
            None,
            false,
            Some("Finance actions stay locked until a carrier is assigned.".into()),
        );
    }

    match escrow_status {
        Some("released" | "paid_out") => (
            None,
            None,
            false,
            Some("Funds already released for this leg.".into()),
        ),
        Some("funded") if status_id >= 10 => (
            Some("release".into()),
            Some("Release Funds".into()),
            true,
            Some("This leg is in the release-ready stage and can be paid out directly from the Rust admin list.".into()),
        ),
        Some("funded") => (
            Some("hold".into()),
            Some("Place On Hold".into()),
            true,
            Some("Escrow is funded, but the load is not yet fully release-ready.".into()),
        ),
        Some("pending" | "hold") => (
            Some("hold".into()),
            Some("Refresh Hold".into()),
            true,
            Some("This escrow already needs finance review, so hold flow is the safest direct action.".into()),
        ),
        Some("unfunded") | None if matches!(status_id, 4 | 5 | 6 | 8 | 9 | 10) => (
            Some("fund".into()),
            Some("Fund Escrow".into()),
            true,
            Some("This booked or live leg still needs the initial escrow funding step.".into()),
        ),
        Some(other) => (
            None,
            None,
            false,
            Some(format!(
                "Finance status is {}. Use the payments console for exception handling.",
                other.replace('_', " ")
            )),
        ),
        None => (None, None, false, None),
    }
}

fn admin_load_status_label(status_id: i16) -> String {
    match status_id {
        0 => "Rejected".into(),
        1 => "Pending Approval".into(),
        2 => "Approved".into(),
        7 => "Revision Requested".into(),
        _ => match domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id) {
            Some(domain::dispatch::LegacyLoadLegStatusCode::Reviewed) => "Reviewed".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::OfferReady) => "Offer Ready".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::Booked) => "Booked".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::PickupStarted) => {
                "Pickup Started".into()
            }
            Some(domain::dispatch::LegacyLoadLegStatusCode::AtPickup) => "At Pickup".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::InTransit) => "In Transit".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded) => {
                "Ready for Fund Release".into()
            }
            Some(domain::dispatch::LegacyLoadLegStatusCode::AtDelivery) => "At Delivery".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered) => "Completed".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut) => "Paid Out".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::Draft) => "Draft".into(),
            Some(domain::dispatch::LegacyLoadLegStatusCode::New) => "New".into(),
            None => format!("Status {}", status_id),
        },
    }
}

fn admin_load_status_tone(status_id: i16) -> &'static str {
    match status_id {
        0 => "danger",
        1 => "warning",
        2 => "success",
        7 => "warning",
        10 => "primary",
        11 => "secondary",
        _ => match domain::dispatch::LegacyLoadLegStatusCode::from_legacy_code(status_id) {
            Some(domain::dispatch::LegacyLoadLegStatusCode::Booked) => "primary",
            Some(
                domain::dispatch::LegacyLoadLegStatusCode::PickupStarted
                | domain::dispatch::LegacyLoadLegStatusCode::AtPickup
                | domain::dispatch::LegacyLoadLegStatusCode::InTransit
                | domain::dispatch::LegacyLoadLegStatusCode::AtDelivery,
            ) => "info",
            Some(domain::dispatch::LegacyLoadLegStatusCode::Delivered) => "secondary",
            Some(domain::dispatch::LegacyLoadLegStatusCode::PaidOut) => "dark",
            Some(domain::dispatch::LegacyLoadLegStatusCode::EscrowFunded) => "primary",
            Some(domain::dispatch::LegacyLoadLegStatusCode::OfferReady) => "info",
            Some(domain::dispatch::LegacyLoadLegStatusCode::Reviewed) => "info",
            Some(domain::dispatch::LegacyLoadLegStatusCode::Draft) => "secondary",
            Some(domain::dispatch::LegacyLoadLegStatusCode::New) => "success",
            None => "secondary",
        },
    }
}

fn parse_admin_role_key(value: &str) -> Option<UserRole> {
    match value.trim().to_ascii_lowercase().as_str() {
        "admin" => Some(UserRole::Admin),
        "shipper" => Some(UserRole::Shipper),
        "carrier" => Some(UserRole::Carrier),
        "broker" => Some(UserRole::Broker),
        "freight_forwarder" | "freight forwarder" => Some(UserRole::FreightForwarder),
        _ => None,
    }
}

fn admin_role_key(role: UserRole) -> String {
    match role {
        UserRole::Admin => "admin",
        UserRole::Shipper => "shipper",
        UserRole::Carrier => "carrier",
        UserRole::Broker => "broker",
        UserRole::FreightForwarder => "freight_forwarder",
    }
    .into()
}

fn admin_permission_key(permission: domain::auth::Permission) -> &'static str {
    match permission {
        domain::auth::Permission::AccessAdminPortal => "access_admin_portal",
        domain::auth::Permission::ManageUsers => "manage_users",
        domain::auth::Permission::ManageRoles => "manage_roles",
        domain::auth::Permission::ManageMasterData => "manage_master_data",
        domain::auth::Permission::ManageLoads => "manage_loads",
        domain::auth::Permission::ManageDispatchDesk => "manage_dispatch_desk",
        domain::auth::Permission::ManageMarketplace => "manage_marketplace",
        domain::auth::Permission::ManageTracking => "manage_tracking",
        domain::auth::Permission::ManagePayments => "manage_payments",
        domain::auth::Permission::ManageTmsOperations => "manage_tms_operations",
    }
}

fn parse_admin_status_key(value: &str) -> Option<AccountStatus> {
    match value.trim().to_ascii_lowercase().as_str() {
        "email_verified" => Some(AccountStatus::EmailVerifiedPendingOnboarding),
        "approved" => Some(AccountStatus::Approved),
        "rejected" => Some(AccountStatus::Rejected),
        "pending_review" => Some(AccountStatus::PendingReview),
        "pending_otp" => Some(AccountStatus::PendingOtp),
        "revision_requested" => Some(AccountStatus::RevisionRequested),
        _ => None,
    }
}

fn admin_status_key(status: AccountStatus) -> String {
    match status {
        AccountStatus::EmailVerifiedPendingOnboarding => "email_verified",
        AccountStatus::Approved => "approved",
        AccountStatus::Rejected => "rejected",
        AccountStatus::PendingReview => "pending_review",
        AccountStatus::PendingOtp => "pending_otp",
        AccountStatus::RevisionRequested => "revision_requested",
    }
    .into()
}

fn admin_personal_facts(user: &db::auth::UserRecord) -> Vec<AdminUserProfileFact> {
    let mut facts = Vec::new();

    push_profile_fact(&mut facts, "Phone", user.phone_no.clone());
    push_profile_fact(
        &mut facts,
        "Date of birth",
        user.dob.map(|value| value.format("%b %d, %Y").to_string()),
    );
    push_profile_fact(&mut facts, "Gender", user.gender.clone());
    push_profile_fact(&mut facts, "Address", user.address.clone());
    push_profile_fact(&mut facts, "Nationality", user.nationality.clone());
    push_profile_fact(
        &mut facts,
        "Joined",
        Some(format_datetime(&user.created_at)),
    );

    facts
}

fn admin_company_facts(
    user: &db::auth::UserRecord,
    details: Option<&db::auth::UserDetailRecord>,
) -> Vec<AdminUserProfileFact> {
    let mut facts = Vec::new();

    let detail_company_name = details.and_then(|detail| detail.company_name.clone());
    let detail_company_address = details.and_then(|detail| detail.company_address.clone());
    push_profile_fact(
        &mut facts,
        "Company",
        detail_company_name.or(user.company_name.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Company address",
        detail_company_address.or(user.company_address.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Registration number",
        user.registration_number.clone(),
    );
    push_profile_fact(&mut facts, "Tax ID", user.tax_id.clone());
    push_profile_fact(
        &mut facts,
        "Country of incorporation",
        user.country_of_incorporation.clone(),
    );
    push_profile_fact(
        &mut facts,
        "DOT number",
        details.and_then(|detail| detail.dot_number.clone()),
    );
    push_profile_fact(
        &mut facts,
        "MC number",
        details
            .and_then(|detail| detail.mc_number.clone())
            .or(user.mc_number.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Equipment types",
        details.and_then(|detail| detail.equipment_types.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Business entity ID",
        details.and_then(|detail| detail.business_entity_id.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Facility address",
        details.and_then(|detail| detail.facility_address.clone()),
    );
    push_profile_fact(
        &mut facts,
        "Fulfillment contact",
        details.and_then(|detail| detail.fulfillment_contact_info.clone()),
    );
    push_profile_fact(&mut facts, "USDOT", user.usdot_number.clone());
    push_profile_fact(&mut facts, "CDL number", user.cdl_number.clone());
    push_profile_fact(&mut facts, "Vehicle", user.vehicle_make_model.clone());
    push_profile_fact(&mut facts, "Policy number", user.policy_number.clone());
    push_profile_fact(&mut facts, "Transport modes", user.transport_modes.clone());
    push_profile_fact(
        &mut facts,
        "Countries served",
        user.countries_served.clone(),
    );
    push_profile_fact(
        &mut facts,
        "Years in operation",
        user.years_in_operation.clone(),
    );

    facts
}

fn push_profile_fact(facts: &mut Vec<AdminUserProfileFact>, label: &str, value: Option<String>) {
    if let Some(value) = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        facts.push(AdminUserProfileFact {
            label: label.into(),
            value,
        });
    }
}

fn format_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{
        auth_headers_for_user, insert_load_fixture, insert_user_with_role_status, prepare_pool,
        read_leg_status, test_state,
    };
    use domain::auth::{AccountStatus, UserRole};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn review_user_handler_updates_status_and_reports_email_delivery()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Admin Reviewer",
            "admin-reviewer@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let review_target = insert_user_with_role_status(
            &pool,
            "Pending Carrier",
            "pending-carrier@example.com",
            UserRole::Carrier,
            AccountStatus::PendingReview,
        )
        .await?;
        let admin_headers = auth_headers_for_user(&state, &admin_user).await?;

        let response = review_user_handler(
            State(state),
            Path(review_target.id),
            admin_headers,
            Json(ReviewOnboardingRequest {
                decision: "revision".into(),
                remarks: Some("Need clearer carrier compliance detail.".into()),
            }),
        )
        .await
        .expect("admin review user request should succeed")
        .0
        .data;

        assert!(response.success);
        assert_eq!(response.status_label.as_deref(), Some("Revision Requested"));
        assert!(response.message.contains("Email notification logged"));
        let updated = db::auth::find_user_by_id(&pool, review_target.id)
            .await?
            .expect("review target still exists");
        assert_eq!(
            updated.account_status(),
            Some(AccountStatus::RevisionRequested)
        );

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn review_load_handler_updates_legs_and_reports_email_delivery()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(pool) = prepare_pool().await? else {
            return Ok(());
        };
        let state = test_state(pool.clone());
        let admin_user = insert_user_with_role_status(
            &pool,
            "Admin Load Reviewer",
            "admin-load-reviewer@example.com",
            UserRole::Admin,
            AccountStatus::Approved,
        )
        .await?;
        let fixture = insert_load_fixture(&pool, 1).await?;
        let admin_headers = auth_headers_for_user(&state, &admin_user).await?;

        let response = review_load_handler(
            State(state),
            Path(fixture.load_id),
            admin_headers,
            Json(AdminReviewLoadRequest {
                decision: "approve".into(),
                remarks: Some("Approved from backend route acceptance test.".into()),
            }),
        )
        .await
        .expect("admin review load request should succeed")
        .0
        .data;

        assert!(response.success);
        assert_eq!(response.status_label, "Approved");
        assert!(response.message.contains("Email notification logged"));
        assert_eq!(read_leg_status(&pool, fixture.leg_id).await?, 2);

        Ok(())
    }
}
