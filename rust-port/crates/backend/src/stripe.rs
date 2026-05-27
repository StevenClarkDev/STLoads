use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

use crate::config::RuntimeConfig;

#[derive(Clone)]
pub struct StripeService {
    client: Client,
    config: StripeConfig,
}

#[derive(Clone)]
struct StripeConfig {
    secret_key: Option<String>,
    api_base_url: String,
    connect_refresh_url: String,
    connect_return_url: String,
}

#[derive(Debug, Clone)]
pub struct StripeAccount {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct StripeAccountLink {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct StripePaymentIntent {
    pub id: String,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StripeTransfer {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct StripeAccountResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct StripeAccountLinkResponse {
    url: String,
}

#[derive(Debug, Deserialize)]
struct StripePaymentIntentResponse {
    id: String,
    client_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StripeTransferResponse {
    id: String,
}

impl StripeService {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        let portal_url = config.portal_url.trim().trim_end_matches('/').to_string();
        Self {
            client: Client::new(),
            config: StripeConfig {
                secret_key: config.stripe_secret_key.clone(),
                api_base_url: config
                    .stripe_api_base_url
                    .trim()
                    .trim_end_matches('/')
                    .to_string(),
                connect_refresh_url: config
                    .stripe_connect_refresh_url
                    .clone()
                    .unwrap_or_else(|| format!("{}/settings/payouts?refresh=1", portal_url)),
                connect_return_url: config
                    .stripe_connect_return_url
                    .clone()
                    .unwrap_or_else(|| format!("{}/settings/payouts?done=1", portal_url)),
            },
        }
    }

    pub fn is_configured(&self) -> bool {
        self.config
            .secret_key
            .as_deref()
            .map(is_usable_secret)
            .unwrap_or(false)
    }

    pub async fn create_express_account(
        &self,
        email: &str,
        request_id: Option<&str>,
    ) -> Result<StripeAccount, String> {
        let response = self
            .post_form::<StripeAccountResponse>(
                "/accounts",
                &[
                    ("type", "express"),
                    ("capabilities[transfers][requested]", "true"),
                    ("email", email),
                ],
                request_id,
                None,
            )
            .await?;

        Ok(StripeAccount { id: response.id })
    }

    pub async fn create_account_link_with_urls(
        &self,
        account_id: &str,
        refresh_url: Option<&str>,
        return_url: Option<&str>,
        request_id: Option<&str>,
    ) -> Result<StripeAccountLink, String> {
        let refresh_url = refresh_url.unwrap_or(&self.config.connect_refresh_url);
        let return_url = return_url.unwrap_or(&self.config.connect_return_url);
        let response = self
            .post_form::<StripeAccountLinkResponse>(
                "/account_links",
                &[
                    ("account", account_id),
                    ("type", "account_onboarding"),
                    ("refresh_url", refresh_url),
                    ("return_url", return_url),
                ],
                request_id,
                None,
            )
            .await?;

        Ok(StripeAccountLink { url: response.url })
    }

    // Stripe requests are external API contracts; parameters stay explicit so
    // money movement and idempotency evidence are visible at each call site.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        transfer_group: &str,
        leg_id: i64,
        description: &str,
        request_id: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<StripePaymentIntent, String> {
        let amount = amount.to_string();
        let leg_id = leg_id.to_string();

        let response = self
            .post_form::<StripePaymentIntentResponse>(
                "/payment_intents",
                &[
                    ("amount", &amount),
                    ("currency", currency),
                    ("transfer_group", transfer_group),
                    ("automatic_payment_methods[enabled]", "true"),
                    ("automatic_payment_methods[allow_redirects]", "never"),
                    ("description", description),
                    ("metadata[leg_id]", &leg_id),
                    ("metadata[request_id]", request_id.unwrap_or("")),
                ],
                request_id,
                idempotency_key,
            )
            .await?;

        Ok(StripePaymentIntent {
            id: response.id,
            client_secret: response.client_secret,
        })
    }

    // Stripe transfers carry payout, destination, grouping, and idempotency data
    // as one request boundary.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_transfer(
        &self,
        amount: i64,
        currency: &str,
        destination_account_id: &str,
        source_charge_id: &str,
        transfer_group: Option<&str>,
        request_id: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<StripeTransfer, String> {
        let amount = amount.to_string();
        let mut form = vec![
            ("amount", amount.as_str()),
            ("currency", currency),
            ("destination", destination_account_id),
            ("source_transaction", source_charge_id),
        ];
        if let Some(transfer_group) = transfer_group.filter(|value| !value.trim().is_empty()) {
            form.push(("transfer_group", transfer_group));
        }
        if let Some(request_id) = request_id.filter(|value| !value.trim().is_empty()) {
            form.push(("metadata[request_id]", request_id));
        }

        let response = self
            .post_form::<StripeTransferResponse>("/transfers", &form, request_id, idempotency_key)
            .await?;

        Ok(StripeTransfer { id: response.id })
    }

    async fn post_form<T>(
        &self,
        path: &str,
        form: &[(&str, &str)],
        request_id: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        let Some(secret_key) = self
            .config
            .secret_key
            .as_deref()
            .filter(|value| is_usable_secret(value))
        else {
            return Err("STRIPE_SECRET is not configured for live Stripe API calls.".into());
        };

        let url = format!("{}{}", self.config.api_base_url, path);
        let mut request = self
            .client
            .post(&url)
            .bearer_auth(secret_key)
            .header("x-request-id", request_id.unwrap_or(""))
            .form(form);
        if let Some(idempotency_key) = idempotency_key
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request = request.header("Idempotency-Key", idempotency_key);
        }

        let response = request
            .send()
            .await
            .map_err(|error| format!("Stripe request failed: {}", error))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| format!("Stripe response read failed: {}", error))?;

        if !status.is_success() {
            warn!(status = %status, "Stripe API returned an error");
            return Err(format!(
                "Stripe API returned {}: {}",
                status,
                summarize_stripe_error(&body)
            ));
        }

        serde_json::from_str::<T>(&body)
            .map_err(|error| format!("Stripe response parsing failed: {}", error))
    }
}

fn is_usable_secret(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.eq_ignore_ascii_case("null")
        && !value.eq_ignore_ascii_case("replace-me")
        && !value.eq_ignore_ascii_case("replace_me")
}

fn summarize_stripe_error(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(|message| message.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| body.chars().take(240).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_placeholder_secrets() {
        assert!(!is_usable_secret(""));
        assert!(!is_usable_secret("replace-me"));
        assert!(!is_usable_secret("null"));
        assert!(is_usable_secret("sk_test_123"));
    }
}
