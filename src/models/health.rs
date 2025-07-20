use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProcessorHealthResponse {
    pub failing: bool,
    #[serde(rename = "minResponseTime")]
    pub min_response_time: u32,
}