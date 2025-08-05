use may_minihttp::{Request, Response};
use rust_decimal::prelude::*;
use serde_json;
use std::io::{self, Read};

use crate::models::payment::ApiPaymentRequest;
use crate::processor::{PaymentProcessor, QueuedPayment};
use crate::shared_mem::SharedMemoryManager;

pub fn handle_payment(
    req: Request,
    res: &mut Response,
    payment_processor: &PaymentProcessor,
    _shared_memory: &SharedMemoryManager,
) -> io::Result<()> {
    let mut body = Vec::with_capacity(256);
    req.body().read_to_end(&mut body).unwrap();

    let payment_req: ApiPaymentRequest = serde_json::from_slice(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let amount_cents = (payment_req.amount * Decimal::from(100))
        .to_u64()
        .unwrap_or(0) as u16;

    let queued_payment = QueuedPayment {
        correlation_id: payment_req.correlation_id,
        amount: payment_req.amount,
        amount_cents,
        retry_count: 0,
    };

    match payment_processor.queue_payment(queued_payment) {
        Ok(_) => {
            res.status_code(202, "Accepted");
        }
        Err(_) => {
            res.status_code(503, "Service Unavailable");
        }
    }

    Ok(())
}
