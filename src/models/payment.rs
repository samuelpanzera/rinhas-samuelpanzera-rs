use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};


#[derive(Deserialize, Debug)]
pub struct ApiPaymentRequest {
    #[serde(rename = "correlationId")]
    pub correlation_id: Uuid,
    pub amount: Decimal,
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