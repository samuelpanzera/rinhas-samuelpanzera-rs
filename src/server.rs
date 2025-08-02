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
        println!("🔍 Received request: {} {}", req.method(), req.path());

        // Extract path without query parameters for routing
        let path_without_query = req.path().split('?').next().unwrap_or(req.path());
        println!("🔍 Path for routing: '{}'", path_without_query);

        match (req.method(), path_without_query) {
            ("POST", "/payments") => {
                println!("✅ Routing to payments handler");
                payment::handle_payment(req, res, &self.payment_processor, &self.shared_memory)
            }
            ("GET", "/payments-summary") => {
                println!("✅ Routing to payments-summary handler");
                summary::handle_payments_summary(req, res, &self.shared_memory)
            }
            _ => {
                println!(
                    "❌ Route not found: {} {} (path: '{}')",
                    req.method(),
                    req.path(),
                    path_without_query
                );
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
        // Create shared memory manager
        let shared_memory =
            Arc::new(SharedMemoryManager::new().expect("Failed to create SharedMemoryManager"));

        // Create payment processor with queue
        // ✅ CORREÇÃO: Passa Arc diretamente em vez de clone que limpava dados
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
