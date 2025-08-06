use may_minihttp::{Request, Response};
use serde_json;
use std::io;

use crate::models::summary::ApiSummaryResponse;
use crate::shared_mem::SharedMemoryManager;
use rust_decimal::Decimal;

pub fn handle_payments_summary(
    req: Request,
    res: &mut Response,
    shared_memory: &SharedMemoryManager,
) -> io::Result<()> {
    let (from, to) = parse_query_params_from_path(req.path());

    let result = shared_memory.get_summary_range(from.as_deref(), to.as_deref());

    match result {
        Ok((default_requests, default_amount_cents, fallback_requests, fallback_amount_cents)) => {

            let default_amount_decimal = Decimal::new(default_amount_cents as i64, 2);
            let fallback_amount_decimal = Decimal::new(fallback_amount_cents as i64, 2);

            let summary = ApiSummaryResponse {
                default: crate::models::summary::SummaryDetails {
                    total_requests: default_requests,
                    total_amount: default_amount_decimal,
                },
                fallback: crate::models::summary::SummaryDetails {
                    total_requests: fallback_requests,
                    total_amount: fallback_amount_decimal,
                },
            };

            let json_body = match serde_json::to_vec(&summary) {
                Ok(json) => json,
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::Other, e));
                }
            };

            res.status_code(200, "OK")
                .header("Content-Type: application/json")
                .header("Cache-Control: no-cache")
                .body_mut()
                .extend_from_slice(&json_body);
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}

fn parse_query_params_from_path(path: &str) -> (Option<String>, Option<String>) {

    if let Some(query_start) = path.find('?') {
        let query = &path[query_start + 1..];

        let mut from: Option<String> = None;
        let mut to: Option<String> = None;

        for param in query.split('&') {
            if let Some(eq_pos) = param.find('=') {
                let key = &param[..eq_pos];
                let value = &param[eq_pos + 1..];

                let decoded_value = url_decode(value);

                match key {
                    "from" => {
                        from = Some(decoded_value.clone());
                    }
                    "to" => {
                        to = Some(decoded_value.clone());
                    }
                    _ => {}
                }
            }
        }

        (from, to)
    } else {
        (None, None)
    }
}


fn url_decode(input: &str) -> String {
    input
        .replace("%3A", ":")
        .replace("%2B", "+")
        .replace("%2D", "-")
        .replace("%2E", ".")
        .replace("%20", " ")
        .replace("+", " ")
}
