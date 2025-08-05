use may_minihttp::{HttpService, HttpServiceFactory, Request, Response};
use std::io;
use std::sync::Arc;

use crate::handlers::{payment, summary};
use crate::processor::PaymentProcessor;
use crate::shared_mem::SharedMemoryManager;

#[derive(Clone)]
pub struct RinhaBackend {
    payment_processor: Arc<PaymentProcessor>,
    shared_memory: Arc<SharedMemoryManager>,
}

impl HttpService for RinhaBackend {
    fn call(&mut self, req: Request, res: &mut Response) -> io::Result<()> {
        let path_without_query = req.path().split('?').next().unwrap_or(req.path());

        match (req.method(), path_without_query) {
            ("POST", "/payments") => {
                payment::handle_payment(req, res, &self.payment_processor, &self.shared_memory)
            }
            ("GET", "/payments-summary") => {
                summary::handle_payments_summary(req, res, &self.shared_memory)
            }
            _ => {
                res.status_code(404, "Not Found");
                Ok(())
            }
        }
    }
}

pub struct RinhaBackendFactory {
    payment_processor: Arc<PaymentProcessor>,
    shared_memory: Arc<SharedMemoryManager>,
}

impl RinhaBackendFactory {
    pub fn new() -> Self {
        let shared_memory =
            Arc::new(SharedMemoryManager::new().expect("Failed to create SharedMemoryManager"));

        let payment_processor = Arc::new(PaymentProcessor::new(shared_memory.clone()));

        Self {
            payment_processor,
            shared_memory,
        }
    }
}

impl HttpServiceFactory for RinhaBackendFactory {
    type Service = RinhaBackend;

    fn new_service(&self, _: usize) -> Self::Service {
        RinhaBackend {
            payment_processor: self.payment_processor.clone(),
            shared_memory: self.shared_memory.clone(),
        }
    }
}
