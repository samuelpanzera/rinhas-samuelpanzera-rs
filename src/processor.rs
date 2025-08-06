use chrono::Utc;
use rust_decimal::Decimal;
use serde_json;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use crate::models::payment::ProcessorPaymentRequest;
use crate::shared_mem::SharedMemoryManager;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct QueuedPayment {
    pub correlation_id: Uuid,
    pub amount: Decimal,
    pub amount_cents: u16,
}

pub struct PaymentProcessor {
    tx: mpsc::UnboundedSender<QueuedPayment>,
}

impl PaymentProcessor {
    pub fn new(shared_memory: Arc<SharedMemoryManager>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(Self::payment_worker(rx, shared_memory));
        });

        Self { tx }
    }

    pub fn queue_payment(
        &self,
        payment: QueuedPayment,
    ) -> Result<(), mpsc::error::SendError<QueuedPayment>> {
        self.tx.send(payment)
    }

    async fn payment_worker(
        mut rx: mpsc::UnboundedReceiver<QueuedPayment>,
        shared_memory: Arc<SharedMemoryManager>,
    ) {
        while let Some(payment) = rx.recv().await {
            let mut success = false;

            for attempt in 1..=5 {
                match Self::call_processor(&payment, true, &shared_memory).await {
                    Ok(_) => {
                        success = true;
                        break;
                    }
                    Err(_) => {
                        if attempt < 5 {
                            sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            }

            if !success {
                let _ = Self::call_processor(&payment, false, &shared_memory).await;
            }
        }
    }

    async fn call_processor(
        payment: &QueuedPayment,
        is_default: bool,
        shared_memory: &Arc<SharedMemoryManager>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let base_url = if is_default {
            env::var("PAYMENT_PROCESSOR_URL_DEFAULT")
                .unwrap_or_else(|_| "http://localhost:8001".to_string())
        } else {
            env::var("PAYMENT_PROCESSOR_URL_FALLBACK")
                .unwrap_or_else(|_| "http://localhost:8002".to_string())
        };

        let url = format!("{}/payments", base_url);

        let requested_at = Utc::now();

        let processor_request = ProcessorPaymentRequest {
            correlation_id: &payment.correlation_id,
            amount: &payment.amount,
            requested_at,
        };

        let json_body = serde_json::to_string(&processor_request)?;

        let client = reqwest::Client::new();
        let response = timeout(
            Duration::from_secs(10),
            client
                .post(url)
                .header("Content-Type", "application/json")
                .body(json_body)
                .send(),
        )
        .await??;

        if response.status().is_success() {
            shared_memory.add_payment_if_new(
                payment.correlation_id,
                payment.amount_cents,
                is_default,
                requested_at,
            );
            Ok(())
        } else {
            Err(format!("HTTP Error: {}", response.status()).into())
        }
    }
}
