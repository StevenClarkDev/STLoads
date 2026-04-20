use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use db::auth::{
    ChangeSelfPasswordInput, CreateKycDocumentInput, CreateUserInput, UpdateKycDocumentInput,
    UpdateSelfProfileInput, UpsertUserOnboardingInput, change_self_password,
    consume_password_reset_otp, consume_password_reset_token, consume_registration_otp,
    create_kyc_document, create_registered_user, delete_kyc_document, find_kyc_document_by_id,
    find_user_by_email, find_user_by_id, find_user_detail_by_user_id,
    list_kyc_documents_by_user_id, refresh_user_otp, store_password_reset_token,
    update_kyc_document, update_self_profile, upsert_user_onboarding_details,
    verify_kyc_document_blockchain,
};
use domain::auth::{
    AccountStatus, AccountStatusDescriptor, AuthModuleContract, PermissionDescriptor,
    RoleDescriptor, RolePermissionContract, UserRole, account_status_descriptors,
    auth_module_contract, permission_descriptors, role_descriptors, role_permission_contracts,
};
use serde::Serialize;
use shared::{
    ApiResponse, AuthOnboardingDraft, AuthOnboardingScreen, AuthSessionState,
    ChangePasswordRequest, ChangePasswordResponse, DeleteKycDocumentResponse,
    ForgotPasswordRequest, ForgotPasswordResponse, KycDocumentItem, LoginRequest, LoginResponse,
    LogoutResponse, OtpPurpose, RealtimeEvent, RealtimeEventKind, RegisterRequest,
    RegisterResponse, ResendOtpRequest, ResendOtpResponse, ResetPasswordRequest,
    ResetPasswordResponse, SelfProfileDraft, SelfProfileFact, SelfProfileScreen,
    SubmitOnboardingRequest, SubmitOnboardingResponse, UpdateSelfProfileRequest,
    UpdateSelfProfileResponse, UpsertKycDocumentRequest, UpsertKycDocumentResponse,
    VerifyKycDocumentRequest, VerifyKycDocumentResponse, VerifyOtpRequest, VerifyOtpResponse,
};

use crate::{auth_session, email::MailOutcome, realtime_bus::RoutedRealtimeEvent, state::AppState};

#[derive(Debug, Serialize)]
struct AuthOverview {
    contract: AuthModuleContract,
    roles: Vec<RoleDescriptor>,
    account_statuses: Vec<AccountStatusDescriptor>,
    permissions: usize,
    role_permission_sets: usize,
}

pub fn router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/roles", get(roles))
        .route("/permissions", get(permissions))
        .route("/account-statuses", get(account_statuses))
        .route("/session-contract", get(session_contract))
        .route("/rbac-contract", get(rbac_contract))
        .route("/session", get(session))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/register", post(register))
        .route("/verify-otp", post(verify_otp))
        .route("/otp/resend", post(resend_otp))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/onboarding-screen", get(onboarding_screen))
        .route("/onboarding", post(submit_onboarding))
        .route("/profile-screen", get(profile_screen))
        .route("/profile", post(update_profile))
        .route("/change-password", post(change_password))
        .route(
            "/profile/documents/upload",
            post(upload_profile_kyc_document_handler)
                .layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/profile/documents/{document_id}",
            post(update_profile_kyc_document_handler),
        )
        .route(
            "/profile/documents/{document_id}/upload",
            post(replace_profile_kyc_document_handler)
                .layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/profile/documents/{document_id}/verify-blockchain",
            post(verify_profile_kyc_document_handler),
        )
        .route(
            "/profile/documents/{document_id}/delete",
            post(delete_profile_kyc_document_handler),
        )
        .route(
            "/onboarding/documents/upload",
            post(upload_kyc_document_handler).layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route(
            "/onboarding/documents/{document_id}/file",
            get(download_kyc_document_file),
        )
}

async fn index() -> Json<ApiResponse<AuthOverview>> {
    Json(ApiResponse::ok(AuthOverview {
        contract: auth_module_contract(),
        roles: role_descriptors().to_vec(),
        account_statuses: account_status_descriptors().to_vec(),
        permissions: permission_descriptors().len(),
        role_permission_sets: role_permission_contracts().len(),
    }))
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("auth route group ready"))
}

async fn roles() -> Json<ApiResponse<Vec<RoleDescriptor>>> {
    Json(ApiResponse::ok(role_descriptors().to_vec()))
}

async fn permissions() -> Json<ApiResponse<Vec<PermissionDescriptor>>> {
    Json(ApiResponse::ok(permission_descriptors().to_vec()))
}

async fn account_statuses() -> Json<ApiResponse<Vec<AccountStatusDescriptor>>> {
    Json(ApiResponse::ok(account_status_descriptors().to_vec()))
}

async fn session_contract() -> Json<ApiResponse<AuthModuleContract>> {
    Json(ApiResponse::ok(auth_module_contract()))
}

async fn rbac_contract() -> Json<ApiResponse<Vec<RolePermissionContract>>> {
    Json(ApiResponse::ok(role_permission_contracts().to_vec()))
}

async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<AuthSessionState>> {
    let session = match auth_session::resolve_session_from_headers(&state, &headers).await {
        Ok(Some(resolved)) => resolved.session,
        Ok(None) => auth_session::unauthenticated_session_state(
            "No bearer token was supplied to the Rust auth session endpoint.",
        ),
        Err(error) => auth_session::unauthenticated_session_state(&format!(
            "Failed to resolve the Rust auth session: {}",
            error
        )),
    };

    Json(ApiResponse::ok(session))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "Login is unavailable because the database connection is disabled.",
            ),
            message: format!(
                "Login is unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    let email = payload.email.trim().to_lowercase();
    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "No matching user exists for this email address.",
            ),
            message: "Invalid email or password.".into(),
        }));
    };

    let password_matches = verify(&payload.password, &user.password)
        .unwrap_or_else(|_| payload.password == user.password);

    if !password_matches {
        return Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(
                "Password verification failed in the Rust auth layer.",
            ),
            message: "Invalid email or password.".into(),
        }));
    }

    match auth_session::issue_session_token(&state, &user).await {
        Ok(token) => {
            let session = auth_session::build_session_state(&state, &user).await;
            Json(ApiResponse::ok(LoginResponse {
                success: true,
                token: Some(token),
                session,
                message: login_message_for_status(user.account_status()),
            }))
        }
        Err(error) => Json(ApiResponse::ok(LoginResponse {
            success: false,
            token: None,
            session: auth_session::unauthenticated_session_state(&error),
            message: format!("Login failed: {}", error),
        })),
    }
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<LogoutResponse>> {
    let Some(token) = auth_session::bearer_token_from_headers(&headers) else {
        return Json(ApiResponse::ok(LogoutResponse {
            success: false,
            message: "No bearer token was supplied for logout.".into(),
        }));
    };

    let actor_user_id = auth_session::resolve_session_from_token(&state, &token)
        .await
        .ok()
        .flatten()
        .map(|resolved| resolved.user.id.max(0) as u64);

    match auth_session::revoke_session_token(&state, &token).await {
        Ok(_) => {
            state.publish_realtime(
                RoutedRealtimeEvent::new(RealtimeEvent {
                    kind: RealtimeEventKind::SessionInvalidated,
                    leg_id: None,
                    conversation_id: None,
                    offer_id: None,
                    message_id: None,
                    actor_user_id,
                    subject_user_id: actor_user_id,
                    presence_state: None,
                    last_read_message_id: None,
                    summary: "Session token revoked from the Rust auth route.".into(),
                })
                .for_user_ids(actor_user_id.into_iter()),
            );

            Json(ApiResponse::ok(LogoutResponse {
                success: true,
                message: "Logged out from the Rust session layer.".into(),
            }))
        }
        Err(error) => Json(ApiResponse::ok(LogoutResponse {
            success: false,
            message: format!("Logout failed: {}", error),
        })),
    }
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Json<ApiResponse<RegisterResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email: payload.email,
            role_key: payload.role_key,
            next_step: "/auth/login".into(),
            message:
                "Registration is unavailable because the Rust database connection is disabled."
                    .into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    let email = payload.email.trim().to_lowercase();
    let name = payload.name.trim().to_string();
    let role = match parse_role_key(&payload.role_key) {
        Some(UserRole::Admin) | None => {
            return Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: payload.role_key,
                next_step: "/auth/register".into(),
                message: "Choose a valid business role before continuing.".into(),
                otp_expires_at: None,
                dev_otp: None,
            }));
        }
        Some(role) => role,
    };

    if name.is_empty() || email.is_empty() {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: "Name and email are required for Rust registration.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    if payload.password != payload.password_confirmation {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: "Password confirmation does not match.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    if payload.password.len() < 8 {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: "Use a password with at least 8 characters.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    if find_user_by_email(pool, &email)
        .await
        .unwrap_or(None)
        .is_some()
    {
        return Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email: email.clone(),
            role_key: role_key(role).into(),
            next_step: format!("/auth/login?email={}", email),
            message: "An account with this email already exists.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    }

    let password_hash = match hash(&payload.password, 12) {
        Ok(value) => value,
        Err(error) => {
            return Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: role_key(role).into(),
                next_step: "/auth/register".into(),
                message: format!("Password hashing failed: {}", error),
                otp_expires_at: None,
                dev_otp: None,
            }));
        }
    };

    let otp = generate_otp();
    let otp_expires_at = Utc::now() + Duration::minutes(5);

    let input = CreateUserInput {
        name: name.clone(),
        email: email.clone(),
        password_hash,
        role_id: role.legacy_id(),
        phone_no: payload.phone_no.filter(|value| !value.trim().is_empty()),
        address: payload.address.filter(|value| !value.trim().is_empty()),
        otp: otp.clone(),
        otp_expires_at: otp_expires_at.naive_utc(),
    };

    let mail_outcome = match state
        .email
        .send_registration_otp(&email, Some(&name), &otp)
        .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            return Json(ApiResponse::ok(RegisterResponse {
                success: false,
                email,
                role_key: role_key(role).into(),
                next_step: "/auth/register".into(),
                message: format!("Registration OTP email failed: {}", error),
                otp_expires_at: None,
                dev_otp: None,
            }));
        }
    };

    match create_registered_user(pool, &input).await {
        Ok(_) => Json(ApiResponse::ok(RegisterResponse {
            success: true,
            email: email.clone(),
            role_key: role_key(role).into(),
            next_step: format!(
                "/auth/verify-otp?email={}&purpose={}",
                email,
                OtpPurpose::Registration.as_key()
            ),
            message: mail_outcome.append_to_message(format!(
                "Rust registration created. Verify the 6-digit OTP for {} to continue.",
                email
            )),
            otp_expires_at: Some(otp_expires_at.to_rfc3339()),
            dev_otp: exposed_secret(&state, &otp),
        })),
        Err(error) => Json(ApiResponse::ok(RegisterResponse {
            success: false,
            email,
            role_key: role_key(role).into(),
            next_step: "/auth/register".into(),
            message: format!("Registration failed: {}", error),
            otp_expires_at: None,
            dev_otp: None,
        })),
    }
}

async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Json<ApiResponse<VerifyOtpResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(VerifyOtpResponse {
            success: false,
            email: payload.email,
            purpose: payload.purpose,
            next_step: "/auth/login".into(),
            message:
                "OTP verification is unavailable because the Rust database connection is disabled."
                    .into(),
            token: None,
            session: None,
            reset_token: None,
        }));
    };

    let email = payload.email.trim().to_lowercase();
    let otp = payload.otp.trim().to_string();
    if otp.len() != 6 || !otp.chars().all(|ch| ch.is_ascii_digit()) {
        return Json(ApiResponse::ok(VerifyOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            next_step: format!(
                "/auth/verify-otp?email={}&purpose={}",
                payload.email.trim().to_lowercase(),
                payload.purpose.as_key()
            ),
            message: "Enter the full 6-digit OTP code.".into(),
            token: None,
            session: None,
            reset_token: None,
        }));
    }

    match payload.purpose {
        OtpPurpose::Registration => match consume_registration_otp(
            pool,
            &email,
            &otp,
            AccountStatus::EmailVerifiedPendingOnboarding.legacy_code(),
        )
        .await
        {
            Ok(Some(user)) => match auth_session::issue_session_token(&state, &user).await {
                Ok(token) => {
                    let session = auth_session::build_session_state(&state, &user).await;
                    Json(ApiResponse::ok(VerifyOtpResponse {
                        success: true,
                        email,
                        purpose: OtpPurpose::Registration,
                        next_step: session
                            .user
                            .as_ref()
                            .map(|item| item.dashboard_href.clone())
                            .unwrap_or_else(|| "/auth/onboarding".into()),
                        message: "Email verified. Your Rust session is live and onboarding is the next required step.".into(),
                        token: Some(token),
                        session: Some(session),
                        reset_token: None,
                    }))
                }
                Err(error) => Json(ApiResponse::ok(VerifyOtpResponse {
                    success: false,
                    email,
                    purpose: OtpPurpose::Registration,
                    next_step: "/auth/login".into(),
                    message: format!("OTP verified, but session issuance failed: {}", error),
                    token: None,
                    session: None,
                    reset_token: None,
                })),
            },
            Ok(None) => Json(ApiResponse::ok(VerifyOtpResponse {
                success: false,
                email,
                purpose: OtpPurpose::Registration,
                next_step: format!(
                    "/auth/verify-otp?email={}&purpose={}",
                    payload.email.trim().to_lowercase(),
                    OtpPurpose::Registration.as_key()
                ),
                message: "Invalid or expired OTP.".into(),
                token: None,
                session: None,
                reset_token: None,
            })),
            Err(error) => Json(ApiResponse::ok(VerifyOtpResponse {
                success: false,
                email,
                purpose: OtpPurpose::Registration,
                next_step: "/auth/register".into(),
                message: format!("OTP verification failed: {}", error),
                token: None,
                session: None,
                reset_token: None,
            })),
        },
        OtpPurpose::PasswordReset => match consume_password_reset_otp(pool, &email, &otp).await {
            Ok(Some(_)) => {
                let reset_token = uuid::Uuid::new_v4().to_string();
                match store_password_reset_token(pool, &email, &reset_token).await {
                    Ok(_) => Json(ApiResponse::ok(VerifyOtpResponse {
                        success: true,
                        email,
                        purpose: OtpPurpose::PasswordReset,
                        next_step: format!(
                            "/auth/reset-password?email={}",
                            payload.email.trim().to_lowercase()
                        ),
                        message: "OTP verified. Set a new password in the Rust reset form.".into(),
                        token: None,
                        session: None,
                        reset_token: exposed_secret(&state, &reset_token),
                    })),
                    Err(error) => Json(ApiResponse::ok(VerifyOtpResponse {
                        success: false,
                        email,
                        purpose: OtpPurpose::PasswordReset,
                        next_step: "/auth/forgot-password".into(),
                        message: format!("Failed to issue password reset token: {}", error),
                        token: None,
                        session: None,
                        reset_token: None,
                    })),
                }
            }
            Ok(None) => Json(ApiResponse::ok(VerifyOtpResponse {
                success: false,
                email,
                purpose: OtpPurpose::PasswordReset,
                next_step: format!(
                    "/auth/verify-otp?email={}&purpose={}",
                    payload.email.trim().to_lowercase(),
                    OtpPurpose::PasswordReset.as_key()
                ),
                message: "Invalid or expired OTP.".into(),
                token: None,
                session: None,
                reset_token: None,
            })),
            Err(error) => Json(ApiResponse::ok(VerifyOtpResponse {
                success: false,
                email,
                purpose: OtpPurpose::PasswordReset,
                next_step: "/auth/forgot-password".into(),
                message: format!("OTP verification failed: {}", error),
                token: None,
                session: None,
                reset_token: None,
            })),
        },
    }
}

async fn resend_otp(
    State(state): State<AppState>,
    Json(payload): Json<ResendOtpRequest>,
) -> Json<ApiResponse<ResendOtpResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ResendOtpResponse {
            success: false,
            email: payload.email,
            purpose: payload.purpose,
            message: "OTP resend is unavailable because the Rust database connection is disabled."
                .into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    let email = payload.email.trim().to_lowercase();
    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        return Json(ApiResponse::ok(ResendOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            message: "No Rust user was found for this email address.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    match next_resend_count(user.last_otp_resend_at, user.otp_resend_count) {
        Ok(resend_count) => {
            let otp = generate_otp();
            let otp_expires_at = Utc::now() + Duration::minutes(5);
            match refresh_user_otp(
                pool,
                user.id,
                &otp,
                otp_expires_at.naive_utc(),
                resend_count,
            )
            .await
            {
                Ok(_) => {
                    let mail_outcome = match send_otp_notification(
                        &state,
                        payload.purpose,
                        &email,
                        Some(&user.name),
                        &otp,
                    )
                    .await
                    {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            return Json(ApiResponse::ok(ResendOtpResponse {
                                success: false,
                                email,
                                purpose: payload.purpose,
                                message: format!(
                                    "OTP was refreshed, but email delivery failed: {}",
                                    error
                                ),
                                otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                                dev_otp: exposed_secret(&state, &otp),
                            }));
                        }
                    };

                    Json(ApiResponse::ok(ResendOtpResponse {
                        success: true,
                        email,
                        purpose: payload.purpose,
                        message: mail_outcome.append_to_message(resend_message(payload.purpose)),
                        otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                        dev_otp: exposed_secret(&state, &otp),
                    }))
                }
                Err(error) => Json(ApiResponse::ok(ResendOtpResponse {
                    success: false,
                    email,
                    purpose: payload.purpose,
                    message: format!("OTP resend failed: {}", error),
                    otp_expires_at: None,
                    dev_otp: None,
                })),
            }
        }
        Err(message) => Json(ApiResponse::ok(ResendOtpResponse {
            success: false,
            email,
            purpose: payload.purpose,
            message,
            otp_expires_at: None,
            dev_otp: None,
        })),
    }
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Json<ApiResponse<ForgotPasswordResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ForgotPasswordResponse {
            success: false,
            email: payload.email,
            next_step: "/auth/login".into(),
            message:
                "Password reset is unavailable because the Rust database connection is disabled."
                    .into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    let email = payload.email.trim().to_lowercase();
    let Some(user) = find_user_by_email(pool, &email).await.unwrap_or(None) else {
        return Json(ApiResponse::ok(ForgotPasswordResponse {
            success: false,
            email,
            next_step: "/auth/forgot-password".into(),
            message: "No Rust user was found for this email address.".into(),
            otp_expires_at: None,
            dev_otp: None,
        }));
    };

    match next_resend_count(user.last_otp_resend_at, user.otp_resend_count) {
        Ok(resend_count) => {
            let otp = generate_otp();
            let otp_expires_at = Utc::now() + Duration::minutes(5);
            match refresh_user_otp(
                pool,
                user.id,
                &otp,
                otp_expires_at.naive_utc(),
                resend_count,
            )
            .await
            {
                Ok(_) => {
                    let mail_outcome = match state
                        .email
                        .send_password_reset_otp(&email, Some(&user.name), &otp)
                        .await
                    {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            return Json(ApiResponse::ok(ForgotPasswordResponse {
                                success: false,
                                email,
                                next_step: "/auth/forgot-password".into(),
                                message: format!(
                                    "Password reset OTP was refreshed, but email delivery failed: {}",
                                    error
                                ),
                                otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                                dev_otp: exposed_secret(&state, &otp),
                            }));
                        }
                    };

                    Json(ApiResponse::ok(ForgotPasswordResponse {
                        success: true,
                        email: email.clone(),
                        next_step: format!(
                            "/auth/verify-otp?email={}&purpose={}",
                            email,
                            OtpPurpose::PasswordReset.as_key()
                        ),
                        message: mail_outcome.append_to_message(
                            "A password reset OTP is ready for verification in the Rust auth flow.",
                        ),
                        otp_expires_at: Some(otp_expires_at.to_rfc3339()),
                        dev_otp: exposed_secret(&state, &otp),
                    }))
                }
                Err(error) => Json(ApiResponse::ok(ForgotPasswordResponse {
                    success: false,
                    email,
                    next_step: "/auth/forgot-password".into(),
                    message: format!("Failed to issue a password reset OTP: {}", error),
                    otp_expires_at: None,
                    dev_otp: None,
                })),
            }
        }
        Err(message) => Json(ApiResponse::ok(ForgotPasswordResponse {
            success: false,
            email,
            next_step: "/auth/forgot-password".into(),
            message,
            otp_expires_at: None,
            dev_otp: None,
        })),
    }
}

async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Json<ApiResponse<ResetPasswordResponse>> {
    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email: payload.email,
            next_step: "/auth/login".into(),
            message:
                "Password reset is unavailable because the Rust database connection is disabled."
                    .into(),
        }));
    };

    let email = payload.email.trim().to_lowercase();
    if payload.password != payload.password_confirmation {
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: format!(
                "/auth/reset-password?email={}",
                payload.email.trim().to_lowercase()
            ),
            message: "Password confirmation does not match.".into(),
        }));
    }

    if payload.password.len() < 8 {
        return Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: format!(
                "/auth/reset-password?email={}",
                payload.email.trim().to_lowercase()
            ),
            message: "Use a password with at least 8 characters.".into(),
        }));
    }

    let password_hash = match hash(&payload.password, 12) {
        Ok(value) => value,
        Err(error) => {
            return Json(ApiResponse::ok(ResetPasswordResponse {
                success: false,
                email,
                next_step: format!(
                    "/auth/reset-password?email={}",
                    payload.email.trim().to_lowercase()
                ),
                message: format!("Password hashing failed: {}", error),
            }));
        }
    };

    match consume_password_reset_token(pool, &email, payload.reset_token.trim(), &password_hash)
        .await
    {
        Ok(true) => Json(ApiResponse::ok(ResetPasswordResponse {
            success: true,
            email,
            next_step: format!("/auth/login?email={}", payload.email.trim().to_lowercase()),
            message: "Password updated in the Rust auth flow. Sign in with the new password."
                .into(),
        })),
        Ok(false) => Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: format!(
                "/auth/reset-password?email={}",
                payload.email.trim().to_lowercase()
            ),
            message: "The reset token is invalid or expired.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ResetPasswordResponse {
            success: false,
            email,
            next_step: "/auth/forgot-password".into(),
            message: format!("Password reset failed: {}", error),
        })),
    }
}

async fn onboarding_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<AuthOnboardingScreen>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(unauthenticated_onboarding_screen()));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(onboarding_screen_from_user(
            &resolved.user,
            None,
            Vec::new(),
            false,
            vec!["Rust onboarding is waiting for the database connection to come back.".into()],
        )));
    };

    let details = find_user_detail_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or(None);

    let documents = list_kyc_documents_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or_default();
    let can_submit = can_submit_onboarding(resolved.user.account_status());
    let mut notes = onboarding_notes_for_user(&resolved.user);
    notes.push(
        "KYC uploads now use the Rust secure document path, and files are viewable only by admin users plus the profile that uploaded them."
            .into(),
    );

    Json(ApiResponse::ok(onboarding_screen_from_user(
        &resolved.user,
        details,
        documents,
        can_submit,
        notes,
    )))
}

async fn submit_onboarding(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SubmitOnboardingRequest>,
) -> Json<ApiResponse<SubmitOnboardingResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: None,
            next_step: "/auth/login".into(),
            message: "Sign in before submitting Rust onboarding.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: None,
            next_step: "/auth/onboarding".into(),
            message:
                "Rust onboarding cannot save right now because the database connection is disabled."
                    .into(),
        }));
    };

    if resolved.user.email_verified_at.is_none() {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/verify-otp".into(),
            message: "Verify the account OTP before continuing into onboarding.".into(),
        }));
    }

    if !can_submit_onboarding(resolved.user.account_status()) {
        let next_step = resolved
            .session
            .user
            .as_ref()
            .map(|item| item.dashboard_href.clone())
            .unwrap_or_else(|| "/".into());
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step,
            message: "This account is not currently editable through the Rust onboarding form."
                .into(),
        }));
    }

    if payload.company_name.trim().is_empty() || payload.company_address.trim().is_empty() {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message:
                "Company name and company address are required before Rust onboarding can continue."
                    .into(),
        }));
    }

    let role = resolved.user.primary_role();
    if let Some(validation_message) = validate_role_specific_onboarding(role, &payload) {
        return Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message: validation_message,
        }));
    }

    let input = UpsertUserOnboardingInput {
        user_id: resolved.user.id,
        company_name: payload.company_name.trim().to_string(),
        company_address: payload.company_address.trim().to_string(),
        dot_number: optional_owned(&payload.dot_number),
        mc_number: optional_owned(&payload.mc_number),
        equipment_types: optional_owned(&payload.equipment_types),
        business_entity_id: optional_owned(&payload.business_entity_id),
        facility_address: optional_owned(&payload.facility_address),
        fulfillment_contact_info: optional_owned(&payload.fulfillment_contact_info),
        fmcsa_broker_license_no: optional_owned(&payload.fmcsa_broker_license_no),
        mc_authority_number: optional_owned(&payload.mc_authority_number),
        freight_forwarder_license: optional_owned(&payload.freight_forwarder_license),
        customs_license: optional_owned(&payload.customs_license),
        next_status: AccountStatus::PendingReview.legacy_code(),
    };

    match upsert_user_onboarding_details(pool, &input).await {
        Ok(()) => {
            let refreshed_user = find_user_by_id(pool, resolved.user.id)
                .await
                .ok()
                .flatten()
                .unwrap_or(resolved.user);
            let session = auth_session::build_session_state(&state, &refreshed_user).await;
            Json(ApiResponse::ok(SubmitOnboardingResponse {
                success: true,
                session: Some(session),
                next_step: "/".into(),
                message: "Rust onboarding submitted. The account is now pending admin review."
                    .into(),
            }))
        }
        Err(error) => Json(ApiResponse::ok(SubmitOnboardingResponse {
            success: false,
            session: Some(resolved.session),
            next_step: "/auth/onboarding".into(),
            message: format!("Onboarding save failed: {}", error),
        })),
    }
}

async fn profile_screen(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ApiResponse<SelfProfileScreen>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(unauthenticated_profile_screen()));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(SelfProfileScreen {
            title: "My Profile".into(),
            role_key: resolved
                .user
                .primary_role()
                .map(role_key)
                .unwrap_or("unknown")
                .into(),
            role_label: resolved
                .user
                .primary_role()
                .map(|role| role.label().to_string())
                .unwrap_or_else(|| "Unknown".into()),
            status_label: resolved
                .user
                .account_status()
                .map(account_status_display)
                .unwrap_or_else(|| "Unknown".into()),
            draft: self_profile_draft_from_user(&resolved.user, None),
            personal_facts: self_profile_personal_facts(&resolved.user),
            company_facts: self_profile_company_facts(&resolved.user, None),
            documents: Vec::new(),
            notes: vec![format!(
                "Profile data is read-only right now because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            )],
        }));
    };

    let details = find_user_detail_by_user_id(pool, resolved.user.id)
        .await
        .ok()
        .flatten();
    let documents = list_kyc_documents_by_user_id(pool, resolved.user.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|document| build_kyc_document_item(document, true))
        .collect::<Vec<_>>();

    Json(ApiResponse::ok(profile_screen_from_user(
        &resolved.user,
        details,
        documents,
    )))
}

async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateSelfProfileRequest>,
) -> Json<ApiResponse<UpdateSelfProfileResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Sign in before editing the Rust profile.".into(),
            session: None,
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: format!(
                "Profile updates are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
            session: Some(resolved.session),
        }));
    };

    if payload.name.trim().is_empty() || payload.email.trim().is_empty() {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Name and email are required before saving the Rust profile.".into(),
            session: Some(resolved.session),
        }));
    }

    if let Some(existing_user) = find_user_by_email(pool, payload.email.trim())
        .await
        .unwrap_or(None)
    {
        if existing_user.id != resolved.user.id {
            return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                success: false,
                message: "Another account already uses that email address.".into(),
                session: Some(resolved.session),
            }));
        }
    }

    let password_hash = match (
        optional_owned(&payload.password),
        optional_owned(&payload.password_confirmation),
    ) {
        (Some(password), Some(password_confirmation)) => {
            if password != password_confirmation {
                return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                    success: false,
                    message: "Password confirmation does not match.".into(),
                    session: Some(resolved.session),
                }));
            }
            if password.len() < 8 {
                return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                    success: false,
                    message: "Use a password with at least 8 characters.".into(),
                    session: Some(resolved.session),
                }));
            }

            match hash(&password, 12) {
                Ok(value) => Some(value),
                Err(error) => {
                    return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                        success: false,
                        message: format!("Password hashing failed: {}", error),
                        session: Some(resolved.session),
                    }));
                }
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            return Json(ApiResponse::ok(UpdateSelfProfileResponse {
                success: false,
                message: "Enter both password fields before saving a new password.".into(),
                session: Some(resolved.session),
            }));
        }
        (None, None) => None,
    };

    let role = resolved.user.primary_role();
    if matches!(role, Some(UserRole::Carrier))
        && (optional_owned(&payload.dot_number).is_none()
            || optional_owned(&payload.mc_number).is_none())
    {
        return Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "Carrier profiles need DOT and MC numbers in the Rust edit flow.".into(),
            session: Some(resolved.session),
        }));
    }

    let input = UpdateSelfProfileInput {
        user_id: resolved.user.id,
        name: payload.name.trim().to_string(),
        email: payload.email.trim().to_lowercase(),
        phone_no: optional_owned(&payload.phone_no),
        address: optional_owned(&payload.address),
        company_name: optional_owned(&payload.company_name),
        dot_number: optional_owned(&payload.dot_number),
        mc_number: optional_owned(&payload.mc_number),
        mc_cbsa_usdot_no: optional_owned(&payload.mc_cbsa_usdot_no),
        ucr_hcc_no: optional_owned(&payload.ucr_hcc_no),
        password_hash,
        status: resolved.user.status,
    };

    match update_self_profile(pool, &input).await {
        Ok(Some(user)) => {
            let session = auth_session::build_session_state(&state, &user).await;
            Json(ApiResponse::ok(UpdateSelfProfileResponse {
                success: true,
                message: "Rust profile updated successfully.".into(),
                session: Some(session),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: "The selected Rust profile could not be updated.".into(),
            session: Some(resolved.session),
        })),
        Err(error) => Json(ApiResponse::ok(UpdateSelfProfileResponse {
            success: false,
            message: format!("Profile update failed: {}", error),
            session: Some(resolved.session),
        })),
    }
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Json<ApiResponse<ChangePasswordResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Sign in before changing your Rust password.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: format!(
                "Password changes are unavailable because the database is {} on {}.",
                state.database_state(),
                state.config.deployment_target
            ),
        }));
    };

    if payload.current_password.is_empty() {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Enter your current password before saving a new one.".into(),
        }));
    }

    if payload.password != payload.password_confirmation {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Password confirmation does not match.".into(),
        }));
    }

    if payload.password.len() < 8 {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Use a password with at least 8 characters.".into(),
        }));
    }

    let current_password_matches = verify(&payload.current_password, &resolved.user.password)
        .unwrap_or_else(|_| payload.current_password == resolved.user.password);

    if !current_password_matches {
        return Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "Current password is incorrect.".into(),
        }));
    }

    let password_hash = match hash(&payload.password, 12) {
        Ok(value) => value,
        Err(error) => {
            return Json(ApiResponse::ok(ChangePasswordResponse {
                success: false,
                message: format!("Password hashing failed: {}", error),
            }));
        }
    };

    match change_self_password(
        pool,
        &ChangeSelfPasswordInput {
            user_id: resolved.user.id,
            password_hash,
            status: resolved.user.status,
            remarks: Some("Password changed through the Rust account security page.".into()),
        },
    )
    .await
    {
        Ok(Some(_)) => Json(ApiResponse::ok(ChangePasswordResponse {
            success: true,
            message: "Rust password updated successfully.".into(),
        })),
        Ok(None) => Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: "The current account could not be updated.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(ChangePasswordResponse {
            success: false,
            message: format!("Password update failed: {}", error),
        })),
    }
}

fn parse_role_key(value: &str) -> Option<UserRole> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "admin" => Some(UserRole::Admin),
        "shipper" => Some(UserRole::Shipper),
        "carrier" => Some(UserRole::Carrier),
        "broker" => Some(UserRole::Broker),
        "freight_forwarder" | "freight forwarder" => Some(UserRole::FreightForwarder),
        _ => None,
    }
}

fn role_key(role: UserRole) -> &'static str {
    match role {
        UserRole::Admin => "admin",
        UserRole::Shipper => "shipper",
        UserRole::Carrier => "carrier",
        UserRole::Broker => "broker",
        UserRole::FreightForwarder => "freight_forwarder",
    }
}

fn unauthenticated_profile_screen() -> SelfProfileScreen {
    SelfProfileScreen {
        title: "My Profile".into(),
        role_key: "unknown".into(),
        role_label: "Unknown".into(),
        status_label: "Signed out".into(),
        draft: SelfProfileDraft {
            name: String::new(),
            email: String::new(),
            phone_no: None,
            address: None,
            company_name: None,
            dot_number: None,
            mc_number: None,
            mc_cbsa_usdot_no: None,
            ucr_hcc_no: None,
        },
        personal_facts: Vec::new(),
        company_facts: Vec::new(),
        documents: Vec::new(),
        notes: vec!["Sign in first so the Rust profile can load your account.".into()],
    }
}

fn profile_screen_from_user(
    user: &db::auth::UserRecord,
    details: Option<db::auth::UserDetailRecord>,
    documents: Vec<KycDocumentItem>,
) -> SelfProfileScreen {
    let role = user.primary_role();
    let role_key_value = role.map(role_key).unwrap_or("unknown").to_string();
    SelfProfileScreen {
        title: "My Profile".into(),
        role_key: role_key_value,
        role_label: role
            .map(|value| value.label().to_string())
            .unwrap_or_else(|| "Unknown".into()),
        status_label: user
            .account_status()
            .map(account_status_display)
            .unwrap_or_else(|| "Unknown".into()),
        draft: self_profile_draft_from_user(user, details.as_ref()),
        personal_facts: self_profile_personal_facts(user),
        company_facts: self_profile_company_facts(user, details.as_ref()),
        documents,
        notes: vec![
            "This Rust profile replaces the old read-only and edit profile Blade pages with one self-serve workspace.".into(),
            "Admins can still inspect the deeper compliance view from the Rust admin directory when needed.".into(),
        ],
    }
}

fn self_profile_draft_from_user(
    user: &db::auth::UserRecord,
    details: Option<&db::auth::UserDetailRecord>,
) -> shared::SelfProfileDraft {
    shared::SelfProfileDraft {
        name: user.name.clone(),
        email: user.email.clone(),
        phone_no: user.phone_no.clone(),
        address: user.address.clone(),
        company_name: details
            .and_then(|item| item.company_name.clone())
            .or_else(|| user.company_name.clone()),
        dot_number: details.and_then(|item| item.dot_number.clone()),
        mc_number: details
            .and_then(|item| item.mc_number.clone())
            .or_else(|| user.mc_number.clone()),
        mc_cbsa_usdot_no: user.mc_cbsa_usdot_no.clone(),
        ucr_hcc_no: user.ucr_hcc_no.clone(),
    }
}

fn self_profile_personal_facts(user: &db::auth::UserRecord) -> Vec<SelfProfileFact> {
    let mut facts = Vec::new();
    push_self_profile_fact(&mut facts, "Email", Some(user.email.clone()));
    push_self_profile_fact(&mut facts, "Phone", user.phone_no.clone());
    push_self_profile_fact(&mut facts, "Address", user.address.clone());
    push_self_profile_fact(
        &mut facts,
        "Date of birth",
        user.dob.map(|value| value.format("%Y-%m-%d").to_string()),
    );
    push_self_profile_fact(&mut facts, "Gender", user.gender.clone());
    push_self_profile_fact(
        &mut facts,
        "Member since",
        Some(format_profile_datetime(&user.created_at)),
    );
    facts
}

fn self_profile_company_facts(
    user: &db::auth::UserRecord,
    details: Option<&db::auth::UserDetailRecord>,
) -> Vec<SelfProfileFact> {
    let mut facts = Vec::new();
    push_self_profile_fact(
        &mut facts,
        "Company",
        details
            .and_then(|item| item.company_name.clone())
            .or_else(|| user.company_name.clone()),
    );
    push_self_profile_fact(
        &mut facts,
        "DOT number",
        details.and_then(|item| item.dot_number.clone()),
    );
    push_self_profile_fact(
        &mut facts,
        "MC number",
        details
            .and_then(|item| item.mc_number.clone())
            .or_else(|| user.mc_number.clone()),
    );
    push_self_profile_fact(&mut facts, "MC/CBSA/USDOT", user.mc_cbsa_usdot_no.clone());
    push_self_profile_fact(&mut facts, "UCR/HCC", user.ucr_hcc_no.clone());
    facts
}

fn push_self_profile_fact(facts: &mut Vec<SelfProfileFact>, label: &str, value: Option<String>) {
    if let Some(value) = value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
    {
        facts.push(SelfProfileFact {
            label: label.into(),
            value,
        });
    }
}

fn generate_otp() -> String {
    let seed = Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_else(|| Utc::now().timestamp_micros() * 1000)
        .unsigned_abs();
    let value = 100_000 + (seed % 900_000);
    format!("{:06}", value)
}

fn next_resend_count(
    last_otp_resend_at: Option<chrono::NaiveDateTime>,
    current_count: i32,
) -> Result<i32, String> {
    let now = Utc::now().naive_utc();
    let base_count = match last_otp_resend_at {
        Some(last_sent_at) if now - last_sent_at < Duration::hours(1) => current_count,
        _ => 0,
    };

    if base_count >= 5 {
        Err("OTP resend limit reached. Try again after an hour.".into())
    } else {
        Ok(base_count + 1)
    }
}

fn exposed_secret(state: &AppState, value: &str) -> Option<String> {
    if state.config.environment.eq_ignore_ascii_case("production") {
        None
    } else {
        Some(value.to_string())
    }
}

fn resend_message(purpose: OtpPurpose) -> String {
    match purpose {
        OtpPurpose::Registration => {
            "A fresh registration OTP is ready in the Rust auth flow.".into()
        }
        OtpPurpose::PasswordReset => {
            "A fresh password reset OTP is ready in the Rust auth flow.".into()
        }
    }
}

async fn send_otp_notification(
    state: &AppState,
    purpose: OtpPurpose,
    email: &str,
    name: Option<&str>,
    otp: &str,
) -> Result<MailOutcome, String> {
    match purpose {
        OtpPurpose::Registration => state.email.send_registration_otp(email, name, otp).await,
        OtpPurpose::PasswordReset => state.email.send_password_reset_otp(email, name, otp).await,
    }
}

fn login_message_for_status(status: Option<AccountStatus>) -> String {
    match status {
        Some(AccountStatus::PendingOtp) => {
            "Logged in, but OTP verification is still pending for this account.".into()
        }
        Some(AccountStatus::EmailVerifiedPendingOnboarding) => {
            "Logged in. Email is verified and onboarding is the next Rust migration slice.".into()
        }
        Some(AccountStatus::PendingReview) => {
            "Logged in. This account is waiting for admin review.".into()
        }
        Some(AccountStatus::RevisionRequested) => {
            "Logged in. Admin requested onboarding revisions for this account.".into()
        }
        Some(AccountStatus::Rejected) => {
            "Logged in, but this account is currently rejected in the legacy status model.".into()
        }
        _ => "Logged in through the Rust session layer.".into(),
    }
}

fn unauthenticated_onboarding_screen() -> AuthOnboardingScreen {
    AuthOnboardingScreen {
        title: "Rust Onboarding".into(),
        subtitle: "Continue the account setup after OTP verification.".into(),
        role_key: "unknown".into(),
        role_label: "Unknown".into(),
        status_label: "Signed out".into(),
        can_submit: false,
        requires_otp: true,
        draft: empty_onboarding_draft(),
        documents: Vec::new(),
        required_fields: vec!["company_name".into(), "company_address".into()],
        notes: vec!["Sign in first so the Rust onboarding flow can load your account.".into()],
    }
}

fn onboarding_screen_from_user(
    user: &db::auth::UserRecord,
    details: Option<db::auth::UserDetailRecord>,
    documents: Vec<db::auth::KycDocumentRecord>,
    can_submit: bool,
    notes: Vec<String>,
) -> AuthOnboardingScreen {
    let role = user.primary_role();
    let required_fields = required_onboarding_fields(role);
    let draft = AuthOnboardingDraft {
        company_name: details
            .as_ref()
            .and_then(|item| item.company_name.clone())
            .or_else(|| user.company_name.clone()),
        company_address: details
            .as_ref()
            .and_then(|item| item.company_address.clone())
            .or_else(|| user.company_address.clone()),
        dot_number: details.as_ref().and_then(|item| item.dot_number.clone()),
        mc_number: details.as_ref().and_then(|item| item.mc_number.clone()),
        equipment_types: details
            .as_ref()
            .and_then(|item| item.equipment_types.clone()),
        business_entity_id: details
            .as_ref()
            .and_then(|item| item.business_entity_id.clone()),
        facility_address: details
            .as_ref()
            .and_then(|item| item.facility_address.clone()),
        fulfillment_contact_info: details
            .as_ref()
            .and_then(|item| item.fulfillment_contact_info.clone()),
        fmcsa_broker_license_no: details
            .as_ref()
            .and_then(|item| item.fmcsa_broker_license_no.clone()),
        mc_authority_number: details
            .as_ref()
            .and_then(|item| item.mc_authority_number.clone()),
        freight_forwarder_license: details
            .as_ref()
            .and_then(|item| item.freight_forwarder_license.clone()),
        customs_license: details
            .as_ref()
            .and_then(|item| item.customs_license.clone()),
    };

    AuthOnboardingScreen {
        title: "Rust Onboarding".into(),
        subtitle: "Complete the company profile that follows OTP verification before the account enters review.".into(),
        role_key: role.map(role_key).unwrap_or("unknown").into(),
        role_label: role.map(|item| item.label().to_string()).unwrap_or_else(|| "Unknown".into()),
        status_label: user
            .account_status()
            .map(account_status_display)
            .unwrap_or_else(|| "Unknown".into()),
        can_submit,
        requires_otp: user.email_verified_at.is_none(),
        draft,
        documents: documents
            .into_iter()
            .map(|document| build_kyc_document_item(document, true))
            .collect(),
        required_fields,
        notes,
    }
}

fn empty_onboarding_draft() -> AuthOnboardingDraft {
    AuthOnboardingDraft {
        company_name: None,
        company_address: None,
        dot_number: None,
        mc_number: None,
        equipment_types: None,
        business_entity_id: None,
        facility_address: None,
        fulfillment_contact_info: None,
        fmcsa_broker_license_no: None,
        mc_authority_number: None,
        freight_forwarder_license: None,
        customs_license: None,
    }
}

fn can_submit_onboarding(status: Option<AccountStatus>) -> bool {
    matches!(
        status,
        Some(AccountStatus::EmailVerifiedPendingOnboarding | AccountStatus::RevisionRequested)
    )
}

fn onboarding_notes_for_user(user: &db::auth::UserRecord) -> Vec<String> {
    let mut notes = Vec::new();
    match user.account_status() {
        Some(AccountStatus::PendingOtp) => notes.push(
            "OTP verification is still pending, so the onboarding form stays locked until that step completes."
                .into(),
        ),
        Some(AccountStatus::EmailVerifiedPendingOnboarding) => notes.push(
            "OTP is complete. Submit this form to move the account into pending review."
                .into(),
        ),
        Some(AccountStatus::RevisionRequested) => notes.push(
            "Admin requested revisions, so this Rust onboarding form is open for another submission cycle."
                .into(),
        ),
        Some(AccountStatus::PendingReview) => notes.push(
            "This account is already pending review, so the form is now read-only until admin feedback arrives."
                .into(),
        ),
        Some(AccountStatus::Approved) => notes.push(
            "This account is already approved, so onboarding values are shown for reference only."
                .into(),
        ),
        Some(AccountStatus::Rejected) => notes.push(
            "This account is currently rejected in the legacy status model.".into(),
        ),
        None => notes.push("The account status could not be mapped from the legacy schema.".into()),
    }
    notes
}

fn required_onboarding_fields(role: Option<UserRole>) -> Vec<String> {
    let mut fields = vec!["company_name".into(), "company_address".into()];
    match role {
        Some(UserRole::Carrier) => {
            fields.push("dot_number".into());
            fields.push("mc_number".into());
            fields.push("equipment_types".into());
        }
        Some(UserRole::Shipper) => {
            fields.push("business_entity_id".into());
            fields.push("facility_address".into());
            fields.push("fulfillment_contact_info".into());
        }
        Some(UserRole::Broker) => {
            fields.push("fmcsa_broker_license_no".into());
            fields.push("mc_authority_number".into());
        }
        Some(UserRole::FreightForwarder) => {
            fields.push("freight_forwarder_license".into());
            fields.push("customs_license".into());
        }
        _ => {}
    }
    fields
}

fn validate_role_specific_onboarding(
    role: Option<UserRole>,
    payload: &SubmitOnboardingRequest,
) -> Option<String> {
    match role {
        Some(UserRole::Carrier)
            if optional_owned(&payload.dot_number).is_none()
                || optional_owned(&payload.mc_number).is_none()
                || optional_owned(&payload.equipment_types).is_none() =>
        {
            Some("Carrier onboarding requires DOT number, MC number, and equipment types.".into())
        }
        Some(UserRole::Shipper)
            if optional_owned(&payload.business_entity_id).is_none()
                || optional_owned(&payload.facility_address).is_none()
                || optional_owned(&payload.fulfillment_contact_info).is_none() =>
        {
            Some("Shipper onboarding requires business entity id, facility address, and fulfillment contact info.".into())
        }
        Some(UserRole::Broker)
            if optional_owned(&payload.fmcsa_broker_license_no).is_none()
                || optional_owned(&payload.mc_authority_number).is_none() =>
        {
            Some("Broker onboarding requires FMCSA broker license number and MC authority number.".into())
        }
        Some(UserRole::FreightForwarder)
            if optional_owned(&payload.freight_forwarder_license).is_none()
                || optional_owned(&payload.customs_license).is_none() =>
        {
            Some("Freight forwarder onboarding requires freight forwarder and customs license values.".into())
        }
        _ => None,
    }
}

fn optional_owned(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn account_status_display(status: AccountStatus) -> String {
    match status {
        AccountStatus::EmailVerifiedPendingOnboarding => "Email Verified".into(),
        AccountStatus::Approved => "Approved".into(),
        AccountStatus::Rejected => "Rejected".into(),
        AccountStatus::PendingReview => "Pending Review".into(),
        AccountStatus::PendingOtp => "Pending OTP".into(),
        AccountStatus::RevisionRequested => "Revision Requested".into(),
    }
}

fn build_kyc_document_item(
    document: db::auth::KycDocumentRecord,
    can_edit: bool,
) -> KycDocumentItem {
    let file_label = document
        .original_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| profile_file_label(&document.file_path));
    let blockchain_label = if document.document_type.eq_ignore_ascii_case("blockchain") {
        Some(
            document
                .mock_blockchain_tx
                .as_ref()
                .map(|_| "Anchored to blockchain".to_string())
                .unwrap_or_else(|| "Blockchain requested".into()),
        )
    } else {
        None
    };
    let blockchain_tone = if document.document_type.eq_ignore_ascii_case("blockchain") {
        Some(
            if document.hash.is_some() {
                "success"
            } else {
                "warning"
            }
            .into(),
        )
    } else {
        None
    };
    let blockchain_hash_preview = document.hash.as_ref().map(|hash| {
        if hash.len() > 24 {
            format!("{}...", &hash[..24])
        } else {
            hash.clone()
        }
    });

    KycDocumentItem {
        id: document.id.max(0) as u64,
        document_name: document.document_name,
        document_type: document.document_type,
        file_label,
        original_name: document.original_name,
        mime_type: document.mime_type,
        file_size_bytes: document
            .file_size
            .and_then(|value| if value >= 0 { Some(value as u64) } else { None }),
        uploaded_at_label: format_profile_datetime(&document.created_at),
        download_path: Some(kyc_document_download_path(document.id.max(0) as u64)),
        can_view_file: true,
        blockchain_label,
        blockchain_tone,
        blockchain_hash_preview,
        blockchain_hash: document.hash.clone(),
        can_edit,
        can_verify_blockchain: can_edit && !document.hash.is_some(),
        can_delete: can_edit,
    }
}

fn validate_kyc_document_type(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "standard" | "blockchain" => Some(normalized),
        _ => None,
    }
}

fn validate_profile_kyc_payload(
    payload: &UpsertKycDocumentRequest,
) -> Result<UpsertKycDocumentRequest, String> {
    let document_name = payload.document_name.trim().to_string();
    if document_name.is_empty() {
        return Err("Enter a document name before saving this KYC row.".into());
    }

    let Some(document_type) = validate_kyc_document_type(&payload.document_type) else {
        return Err("Choose either standard or blockchain for this KYC row.".into());
    };

    Ok(UpsertKycDocumentRequest {
        document_name,
        document_type,
    })
}

async fn upload_profile_kyc_document_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before updating profile KYC documents.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Profile KYC document actions are unavailable because the database connection is disabled.",
        );
    };

    let parsed = match parse_kyc_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => return text_response(StatusCode::BAD_REQUEST, &message),
    };

    let Some(document_type) = validate_kyc_document_type(&parsed.document_type) else {
        return text_response(
            StatusCode::BAD_REQUEST,
            "Choose either standard or blockchain before uploading a profile document.",
        );
    };

    let saved = match state
        .document_storage
        .save_kyc_document(resolved.user.id, &parsed.original_name, &parsed.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC document storage failed: {}", error),
            );
        }
    };

    let mut record = match create_kyc_document(
        pool,
        &CreateKycDocumentInput {
            user_id: resolved.user.id,
            document_name: parsed.document_name.clone(),
            document_type: document_type.clone(),
            file_path: saved.file_path,
            original_name: Some(parsed.original_name.clone()),
            mime_type: parsed.mime_type.clone(),
            file_size: Some(parsed.bytes.len() as i64),
        },
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC document persistence failed: {}", error),
            );
        }
    };

    let pending_review = AccountStatus::PendingReview.legacy_code();
    let updated = match update_kyc_document(
        pool,
        record.id,
        resolved.user.id,
        &UpdateKycDocumentInput {
            document_name: record.document_name.clone(),
            document_type: document_type.clone(),
            file_path: None,
            original_name: None,
            mime_type: None,
            file_size: None,
            next_status: pending_review,
        },
    )
    .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return text_response(
                StatusCode::NOT_FOUND,
                "The uploaded KYC document could not be refreshed after save.",
            );
        }
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC review handoff failed: {}", error),
            );
        }
    };
    record = updated;

    if document_type == "blockchain" {
        record = match verify_kyc_document_blockchain(
            pool,
            record.id,
            resolved.user.id,
            Some("Anchored immediately after Rust profile upload."),
            pending_review,
        )
        .await
        {
            Ok(Some(value)) => value,
            Ok(None) => {
                return text_response(
                    StatusCode::NOT_FOUND,
                    "The uploaded KYC document disappeared before blockchain anchoring completed.",
                );
            }
            Err(error) => {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Profile KYC blockchain anchoring failed: {}", error),
                );
            }
        };
    }

    publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);

    Json(ApiResponse::ok(UpsertKycDocumentResponse {
        success: true,
        document_id: Some(record.id.max(0) as u64),
        message: format!(
            "Profile document {} was uploaded through the Rust revision workspace and the account is back in pending review.",
            record.document_name
        ),
    }))
    .into_response()
}

async fn update_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<UpsertKycDocumentRequest>,
) -> Json<ApiResponse<UpsertKycDocumentResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message: "Sign in before editing profile KYC rows.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message:
                "Profile KYC document actions are unavailable because the database connection is disabled."
                    .into(),
        }));
    };

    let payload = match validate_profile_kyc_payload(&payload) {
        Ok(value) => value,
        Err(message) => {
            return Json(ApiResponse::ok(UpsertKycDocumentResponse {
                success: false,
                document_id: Some(document_id.max(0) as u64),
                message,
            }));
        }
    };

    match update_kyc_document(
        pool,
        document_id,
        resolved.user.id,
        &UpdateKycDocumentInput {
            document_name: payload.document_name.clone(),
            document_type: payload.document_type.clone(),
            file_path: None,
            original_name: None,
            mime_type: None,
            file_size: None,
            next_status: AccountStatus::PendingReview.legacy_code(),
        },
    )
    .await
    {
        Ok(Some(record)) => {
            publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);
            Json(ApiResponse::ok(UpsertKycDocumentResponse {
                success: true,
                document_id: Some(record.id.max(0) as u64),
                message: format!(
                    "Profile document {} was updated and the account is queued for another admin review.",
                    record.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message: "That KYC row could not be found for this signed-in profile.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(UpsertKycDocumentResponse {
            success: false,
            document_id: Some(document_id.max(0) as u64),
            message: format!("Profile document update failed: {}", error),
        })),
    }
}

async fn replace_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before replacing a profile KYC file.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Profile KYC document actions are unavailable because the database connection is disabled.",
        );
    };

    let Some(existing) = find_kyc_document_by_id(pool, document_id)
        .await
        .unwrap_or(None)
    else {
        return text_response(StatusCode::NOT_FOUND, "KYC document not found.");
    };

    if existing.user_id != resolved.user.id {
        return text_response(
            StatusCode::FORBIDDEN,
            "This signed-in profile cannot replace that KYC document.",
        );
    }

    let parsed = match parse_kyc_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => return text_response(StatusCode::BAD_REQUEST, &message),
    };

    let Some(document_type) = validate_kyc_document_type(&parsed.document_type) else {
        return text_response(
            StatusCode::BAD_REQUEST,
            "Choose either standard or blockchain before replacing this profile file.",
        );
    };

    let saved = match state
        .document_storage
        .save_kyc_document(resolved.user.id, &parsed.original_name, &parsed.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC replacement storage failed: {}", error),
            );
        }
    };

    if let Err(error) = state
        .document_storage
        .delete_document(
            normalize_storage_provider(&existing.file_path),
            &existing.file_path,
        )
        .await
    {
        tracing::warn!(
            document_id,
            error = %error,
            "old profile KYC file could not be removed after replacement"
        );
    }

    let mut record = match update_kyc_document(
        pool,
        document_id,
        resolved.user.id,
        &UpdateKycDocumentInput {
            document_name: parsed.document_name.clone(),
            document_type: document_type.clone(),
            file_path: Some(saved.file_path),
            original_name: Some(parsed.original_name.clone()),
            mime_type: parsed.mime_type.clone(),
            file_size: Some(parsed.bytes.len() as i64),
            next_status: AccountStatus::PendingReview.legacy_code(),
        },
    )
    .await
    {
        Ok(Some(value)) => value,
        Ok(None) => {
            return text_response(
                StatusCode::NOT_FOUND,
                "That KYC row could not be found for this signed-in profile.",
            );
        }
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Profile KYC replacement persistence failed: {}", error),
            );
        }
    };

    if document_type == "blockchain" {
        record = match verify_kyc_document_blockchain(
            pool,
            record.id,
            resolved.user.id,
            Some("Anchored immediately after file replacement in the Rust profile."),
            AccountStatus::PendingReview.legacy_code(),
        )
        .await
        {
            Ok(Some(value)) => value,
            Ok(None) => {
                return text_response(
                    StatusCode::NOT_FOUND,
                    "The replaced KYC document disappeared before blockchain anchoring completed.",
                );
            }
            Err(error) => {
                return text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Profile KYC blockchain anchoring failed: {}", error),
                );
            }
        };
    }

    publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);

    Json(ApiResponse::ok(UpsertKycDocumentResponse {
        success: true,
        document_id: Some(record.id.max(0) as u64),
        message: format!(
            "Profile document {} was replaced and sent back for admin review.",
            record.document_name
        ),
    }))
    .into_response()
}

async fn verify_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
    Json(payload): Json<VerifyKycDocumentRequest>,
) -> Json<ApiResponse<VerifyKycDocumentResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: "Sign in before anchoring a profile KYC document.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message:
                "Profile KYC blockchain actions are unavailable because the database connection is disabled."
                    .into(),
        }));
    };

    match verify_kyc_document_blockchain(
        pool,
        document_id,
        resolved.user.id,
        payload.note.as_deref(),
        AccountStatus::PendingReview.legacy_code(),
    )
    .await
    {
        Ok(Some(record)) => {
            publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);
            Json(ApiResponse::ok(VerifyKycDocumentResponse {
                success: true,
                document_id: record.id.max(0) as u64,
                hash: record.hash,
                message: format!(
                    "Profile document {} is now anchored to the Rust blockchain stub and pending review again.",
                    record.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: "That KYC row could not be found for this signed-in profile.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(VerifyKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            hash: None,
            message: format!("Profile blockchain verification failed: {}", error),
        })),
    }
}

async fn delete_profile_kyc_document_handler(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
) -> Json<ApiResponse<DeleteKycDocumentResponse>> {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "Sign in before deleting a profile KYC document.".into(),
        }));
    };

    let Some(pool) = state.pool.as_ref() else {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message:
                "Profile KYC document actions are unavailable because the database connection is disabled."
                    .into(),
        }));
    };

    let Some(existing) = find_kyc_document_by_id(pool, document_id)
        .await
        .unwrap_or(None)
    else {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "That KYC row no longer exists.".into(),
        }));
    };

    if existing.user_id != resolved.user.id {
        return Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "This signed-in profile cannot delete that KYC row.".into(),
        }));
    }

    match delete_kyc_document(
        pool,
        document_id,
        resolved.user.id,
        AccountStatus::PendingReview.legacy_code(),
    )
    .await
    {
        Ok(Some(record)) => {
            if let Err(error) = state
                .document_storage
                .delete_document(
                    normalize_storage_provider(&record.file_path),
                    &record.file_path,
                )
                .await
            {
                tracing::warn!(
                    document_id,
                    error = %error,
                    "profile KYC binary could not be removed after row deletion"
                );
            }

            publish_profile_document_realtime(&state, &resolved.user.name, resolved.user.id);

            Json(ApiResponse::ok(DeleteKycDocumentResponse {
                success: true,
                document_id: record.id.max(0) as u64,
                message: format!(
                    "Profile document {} was removed and the account is queued for another review cycle.",
                    record.document_name
                ),
            }))
        }
        Ok(None) => Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: "That KYC row could not be found for this signed-in profile.".into(),
        })),
        Err(error) => Json(ApiResponse::ok(DeleteKycDocumentResponse {
            success: false,
            document_id: document_id.max(0) as u64,
            message: format!("Profile KYC deletion failed: {}", error),
        })),
    }
}

fn publish_profile_document_realtime(state: &AppState, actor_name: &str, actor_user_id: i64) {
    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(actor_user_id.max(0) as u64),
            subject_user_id: Some(actor_user_id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} updated profile KYC documents through the Rust revision workspace.",
                actor_name
            ),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"]),
    );
}

async fn upload_kyc_document_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before uploading a KYC document.",
        );
    };

    if !resolved
        .user
        .account_status()
        .map(|status| can_submit_onboarding(Some(status)))
        .unwrap_or(false)
    {
        return text_response(
            StatusCode::FORBIDDEN,
            "This account cannot upload onboarding KYC documents in the current Rust state.",
        );
    }

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Rust KYC upload is unavailable because the database connection is disabled.",
        );
    };

    let parsed = match parse_kyc_document_upload(multipart).await {
        Ok(value) => value,
        Err(message) => return text_response(StatusCode::BAD_REQUEST, &message),
    };

    let saved = match state
        .document_storage
        .save_kyc_document(resolved.user.id, &parsed.original_name, &parsed.bytes)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("KYC document storage failed: {}", error),
            );
        }
    };

    let record = match create_kyc_document(
        pool,
        &CreateKycDocumentInput {
            user_id: resolved.user.id,
            document_name: parsed.document_name,
            document_type: parsed.document_type,
            file_path: saved.file_path,
            original_name: Some(parsed.original_name),
            mime_type: parsed.mime_type,
            file_size: Some(parsed.bytes.len() as i64),
        },
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("KYC document persistence failed: {}", error),
            );
        }
    };

    let response = ApiResponse::ok(build_kyc_document_item(record, true));

    state.publish_realtime(
        RoutedRealtimeEvent::new(RealtimeEvent {
            kind: RealtimeEventKind::AdminDashboardUpdated,
            leg_id: None,
            conversation_id: None,
            offer_id: None,
            message_id: None,
            actor_user_id: Some(resolved.user.id.max(0) as u64),
            subject_user_id: Some(resolved.user.id.max(0) as u64),
            presence_state: None,
            last_read_message_id: None,
            summary: format!(
                "{} uploaded a KYC document through Rust onboarding.",
                resolved.user.name
            ),
        })
        .for_permission_keys(["manage_users", "access_admin_portal"]),
    );

    Json(response).into_response()
}

async fn download_kyc_document_file(
    State(state): State<AppState>,
    Path(document_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let Some(resolved) = auth_session::resolve_session_from_headers(&state, &headers)
        .await
        .ok()
        .flatten()
    else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Sign in before opening this KYC document.",
        );
    };

    let Some(pool) = state.pool.as_ref() else {
        return text_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Rust KYC documents are unavailable because the database connection is disabled.",
        );
    };

    let Some(document) = find_kyc_document_by_id(pool, document_id)
        .await
        .unwrap_or(None)
    else {
        return text_response(StatusCode::NOT_FOUND, "KYC document not found.");
    };

    let can_view = resolved.user.primary_role() == Some(UserRole::Admin)
        || document.user_id == resolved.user.id;
    if !can_view {
        return text_response(
            StatusCode::FORBIDDEN,
            "This KYC document is restricted to the uploader and admin users.",
        );
    }

    let bytes = match state
        .document_storage
        .read_document(
            normalize_storage_provider(&document.file_path),
            &document.file_path,
        )
        .await
    {
        Ok(value) => value,
        Err(error) => {
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("KYC document read failed: {}", error),
            );
        }
    };

    let mime = document
        .mime_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("application/octet-stream");
    let file_name = document
        .original_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("document.bin");

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "inline; filename=\"{}\"",
            sanitize_header_file_name(file_name)
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    response
}

#[derive(Debug)]
struct ParsedKycDocumentUpload {
    document_name: String,
    document_type: String,
    original_name: String,
    mime_type: Option<String>,
    bytes: Vec<u8>,
}

async fn parse_kyc_document_upload(
    mut multipart: Multipart,
) -> Result<ParsedKycDocumentUpload, String> {
    let mut document_name = None::<String>;
    let mut document_type = None::<String>;
    let mut original_name = None::<String>;
    let mut mime_type = None::<String>;
    let mut bytes = None::<Vec<u8>>;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| format!("KYC upload parsing failed: {}", error))?
    {
        match field.name().unwrap_or_default() {
            "document_name" => {
                document_name = Some(
                    field
                        .text()
                        .await
                        .map_err(|error| format!("Document name parsing failed: {}", error))?,
                );
            }
            "document_type" => {
                document_type = Some(
                    field
                        .text()
                        .await
                        .map_err(|error| format!("Document type parsing failed: {}", error))?,
                );
            }
            "file" => {
                original_name = field.file_name().map(str::to_string);
                mime_type = field.content_type().map(str::to_string);
                let payload = field
                    .bytes()
                    .await
                    .map_err(|error| format!("Document file parsing failed: {}", error))?;
                bytes = Some(payload.to_vec());
            }
            _ => {}
        }
    }

    let document_name = document_name
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Enter a document name before uploading a KYC file.".to_string())?;
    let document_type = document_type
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Choose a KYC document type before uploading a file.".to_string())?;
    let bytes =
        bytes.ok_or_else(|| "Choose a file before uploading a KYC document.".to_string())?;
    if bytes.is_empty() {
        return Err("Uploaded KYC files cannot be empty.".into());
    }

    Ok(ParsedKycDocumentUpload {
        document_name,
        document_type,
        original_name: original_name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "document.bin".into()),
        mime_type: mime_type
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        bytes,
    })
}

fn normalize_storage_provider(file_path: &str) -> &str {
    if file_path.starts_with("local://") {
        "local"
    } else if file_path.starts_with("ibm-cos://") {
        "ibm_cos"
    } else if file_path.starts_with("s3://") {
        "s3"
    } else {
        "local"
    }
}

fn kyc_document_download_path(document_id: u64) -> String {
    format!("/auth/onboarding/documents/{}/file", document_id)
}

fn profile_file_label(file_path: &str) -> String {
    let normalized = file_path
        .strip_prefix("local://")
        .or_else(|| file_path.strip_prefix("ibm-cos://"))
        .or_else(|| file_path.strip_prefix("s3://"))
        .unwrap_or(file_path);
    normalized
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("document.bin")
        .to_string()
}

fn sanitize_header_file_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '"' | '\\' | '\r' | '\n' => '_',
            _ => ch,
        })
        .collect::<String>();
    if sanitized.trim().is_empty() {
        "document.bin".into()
    } else {
        sanitized
    }
}

fn format_profile_datetime(value: &chrono::NaiveDateTime) -> String {
    value.format("%b %d, %Y %H:%M").to_string()
}

fn text_response(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}
