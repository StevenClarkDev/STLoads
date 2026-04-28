use chrono::Datelike;
use db::{
    DbPool,
    email_outbox::{
        EmailOutboxRecord, EnqueueEmailParams, claim_due_emails, enqueue_email,
        mark_email_delivered, mark_email_retry, reset_stale_processing_emails,
    },
};
use domain::auth::AccountStatus;
use lettre::{
    Address, Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tokio::time::{Duration, sleep};
use tracing::{debug, info, warn};

use crate::config::RuntimeConfig;

#[derive(Clone)]
pub struct EmailService {
    config: EmailConfig,
    pool: Option<DbPool>,
}

#[derive(Clone)]
struct EmailConfig {
    mailer: MailerMode,
    host: Option<String>,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    encryption: Option<String>,
    from_address: String,
    from_name: String,
    fail_open: bool,
    outbox_enabled: bool,
    outbox_worker_enabled: bool,
    outbox_batch_size: i64,
    outbox_retry_interval_seconds: u64,
    outbox_max_attempts: i32,
    portal_url: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MailerMode {
    Disabled,
    Log,
    Smtp,
}

#[derive(Debug, Clone)]
pub enum MailOutcome {
    Sent,
    Logged,
    Skipped,
    Queued,
    FailedOpen(String),
}

impl MailOutcome {
    pub fn status_note(&self) -> Option<String> {
        match self {
            MailOutcome::Sent => Some("Email notification sent.".into()),
            MailOutcome::Logged => Some("Email notification logged by the Rust mailer.".into()),
            MailOutcome::Queued => {
                Some("Email notification queued for retry by the Rust mailer.".into())
            }
            MailOutcome::Skipped => None,
            MailOutcome::FailedOpen(message) => Some(message.clone()),
        }
    }

    pub fn append_to_message(&self, base: impl Into<String>) -> String {
        let base = base.into();
        match self.status_note() {
            Some(note) if !note.trim().is_empty() => format!("{} {}", base, note),
            _ => base,
        }
    }
}

impl EmailService {
    #[cfg(test)]
    pub fn from_config(config: &RuntimeConfig) -> Self {
        Self::from_config_with_pool(config, None)
    }

    pub fn from_config_with_pool(config: &RuntimeConfig, pool: Option<DbPool>) -> Self {
        let mailer = match config.mail_mailer.trim().to_ascii_lowercase().as_str() {
            "smtp" => MailerMode::Smtp,
            "log" | "array" => MailerMode::Log,
            "none" | "null" | "disabled" => MailerMode::Disabled,
            other => {
                warn!(
                    mailer = other,
                    "unknown MAIL_MAILER value; falling back to log mailer"
                );
                MailerMode::Log
            }
        };

        Self {
            config: EmailConfig {
                mailer,
                host: config.mail_host.clone(),
                port: config.mail_port,
                username: config.mail_username.clone(),
                password: config.mail_password.clone(),
                encryption: config.mail_encryption.clone(),
                from_address: config.mail_from_address.clone(),
                from_name: config.mail_from_name.clone(),
                fail_open: config.mail_fail_open,
                outbox_enabled: config.mail_outbox_enabled,
                outbox_worker_enabled: config.mail_outbox_worker_enabled,
                outbox_batch_size: config.mail_outbox_batch_size,
                outbox_retry_interval_seconds: config.mail_outbox_retry_interval_seconds,
                outbox_max_attempts: config.mail_outbox_max_attempts,
                portal_url: config.portal_url.clone(),
            },
            pool,
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self.config.mailer {
            MailerMode::Disabled => "disabled",
            MailerMode::Log => "log",
            MailerMode::Smtp => "smtp",
        }
    }

    pub fn outbox_label(&self) -> &'static str {
        if self.pool.is_some() && self.config.outbox_enabled {
            if self.config.outbox_worker_enabled {
                "enabled"
            } else {
                "queued-only"
            }
        } else {
            "disabled"
        }
    }

    pub fn start_outbox_worker(&self) {
        if self.pool.is_none() || !self.config.outbox_enabled || !self.config.outbox_worker_enabled
        {
            return;
        }

        let service = self.clone();
        tokio::spawn(async move {
            service.run_outbox_worker().await;
        });
    }

    pub async fn send_registration_otp(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        otp: &str,
    ) -> Result<MailOutcome, String> {
        let body = otp_template(
            "Your OTP Code",
            "Use the code below to complete your STLoads registration.",
            otp,
            &self.config.portal_url,
        );

        self.send_html(
            to_email,
            to_name,
            "Your Registration OTP",
            &body,
            "registration_otp",
        )
        .await
    }

    pub async fn send_password_reset_otp(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        otp: &str,
    ) -> Result<MailOutcome, String> {
        let body = otp_template(
            "Password Reset Code",
            "Use the code below to reset your STLoads password.",
            otp,
            &self.config.portal_url,
        );

        self.send_html(
            to_email,
            to_name,
            "Your Password Reset OTP",
            &body,
            "password_reset_otp",
        )
        .await
    }

    pub async fn send_account_review_status(
        &self,
        to_email: &str,
        to_name: &str,
        role_label: &str,
        status: AccountStatus,
        remarks: Option<&str>,
    ) -> Result<MailOutcome, String> {
        let (subject, template_name, body) = match status {
            AccountStatus::Approved => (
                "Your STLoads Account Is Approved",
                "account_approved",
                account_approved_template(
                    to_name,
                    role_label,
                    &self.config.portal_url,
                    chrono::Utc::now()
                        .date_naive()
                        .format("%b %d, %Y")
                        .to_string(),
                ),
            ),
            AccountStatus::Rejected => (
                "STLoads Account Application Update",
                "account_rejected",
                account_rejected_template(
                    to_name,
                    role_label,
                    remarks.unwrap_or("No remarks were provided by the review team."),
                    &self.config.portal_url,
                ),
            ),
            AccountStatus::RevisionRequested => (
                "Action Required On Your STLoads Application",
                "account_revision",
                account_revision_template(
                    to_name,
                    role_label,
                    remarks.unwrap_or("Please review and update your submitted profile."),
                    &self.config.portal_url,
                ),
            ),
            _ => {
                return Ok(MailOutcome::Skipped);
            }
        };

        self.send_html(to_email, Some(to_name), subject, &body, template_name)
            .await
    }

    pub async fn send_load_review_status(
        &self,
        to_email: &str,
        to_name: &str,
        load_id: i64,
        status_id: i16,
        remarks: Option<&str>,
    ) -> Result<MailOutcome, String> {
        let (subject, title, summary, box_title, box_body, template_name) = match status_id {
            2 => (
                "Your Load Has Been Approved",
                "Your load is approved",
                format!(
                    "Load #{} has been approved by the STLoads operations team.",
                    load_id
                ),
                "Next step",
                "You can now continue with carrier matching, booking, and execution.",
                "load_approved",
            ),
            0 => (
                "Your Load Has Been Rejected",
                "Your load was rejected",
                format!(
                    "Load #{} could not be approved by the STLoads operations team.",
                    load_id
                ),
                "Admin remarks",
                remarks.unwrap_or("No remarks were provided."),
                "load_rejected",
            ),
            7 => (
                "Action Needed: Load Requires Revision",
                "Your load requires revision",
                format!(
                    "Load #{} needs updates before approval can continue.",
                    load_id
                ),
                "What needs updating",
                remarks.unwrap_or("Please review and update the load details."),
                "load_revision",
            ),
            _ => return Ok(MailOutcome::Skipped),
        };

        let body = branded_status_template(
            title,
            &format!("Hello {},", escape_html(to_name)),
            &escape_html(&summary),
            box_title,
            &escape_html(box_body),
            "Open STLoads",
            &self.config.portal_url,
            "#f4f6f9",
            "#1F537B",
        );

        self.send_html(to_email, Some(to_name), subject, &body, template_name)
            .await
    }

    async fn send_html(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        html_body: &str,
        template_name: &str,
    ) -> Result<MailOutcome, String> {
        let to_email = to_email.trim();
        if to_email.is_empty() {
            return Err("Email recipient is empty.".into());
        }

        if let Some(pool) = self.pool.as_ref().filter(|_| self.config.outbox_enabled) {
            let record = enqueue_email(
                pool,
                EnqueueEmailParams {
                    template_name,
                    to_email,
                    to_name,
                    subject,
                    html_body,
                    max_attempts: self.config.outbox_max_attempts,
                },
            )
            .await
            .map_err(|error| format!("Email outbox enqueue failed: {}", error))?;

            return self.deliver_outbox_record(record).await;
        }

        self.deliver_html(to_email, to_name, subject, html_body, template_name)
            .await
    }

    async fn deliver_outbox_record(
        &self,
        record: EmailOutboxRecord,
    ) -> Result<MailOutcome, String> {
        let Some(pool) = self.pool.as_ref() else {
            return self
                .deliver_html(
                    &record.to_email,
                    record.to_name.as_deref(),
                    &record.subject,
                    &record.html_body,
                    &record.template_name,
                )
                .await;
        };

        match self
            .deliver_html(
                &record.to_email,
                record.to_name.as_deref(),
                &record.subject,
                &record.html_body,
                &record.template_name,
            )
            .await
        {
            Ok(MailOutcome::FailedOpen(message)) => {
                let _ = mark_email_retry(pool, record.id, &message).await;
                Ok(MailOutcome::FailedOpen(message))
            }
            Ok(outcome) => {
                let status = outbox_status_for_outcome(&outcome);
                if let Err(error) = mark_email_delivered(pool, record.id, status).await {
                    warn!(
                        email_outbox_id = record.id,
                        error = %error,
                        "email delivered but outbox status update failed"
                    );
                }
                Ok(outcome)
            }
            Err(error) => {
                if let Err(update_error) = mark_email_retry(pool, record.id, &error).await {
                    warn!(
                        email_outbox_id = record.id,
                        error = %update_error,
                        "email failed and outbox retry status update failed"
                    );
                }

                if self.config.fail_open {
                    Ok(MailOutcome::Queued)
                } else {
                    Err(error)
                }
            }
        }
    }

    async fn run_outbox_worker(self) {
        let interval = Duration::from_secs(self.config.outbox_retry_interval_seconds);
        info!(
            batch_size = self.config.outbox_batch_size,
            retry_interval_seconds = self.config.outbox_retry_interval_seconds,
            "email outbox worker started"
        );

        loop {
            if let Err(error) = self.process_outbox_once().await {
                warn!(error = %error, "email outbox worker cycle failed");
            }
            sleep(interval).await;
        }
    }

    pub async fn process_outbox_once(&self) -> Result<usize, String> {
        let Some(pool) = self.pool.as_ref() else {
            return Ok(0);
        };

        reset_stale_processing_emails(pool, 15)
            .await
            .map_err(|error| format!("Email outbox stale reset failed: {}", error))?;

        let records = claim_due_emails(pool, self.config.outbox_batch_size)
            .await
            .map_err(|error| format!("Email outbox claim failed: {}", error))?;
        let count = records.len();

        for record in records {
            debug!(
                email_outbox_id = record.id,
                template = record.template_name,
                to = record.to_email,
                "email outbox worker delivering record"
            );
            let _ = self.deliver_outbox_record(record).await;
        }

        Ok(count)
    }

    async fn deliver_html(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        html_body: &str,
        template_name: &str,
    ) -> Result<MailOutcome, String> {
        match self.config.mailer {
            MailerMode::Disabled => {
                info!(
                    template = template_name,
                    to = to_email,
                    "email skipped because MAIL_MAILER is disabled"
                );
                Ok(MailOutcome::Skipped)
            }
            MailerMode::Log => {
                info!(
                    template = template_name,
                    to = to_email,
                    subject,
                    "email logged by Rust mailer"
                );
                Ok(MailOutcome::Logged)
            }
            MailerMode::Smtp => {
                let config = self.config.clone();
                let to_name = to_name.map(str::to_string);
                let subject = subject.to_string();
                let html_body = html_body.to_string();
                let template_name = template_name.to_string();
                let to_email = to_email.to_string();
                let send_subject = subject.clone();
                let send_to_email = to_email.clone();

                let send_result = tokio::task::spawn_blocking(move || {
                    send_smtp_message(
                        &config,
                        &send_to_email,
                        to_name.as_deref(),
                        &send_subject,
                        &html_body,
                    )
                })
                .await
                .map_err(|error| format!("Email task failed: {}", error))?;

                match send_result {
                    Ok(()) => {
                        info!(
                            template = template_name,
                            to = to_email,
                            subject,
                            "email sent through SMTP"
                        );
                        Ok(MailOutcome::Sent)
                    }
                    Err(error) if self.config.fail_open => {
                        warn!(
                            template = template_name,
                            to = to_email,
                            error = %error,
                            "SMTP delivery failed but MAIL_FAIL_OPEN allowed the workflow to continue"
                        );
                        Ok(MailOutcome::FailedOpen(
                            "Email delivery could not be confirmed, but the Rust workflow continued because MAIL_FAIL_OPEN is enabled.".into(),
                        ))
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }
}

fn outbox_status_for_outcome(outcome: &MailOutcome) -> &'static str {
    match outcome {
        MailOutcome::Sent => "sent",
        MailOutcome::Logged => "logged",
        MailOutcome::Skipped => "skipped",
        MailOutcome::Queued | MailOutcome::FailedOpen(_) => "retry",
    }
}

fn send_smtp_message(
    config: &EmailConfig,
    to_email: &str,
    to_name: Option<&str>,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    let host = config
        .host
        .as_ref()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "MAIL_HOST is required when MAIL_MAILER=smtp.".to_string())?;

    let from_address = config
        .from_address
        .parse::<Address>()
        .map_err(|error| format!("MAIL_FROM_ADDRESS is invalid: {}", error))?;
    let to_address = to_email
        .parse::<Address>()
        .map_err(|error| format!("Recipient email is invalid: {}", error))?;

    let email = Message::builder()
        .from(Mailbox::new(Some(config.from_name.clone()), from_address))
        .to(Mailbox::new(to_name.map(str::to_string), to_address))
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|error| format!("Email message build failed: {}", error))?;

    let mut builder = smtp_transport_builder(config, host)?;

    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
    }

    builder
        .build()
        .send(&email)
        .map_err(|error| format!("SMTP send failed: {}", error))?;

    Ok(())
}

fn smtp_transport_builder(
    config: &EmailConfig,
    host: &str,
) -> Result<lettre::transport::smtp::SmtpTransportBuilder, String> {
    let mode = smtp_encryption_mode(config.encryption.as_deref(), config.port);

    let builder = match mode {
        SmtpEncryptionMode::Plain => SmtpTransport::builder_dangerous(host),
        SmtpEncryptionMode::StartTls => SmtpTransport::starttls_relay(host)
            .map_err(|error| format!("SMTP STARTTLS relay setup failed: {}", error))?,
        SmtpEncryptionMode::ImplicitTls => SmtpTransport::relay(host)
            .map_err(|error| format!("SMTP relay setup failed: {}", error))?,
    };

    Ok(builder.port(config.port))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SmtpEncryptionMode {
    Plain,
    StartTls,
    ImplicitTls,
}

fn smtp_encryption_mode(encryption: Option<&str>, port: u16) -> SmtpEncryptionMode {
    match encryption
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("none" | "plain" | "false") => SmtpEncryptionMode::Plain,
        Some("ssl" | "smtps") => SmtpEncryptionMode::ImplicitTls,
        Some("tls" | "starttls" | "true") => SmtpEncryptionMode::StartTls,
        Some(_) => {
            if port == 465 {
                SmtpEncryptionMode::ImplicitTls
            } else {
                SmtpEncryptionMode::StartTls
            }
        }
        None => {
            if port == 465 {
                SmtpEncryptionMode::ImplicitTls
            } else {
                SmtpEncryptionMode::Plain
            }
        }
    }
}

fn otp_template(title: &str, context: &str, otp: &str, portal_url: &str) -> String {
    let year = chrono::Utc::now().year();
    format!(
        r#"<!doctype html>
<html>
<body style="margin:0;padding:0;background:#f4f6f9;font-family:Arial,sans-serif;">
<table width="100%" cellpadding="0" cellspacing="0" style="background:#f4f6f9;padding:40px 0;">
<tr><td align="center">
<table width="600" cellpadding="0" cellspacing="0" style="background:#fff;border-radius:8px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,.08);">
<tr><td style="background:#1F537B;padding:32px 40px;text-align:center;">
<h1 style="margin:0;color:#fff;font-size:24px;font-weight:700;letter-spacing:1px;">STLoads</h1>
<p style="margin:6px 0 0;color:#a8c8e8;font-size:13px;">Freight &amp; Logistics Platform</p>
</td></tr>
<tr><td style="padding:40px;">
<p style="margin:0 0 16px;color:#333;font-size:16px;">Hello,</p>
<p style="margin:0 0 28px;color:#555;font-size:15px;line-height:1.6;">{} This code is valid for <strong>5 minutes</strong>.</p>
<table width="100%" cellpadding="0" cellspacing="0"><tr><td align="center" style="padding:8px 0 32px;">
<div style="display:inline-block;background:#f0f6fc;border:2px dashed #1F537B;border-radius:8px;padding:20px 48px;">
<span style="font-size:42px;font-weight:800;letter-spacing:10px;color:#1F537B;">{}</span>
</div>
</td></tr></table>
<p style="margin:0 0 8px;color:#888;font-size:13px;">If you did not request this code, ignore this email. Do not share this code with anyone.</p>
</td></tr>
<tr><td style="background:#f4f6f9;padding:24px 40px;text-align:center;border-top:1px solid #e8edf2;">
<p style="margin:0;color:#aaa;font-size:12px;">&copy; {} STLoads. All rights reserved.</p>
<p style="margin:6px 0 0;color:#aaa;font-size:12px;">{}</p>
</td></tr>
</table>
</td></tr>
</table>
</body>
</html>"#,
        escape_html(context),
        escape_html(otp),
        year,
        escape_html(portal_url),
    )
    .replace("<html>", &format!("<html><head><title>{}</title></head>", escape_html(title)))
}

fn account_approved_template(
    name: &str,
    role: &str,
    portal_url: &str,
    approved_at: String,
) -> String {
    branded_status_template(
        "Your account is approved",
        &format!("Congratulations, {}.", escape_html(name)),
        &format!(
            "Your <strong>{}</strong> account on STLoads has been verified and approved by our compliance team.",
            escape_html(role)
        ),
        "KYC verified and account active",
        &format!("Approved on {}", escape_html(&approved_at)),
        "Log in to your dashboard",
        portal_url,
        "#f0f7f0",
        "#2e7d32",
    )
}

fn account_rejected_template(name: &str, role: &str, remarks: &str, portal_url: &str) -> String {
    branded_status_template(
        "Account application update",
        &format!("Hello {},", escape_html(name)),
        &format!(
            "Thank you for applying to join STLoads as a <strong>{}</strong>. Our compliance team was unable to approve your account at this time.",
            escape_html(role)
        ),
        "Reason provided",
        &escape_html(remarks),
        "Contact support",
        portal_url,
        "#f9f4f4",
        "#c62828",
    )
}

fn account_revision_template(name: &str, role: &str, remarks: &str, portal_url: &str) -> String {
    branded_status_template(
        "Action required on your application",
        &format!("Hello {},", escape_html(name)),
        &format!(
            "Our team reviewed your <strong>{}</strong> application and needs a few updates before approval can be completed.",
            escape_html(role)
        ),
        "What needs updating",
        &escape_html(remarks),
        "Log in and update my application",
        portal_url,
        "#f9f7f3",
        "#b45309",
    )
}

#[allow(clippy::too_many_arguments)]
fn branded_status_template(
    title: &str,
    greeting: &str,
    body: &str,
    box_title: &str,
    box_body: &str,
    button_label: &str,
    portal_url: &str,
    background: &str,
    accent: &str,
) -> String {
    let year = chrono::Utc::now().year();
    format!(
        r#"<!doctype html>
<html><head><title>{}</title></head>
<body style="margin:0;padding:0;background:{};font-family:Arial,sans-serif;">
<table width="100%" cellpadding="0" cellspacing="0" style="background:{};padding:40px 0;">
<tr><td align="center">
<table width="600" cellpadding="0" cellspacing="0" style="background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 4px 16px rgba(0,0,0,.08);">
<tr><td style="background:#1F537B;padding:40px;text-align:center;">
<h1 style="margin:0 0 6px;color:#fff;font-size:24px;font-weight:700;">STLoads</h1>
<p style="margin:0;color:#a8c8e8;font-size:13px;">{}</p>
</td></tr>
<tr><td style="padding:40px;">
<p style="margin:0 0 16px;color:#1a1a1a;font-size:16px;font-weight:600;">{}</p>
<p style="margin:0 0 24px;color:#555;font-size:15px;line-height:1.7;">{}</p>
<table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;"><tr>
<td style="background:#f8fcff;border-left:4px solid {};border-radius:4px;padding:16px 20px;">
<p style="margin:0 0 6px;color:{};font-size:13px;font-weight:700;text-transform:uppercase;letter-spacing:.5px;">{}</p>
<p style="margin:0;color:#555;font-size:14px;line-height:1.6;">{}</p>
</td></tr></table>
<table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;"><tr><td align="center">
<a href="{}" style="display:inline-block;background:#1F537B;color:#fff;text-decoration:none;font-size:15px;font-weight:700;padding:14px 40px;border-radius:8px;">{}</a>
</td></tr></table>
<p style="margin:0;color:#888;font-size:13px;line-height:1.6;">If you have questions, reply to this email or contact STLoads support.</p>
</td></tr>
<tr><td style="background:#f4f6f9;padding:24px 40px;text-align:center;border-top:1px solid #e8edf2;">
<p style="margin:0;color:#aaa;font-size:12px;">&copy; {} STLoads. All rights reserved.</p>
<p style="margin:6px 0 0;color:#aaa;font-size:12px;">{}</p>
</td></tr>
</table>
</td></tr>
</table>
</body></html>"#,
        escape_html(title),
        background,
        background,
        escape_html(title),
        greeting,
        body,
        accent,
        accent,
        escape_html(box_title),
        box_body,
        escape_html(portal_url),
        escape_html(button_label),
        year,
        escape_html(portal_url),
    )
}

fn escape_html(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect::<Vec<_>>(),
            '>' => "&gt;".chars().collect::<Vec<_>>(),
            '"' => "&quot;".chars().collect::<Vec<_>>(),
            '\'' => "&#39;".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_mailer_records_registration_otp_without_smtp() {
        let service = EmailService::from_config(&test_config("log", true));

        let outcome = service
            .send_registration_otp("driver@example.test", Some("Test Driver"), "123456")
            .await
            .expect("log mailer should not require SMTP");

        assert!(matches!(outcome, MailOutcome::Logged));
    }

    #[tokio::test]
    async fn disabled_mailer_skips_review_notifications() {
        let service = EmailService::from_config(&test_config("disabled", true));

        let outcome = service
            .send_account_review_status(
                "shipper@example.test",
                "Test Shipper",
                "Shipper",
                AccountStatus::Approved,
                None,
            )
            .await
            .expect("disabled mailer should skip cleanly");

        assert!(matches!(outcome, MailOutcome::Skipped));
    }

    #[tokio::test]
    async fn smtp_mailer_without_host_fails_when_fail_open_is_disabled() {
        let service = EmailService::from_config(&test_config("smtp", false));

        let error = service
            .send_password_reset_otp("driver@example.test", Some("Test Driver"), "654321")
            .await
            .expect_err("smtp without host should fail closed");

        assert!(error.contains("MAIL_HOST is required"));
    }

    #[test]
    fn smtp_treats_tls_on_submission_port_as_starttls() {
        assert_eq!(
            smtp_encryption_mode(Some("tls"), 587),
            SmtpEncryptionMode::StartTls
        );
    }

    #[test]
    fn smtp_treats_ssl_as_implicit_tls() {
        assert_eq!(
            smtp_encryption_mode(Some("ssl"), 465),
            SmtpEncryptionMode::ImplicitTls
        );
    }

    #[test]
    fn smtp_defaults_to_implicit_tls_on_smtps_port() {
        assert_eq!(
            smtp_encryption_mode(None, 465),
            SmtpEncryptionMode::ImplicitTls
        );
    }

    #[test]
    fn smtp_defaults_to_plain_when_no_encryption_is_configured() {
        assert_eq!(smtp_encryption_mode(None, 25), SmtpEncryptionMode::Plain);
    }

    fn test_config(mailer: &str, fail_open: bool) -> RuntimeConfig {
        RuntimeConfig {
            bind_addr: "127.0.0.1".into(),
            port: 3001,
            deployment_target: "test".into(),
            environment: "test".into(),
            public_base_url: Some("https://backend.example.test".into()),
            cors_allowed_origins: vec!["https://portal.example.test".into()],
            run_migrations: false,
            database_url: None,
            document_storage_backend: "local".into(),
            document_storage_root: "./runtime/test-documents".into(),
            object_storage_bucket: None,
            object_storage_region: "us-south".into(),
            object_storage_endpoint: None,
            object_storage_access_key_id: None,
            object_storage_secret_access_key: None,
            object_storage_session_token: None,
            object_storage_force_path_style: false,
            object_storage_prefix: "test-documents".into(),
            stripe_webhook_shared_secret: None,
            stripe_webhook_connect_secret: None,
            stripe_secret_key: None,
            stripe_api_base_url: "https://api.stripe.com/v1".into(),
            stripe_connect_refresh_url: Some(
                "https://portal.example.test/settings/payouts?refresh=1".into(),
            ),
            stripe_connect_return_url: Some(
                "https://portal.example.test/settings/payouts?done=1".into(),
            ),
            stripe_live_transfers_required: false,
            tms_shared_secret: None,
            tms_reconciliation_worker_enabled: false,
            tms_reconciliation_interval_seconds: 21_600,
            tms_retry_worker_enabled: false,
            tms_retry_interval_seconds: 300,
            tms_retry_batch_size: 10,
            tms_retry_max_attempts: 5,
            tms_stale_handoff_days: 30,
            mail_mailer: mailer.into(),
            mail_host: None,
            mail_port: 587,
            mail_username: None,
            mail_password: None,
            mail_encryption: Some("tls".into()),
            mail_from_address: "noreply@example.test".into(),
            mail_from_name: "STLoads Test".into(),
            mail_fail_open: fail_open,
            mail_outbox_enabled: false,
            mail_outbox_worker_enabled: false,
            mail_outbox_batch_size: 25,
            mail_outbox_retry_interval_seconds: 30,
            mail_outbox_max_attempts: 8,
            portal_url: "https://portal.example.test".into(),
        }
    }
}
