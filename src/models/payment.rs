use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

fn serialize_timestamp<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    serializer.serialize_str(&formatted)
}

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
    #[serde(rename = "requestedAt", serialize_with = "serialize_timestamp")]
    pub requested_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct ProcessorPaymentResponse {
    pub message: String,
}
