use std::io::{self, Read};
use may_minihttp::{Request, Response};
use serde_json;

use crate::models::payment::ApiPaymentRequest;
use crate::services::payment::PaymentService;

pub fn handle_payment(
    req: Request,
    res: &mut Response,
    payment_service: &PaymentService,
) -> io::Result<()> {
    let mut body = Vec::new();
    req.body().read_to_end(&mut body).unwrap();
    
    let payment_req: ApiPaymentRequest = serde_json::from_slice(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    match payment_service.process_payment(payment_req) {
        Ok(_) => {
            res.status_code(200, "OK").body("Pagamento processado");
        }
        Err(_) => {
            res.status_code(400, "Bad Request").body("Invalid payment request");
        }
    }
    
    Ok(())
}