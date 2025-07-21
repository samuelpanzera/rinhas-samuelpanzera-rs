use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProcessorHealthResponse {
    pub failing: bool,
    #[serde(rename = "minResponseTime")]
    pub min_response_time: u32,
    #[serde(rename = "lastCheckedAt")]
    pub last_checked_at: Option<DateTime<Utc>>,
}
