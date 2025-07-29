use may_minihttp::{HttpService, HttpServiceFactory, Request, Response};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::io;
use std::sync::Arc;

use crate::handlers::{payment, summary};
use crate::services::payment::PaymentService;
use crate::repositories::payment::PaymentRepository;

#[derive(Clone)]
pub struct RinhaBackend {
    payment_service: Arc<PaymentService>,
}

impl HttpService for RinhaBackend {
    fn call(&mut self, req: Request, res: &mut Response) -> io::Result<()> {
        match (req.method(), req.path()) {
            ("POST", "/payments") => {
                payment::handle_payment(req, res, &self.payment_service)
            }
            ("GET", "/payments-summary") => {
                summary::handle_payments_summary(res, &self.payment_service)
            }
            _ => {
                res.status_code(404, "Not Found");
                Ok(())
            }
        }
    }
}

pub struct RinhaBackendFactory {
    payment_service: Arc<PaymentService>,
}

impl RinhaBackendFactory {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        let payment_repository = PaymentRepository::new((*pool).clone());
        let payment_service = Arc::new(PaymentService::new(payment_repository));
        
        Self { payment_service }
    }
}

impl HttpServiceFactory for RinhaBackendFactory {
    type Service = RinhaBackend;

    fn new_service(&self, _: usize) -> Self::Service {
        RinhaBackend {
            payment_service: self.payment_service.clone(),
        }
    }
}