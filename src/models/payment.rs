use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ApiPaymentRequest {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    pub amount: Decimal,
}

#[derive(Debug, Clone)]
pub struct PaymentData {
    pub correlation_id: Uuid,
    pub amount: Decimal,
    pub is_default_processor: bool, // true = default, false = fallback
}

impl PaymentData {
    pub fn new(correlation_id: Uuid, amount: Decimal, is_default_processor: bool) -> Self {
        Self {
            correlation_id,
            amount,
            is_default_processor,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ProcessorPaymentRequest<'a> {
    #[serde(rename = "correlationId")]
    pub correlation_id: &'a Uuid,
    pub amount: &'a Decimal,
    #[serde(rename = "requestedAt")]
    pub requested_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct ProcessorPaymentResponse {
    pub message: String,
}
