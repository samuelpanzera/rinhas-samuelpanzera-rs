use rust_decimal::Decimal;
use std::io;

use crate::models::payment::{ApiPaymentRequest, PaymentData};
use crate::models::summary::ApiSummaryResponse;
use crate::repositories::payment::PaymentRepository;

pub struct PaymentService {
    payment_repository: PaymentRepository,
}

impl PaymentService {
    pub fn new(payment_repository: PaymentRepository) -> Self {
        Self { payment_repository }
    }

    pub fn process_payment(&self, payment_request: ApiPaymentRequest) -> io::Result<String> {
        // Validações de negócio
        if payment_request.amount <= Decimal::ZERO {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Amount must be greater than zero",
            ));
        }

        // Lógica para escolher o processador - retorna bool para máxima performance
        let is_default_processor = self.select_processor(&payment_request)?;

        // Cria a struct PaymentData com bool
        let payment_data = PaymentData::new(
            payment_request.correlation_id,
            payment_request.amount,
            is_default_processor,
        );

        self.payment_repository.save_payment(&payment_data)?;
        // Teria que enviar o pagamento para o processor...

        Ok("Pagamento processado".to_string())
    }

    // Este GET, vai precisar comunicar com outro serviço duplicado da api, um clone de tudo. 
    // Pensei em fazer uma comunicação RPC ou algo assim, mas por enquanto só vamos retornar o resumo

    pub fn get_payments_summary(&self) -> io::Result<ApiSummaryResponse> {
        let summary_data = self.payment_repository.get_payments_summary()?;

        let mut summary = ApiSummaryResponse::default();

        for (is_default_processor, total_requests, total_amount_cents) in summary_data {
            let total_amount = Decimal::new(total_amount_cents, 2);

            if is_default_processor {
                summary.default.total_requests = total_requests as u64;
                summary.default.total_amount = total_amount;
            } else {
                summary.fallback.total_requests = total_requests as u64;
                summary.fallback.total_amount = total_amount;
            }
        }

        Ok(summary)
    }

    fn select_processor(&self, _payment_request: &ApiPaymentRequest) -> io::Result<bool> {
        //Seu backend deverá se integrar com dois Processadores de Pagamento. Os dois serviços possuem API idênticas, então as descrições seguintes servem para os dois.
        // Aqui seria implementada a lógica de seleção do processador
        // Por exemplo: verificar saúde dos processadores, load balancing, etc.
        // Por enquanto, sempre retorna true (default processor)
        // Teremos um endpoint que podemos consumir:

        /*GET /payments/service-health
        HTTP 200 - Ok
        {
            "failing": false,
            "minResponseTime": 100
        }
         */
        // Limite de 1 chamada a cada 5 segundos
        Ok(true)
    }
}
