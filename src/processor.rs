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
    pub retry_count: u8,
    // processed_at removido - será calculado no momento do processamento real
}

pub struct PaymentProcessor {
    tx: mpsc::UnboundedSender<QueuedPayment>,
}

impl PaymentProcessor {
    pub fn new(shared_memory: Arc<SharedMemoryManager>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx_clone = tx.clone(); // Clone for re-queuing

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(Self::payment_worker(rx, shared_memory, tx_clone));
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
        tx: mpsc::UnboundedSender<QueuedPayment>, // For re-queuing
    ) {
        while let Some(mut payment) = rx.recv().await {
            println!("🔄 Processing payment: {}", payment.correlation_id);

            let mut success = false;

            // Try default processor 5 times with 500ms intervals
            for attempt in 1..=5 {
                println!(
                    "🎯 Default processor attempt {}/5 for payment {}",
                    attempt, payment.correlation_id
                );

                match Self::call_processor(&payment, true).await {
                    Ok(_) => {
                        println!(
                            "✅ Default processor success for payment {}",
                            payment.correlation_id
                        );
                        Self::save_successful_payment(&payment, true, &shared_memory).await;
                        success = true;
                        break;
                    }
                    Err(e) => {
                        println!("❌ Default processor attempt {} failed: {}", attempt, e);
                        if attempt < 5 {
                            sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            }

            if !success {
                println!(
                    "🔄 Trying fallback processor for payment {}",
                    payment.correlation_id
                );

                match Self::call_processor(&payment, false).await {
                    Ok(_) => {
                        println!(
                            "✅ Fallback processor success for payment {}",
                            payment.correlation_id
                        );
                        Self::save_successful_payment(&payment, false, &shared_memory).await;
                        success = true;
                    }
                    Err(e) => {
                        println!("❌ Fallback processor failed: {}", e);
                    }
                }
            }

            if !success {
                payment.retry_count += 1;
                if payment.retry_count < 3 {
                    // Limit retries to prevent infinite loops
                    println!(
                        "🔁 Re-queuing payment {} (retry {})",
                        payment.correlation_id, payment.retry_count
                    );

                    // Add delay before re-queuing to avoid overwhelming the processors
                    sleep(Duration::from_secs(5)).await;

                    if let Err(e) = tx.send(payment) {
                        println!("❌ Failed to re-queue payment: {}", e);
                    }
                } else {
                    println!(
                        "💀 Payment {} failed after maximum retries",
                        payment.correlation_id
                    );
                }
            }
        }
    }

    async fn call_processor(
        payment: &QueuedPayment,
        is_default: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = if is_default {
            env::var("PAYMENT_PROCESSOR_URL_DEFAULT")
                .unwrap_or_else(|_| "http://localhost:8001".to_string())
        } else {
            env::var("PAYMENT_PROCESSOR_URL_FALLBACK")
                .unwrap_or_else(|_| "http://localhost:8002".to_string())
        };

        let url = format!("{}/payments", base_url);

        // Calcula timestamp no momento da requisição ao processador
        let requested_at = Utc::now();

        let processor_request = ProcessorPaymentRequest {
            correlation_id: &payment.correlation_id,
            amount: &payment.amount,
            requested_at, // ← Timestamp do momento da requisição
        };

        let json_body = serde_json::to_string(&processor_request)?;

        println!("🅰️ Sending request to {} with body: {}", url, json_body);

        // Using tokio's HTTP client for maximum efficiency
        let client = reqwest::Client::new();

        // 10 second timeout for HTTP requests
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
            Ok(response.text().await?)
        } else {
            Err(format!("HTTP Error: {}", response.status()).into())
        }
    }

    async fn save_successful_payment(
        payment: &QueuedPayment,
        is_default_processor: bool,
        shared_memory: &Arc<SharedMemoryManager>,
    ) {
        // 🕐 CALCULA TIMESTAMP NO MOMENTO REAL DO PROCESSAMENTO BEM-SUCEDIDO
        let processed_at = Utc::now();

        println!(
            "⏰ Processing timestamp calculated: {} for payment {}",
            processed_at, payment.correlation_id
        );

        // Add to shared memory with timestamp calculado no momento do sucesso
        shared_memory.add_payment(
            payment.amount_cents,
            is_default_processor,
            processed_at, // ← Timestamp do momento real do processamento
        );
        println!(
            "📊 Payment added to shared memory: {} at {}",
            payment.correlation_id, processed_at
        );
    }
}
