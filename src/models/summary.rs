use serde::Serialize;
use rust_decimal::Decimal;

#[derive(Serialize, Debug, Default)]
pub struct SummaryDetails {
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    #[serde(rename = "totalAmount")]
    pub total_amount: Decimal,
}

#[derive(Serialize, Debug, Default)]
pub struct ApiSummaryResponse {
    pub default: SummaryDetails,
    pub fallback: SummaryDetails,
}