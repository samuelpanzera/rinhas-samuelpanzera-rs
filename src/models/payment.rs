use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use uuid::Uuid;

// Função customizada para serializar timestamp no formato exato: "2025-07-15T12:34:56.000Z"
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

// PaymentData removido - não mais necessário após remoção do PaymentRepository
// Todos os dados são salvos diretamente na SharedMemory

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
