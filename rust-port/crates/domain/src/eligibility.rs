use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EligibilityAction {
    View,
    Offer,
    TenderAccept,
    BookNow,
}

impl EligibilityAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Offer => "offer",
            Self::TenderAccept => "tender_accept",
            Self::BookNow => "book_now",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligibilityBlock {
    pub key: String,
    pub label: String,
    pub detail: String,
}

impl EligibilityBlock {
    pub fn new(key: &str, label: &str, detail: &str) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityDecision {
    pub tenant_id: String,
    pub posting_id: i64,
    pub carrier_profile_id: i64,
    pub eligible: bool,
    pub result_code: String,
    pub result_detail: String,
    pub blocks: Vec<EligibilityBlock>,
    pub warnings: Vec<EligibilityBlock>,
    pub overridden_blocks: Vec<EligibilityBlock>,
    pub evaluated_at: NaiveDateTime,
}
