#[path = "auth/forgot_password.rs"]
mod forgot_password;
#[path = "auth/landing.rs"]
mod landing;
#[path = "auth/login.rs"]
mod login;
#[path = "auth/mfa.rs"]
mod mfa;
#[path = "auth/onboarding.rs"]
mod onboarding;
#[path = "auth/register.rs"]
mod register;
#[path = "auth/reset_password.rs"]
mod reset_password;
#[path = "auth/verify_otp.rs"]
mod verify_otp;

pub use forgot_password::ForgotPasswordPage;
pub use landing::PortalLandingPage;
pub use login::LoginPage;
pub use mfa::MfaPage;
pub use onboarding::OnboardingPage;
pub use register::RegisterPage;
pub use reset_password::ResetPasswordPage;
pub use verify_otp::VerifyOtpPage;
