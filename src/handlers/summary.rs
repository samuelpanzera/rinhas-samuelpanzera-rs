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
    println!(
        "🔍 SUMMARY: Handling summary request with path: {}",
        req.path()
    );
    println!("🔍 SUMMARY: Request method: {}", req.method());

    let (from, to) = parse_query_params_from_path(req.path());
    println!("🔍 SUMMARY: Parsed params - from: {:?}, to: {:?}", from, to);

    println!("🔍 SUMMARY: Calling shared_memory.get_summary_range()...");
    let result = shared_memory.get_summary_range(from.as_deref(), to.as_deref());

    match result {
        Ok((default_requests, default_amount_cents, fallback_requests, fallback_amount_cents)) => {
            println!(
                "🔍 SUMMARY: Got data - default: {} requests, {} cents; fallback: {} requests, {} cents",
                default_requests, default_amount_cents, fallback_requests, fallback_amount_cents
            );

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

            println!("🔍 SUMMARY: Serializing to JSON...");
            let json_body = match serde_json::to_vec(&summary) {
                Ok(json) => {
                    println!(
                        "🔍 SUMMARY: JSON serialized successfully, size: {} bytes",
                        json.len()
                    );
                    json
                }
                Err(e) => {
                    println!("❌ SUMMARY: JSON serialization error: {}", e);
                    return Err(io::Error::new(io::ErrorKind::Other, e));
                }
            };

            println!("🔍 SUMMARY: Setting response headers and body...");
            res.status_code(200, "OK")
                .header("Content-Type: application/json")
                .header("Cache-Control: no-cache") // Ensure fresh data for Central Bank audits
                .body_mut()
                .extend_from_slice(&json_body);

            println!("✅ SUMMARY: Response completed successfully");
            Ok(())
        }
        Err(e) => {
            println!("❌ SUMMARY: Error getting summary data: {}", e);
            Err(e)
        }
    }
}

fn parse_query_params_from_path(path: &str) -> (Option<String>, Option<String>) {
    println!("🔍 QUERY_PARSE: Parsing path: {}", path);

    if let Some(query_start) = path.find('?') {
        let query = &path[query_start + 1..];
        println!("🔍 QUERY_PARSE: Found query string: {}", query);

        let mut from: Option<String> = None;
        let mut to: Option<String> = None;

        for param in query.split('&') {
            println!("🔍 QUERY_PARSE: Processing param: {}", param);
            if let Some(eq_pos) = param.find('=') {
                let key = &param[..eq_pos];
                let value = &param[eq_pos + 1..];
                println!("🔍 QUERY_PARSE: Found key='{}', value='{}'", key, value);

                // ✅ URL decode dos valores para suportar timestamps com caracteres especiais
                let decoded_value = url_decode(value);
                println!("🔍 QUERY_PARSE: Decoded value: '{}'", decoded_value);

                match key {
                    "from" => {
                        from = Some(decoded_value.clone());
                        println!("🔍 QUERY_PARSE: Set from = {:?}", from);
                    }
                    "to" => {
                        to = Some(decoded_value.clone());
                        println!("🔍 QUERY_PARSE: Set to = {:?}", to);
                    }
                    _ => {
                        println!("🔍 QUERY_PARSE: Ignoring unknown param: {}", key);
                    }
                }
            }
        }

        println!(
            "🔍 QUERY_PARSE: Final result - from: {:?}, to: {:?}",
            from, to
        );
        (from, to)
    } else {
        println!("🔍 QUERY_PARSE: No query string found, returning None values");
        (None, None)
    }
}

// Função simples de URL decode para suportar timestamps ISO
fn url_decode(input: &str) -> String {
    input
        .replace("%3A", ":")
        .replace("%2B", "+")
        .replace("%2D", "-")
        .replace("%2E", ".")
        .replace("%20", " ")
        .replace("+", " ")
}
