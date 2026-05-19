use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha384};
use thiserror::Error;

pub const SUPPORTED_CONTRACT_VERSION: &str = "2026-05-01";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtmpContractAction {
    Publish,
    Update,
    Withdraw,
    Close,
    Cancel,
    Status,
    Document,
    Finance,
}

impl AtmpContractAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Publish => "publish",
            Self::Update => "update",
            Self::Withdraw => "withdraw",
            Self::Close => "close",
            Self::Cancel => "cancel",
            Self::Status => "status",
            Self::Document => "document",
            Self::Finance => "finance",
        }
    }

    pub const fn requires_payload(self) -> bool {
        matches!(
            self,
            Self::Publish | Self::Update | Self::Status | Self::Document | Self::Finance
        )
    }

    pub const fn terminal_status(self) -> Option<&'static str> {
        match self {
            Self::Withdraw => Some("withdrawn"),
            Self::Cancel => Some("canceled"),
            Self::Close => Some("closed"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmpContractEnvelope {
    pub contract_version: String,
    pub action: AtmpContractAction,
    pub tenant_id: String,
    pub event_id: String,
    pub correlation_id: String,
    pub idempotency_key: String,
    pub atmp_load_id: String,
    pub atmp_leg_id: Option<String>,
    pub release_gate: Option<String>,
    pub payload: Value,
}

impl AtmpContractEnvelope {
    pub fn try_from_value(value: Value) -> Result<Self, AtmpContractError> {
        let envelope = serde_json::from_value::<Self>(value.clone())
            .map_err(|error| AtmpContractError::InvalidJson(error.to_string()))?;
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn validate(&self) -> Result<(), AtmpContractError> {
        let mut missing = Vec::new();
        require(&mut missing, "tenant_id", &self.tenant_id);
        require(&mut missing, "event_id", &self.event_id);
        require(&mut missing, "correlation_id", &self.correlation_id);
        require(&mut missing, "idempotency_key", &self.idempotency_key);
        require(&mut missing, "atmp_load_id", &self.atmp_load_id);

        if !missing.is_empty() {
            return Err(AtmpContractError::MissingFields(missing));
        }

        if self.contract_version.trim() != SUPPORTED_CONTRACT_VERSION {
            return Err(AtmpContractError::UnsupportedVersion(
                self.contract_version.clone(),
            ));
        }

        if self.action.requires_payload() && self.payload.is_null() {
            return Err(AtmpContractError::InvalidPayload(
                "payload is required for this action".into(),
            ));
        }

        if matches!(
            self.action,
            AtmpContractAction::Publish | AtmpContractAction::Update
        ) {
            validate_release_gate(self.release_gate.as_deref())?;
            validate_board_payload(&self.payload)?;
        }

        Ok(())
    }

    pub fn normalized_payload(&self) -> Value {
        serde_json::json!({
            "contract_version": self.contract_version,
            "action": self.action.as_str(),
            "tenant_id": self.tenant_id,
            "event_id": self.event_id,
            "correlation_id": self.correlation_id,
            "idempotency_key": self.idempotency_key,
            "atmp_load_id": self.atmp_load_id,
            "atmp_leg_id": self.atmp_leg_id,
            "release_gate": self.release_gate,
            "payload": self.payload,
        })
    }

    pub fn payload_hash(&self) -> String {
        stable_json_hash(&self.normalized_payload())
    }
}

#[derive(Debug, Error)]
pub enum AtmpContractError {
    #[error("invalid_json: {0}")]
    InvalidJson(String),
    #[error("unsupported_contract_version: {0}")]
    UnsupportedVersion(String),
    #[error("missing required fields: {0:?}")]
    MissingFields(Vec<&'static str>),
    #[error("invalid_release_gate: {0}")]
    InvalidReleaseGate(String),
    #[error("invalid_payload: {0}")]
    InvalidPayload(String),
}

impl AtmpContractError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidJson(_) => "invalid_json",
            Self::UnsupportedVersion(_) => "unsupported_contract_version",
            Self::MissingFields(_) => "missing_required_fields",
            Self::InvalidReleaseGate(_) => "invalid_release_gate",
            Self::InvalidPayload(_) => "invalid_payload",
        }
    }
}

pub fn stable_json_hash(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    let mut hasher = Sha384::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn require(missing: &mut Vec<&'static str>, key: &'static str, value: &str) {
    if value.trim().is_empty() {
        missing.push(key);
    }
}

fn validate_release_gate(value: Option<&str>) -> Result<(), AtmpContractError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(AtmpContractError::InvalidReleaseGate(
            "release_gate is required".into(),
        ));
    };

    if matches!(
        value,
        "stloads_ready" | "ready" | "approved" | "dispatch_released"
    ) {
        Ok(())
    } else {
        Err(AtmpContractError::InvalidReleaseGate(value.into()))
    }
}

fn validate_board_payload(payload: &Value) -> Result<(), AtmpContractError> {
    let mut missing = Vec::new();
    for key in [
        "freight_mode",
        "equipment_type",
        "pickup_city",
        "pickup_address",
        "pickup_window_start",
        "dropoff_city",
        "dropoff_address",
        "dropoff_window_start",
    ] {
        if payload
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            missing.push(key);
        }
    }

    if !missing.is_empty() {
        return Err(AtmpContractError::InvalidPayload(format!(
            "missing payload fields: {missing:?}"
        )));
    }

    let weight = payload.get("weight").and_then(Value::as_f64).unwrap_or(0.0);
    if weight <= 0.0 {
        return Err(AtmpContractError::InvalidPayload(
            "weight must be greater than zero".into(),
        ));
    }

    let board_rate = payload
        .get("board_rate")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if board_rate < 0.0 {
        return Err(AtmpContractError::InvalidPayload(
            "board_rate cannot be negative".into(),
        ));
    }

    Ok(())
}
