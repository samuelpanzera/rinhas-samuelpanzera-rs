use may_minihttp::Response;
use serde_json;
use std::io;

use crate::services::payment::PaymentService;

pub fn handle_payments_summary(
    res: &mut Response,
    payment_service: &PaymentService,
) -> io::Result<()> {
    match payment_service.get_payments_summary() {
        Ok(summary) => {
            res.header("Content-Type: application/json");
            let body = serde_json::to_vec(&summary).unwrap();
            res.body_mut().extend_from_slice(&body);
        }
        Err(_) => {
            res.status_code(500, "Internal Server Error")
                .body("Error processing summary");
        }
    }

    Ok(())
}
