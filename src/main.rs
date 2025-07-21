mod models;
mod db;
use models::payment::ApiPaymentRequest;

use may_minihttp::{HttpService, HttpServiceFactory, Request, Response};
use serde_json::json;
use std::io;
use std::time::Duration;

#[derive(Clone)]
struct RinhaBackend;

impl HttpService for RinhaBackend {
    fn call(&mut self, req: Request, res: &mut Response) -> io::Result<()> {
        // Cada chamada a `call` já está rodando em sua própria corrotina.
        // Operações de I/O aqui (como req.body() ou chamadas de rede)
        // não bloquearão outras requisições de serem processadas.

        match (req.method(), req.path()) {
            ("POST", "/payments") => {
                // A leitura do corpo da requisição é uma operação de I/O.
                // O `may` vai gerenciar a concorrência eficientemente aqui.
                let body = req.body();
                let payment_req: ApiPaymentRequest = serde_json::from_reader(body)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

                println!(
                    "Processando pagamento: correlationId={}, amount={}",
                    payment_req.correlation_id, payment_req.amount
                );

                // --- LÓGICA DE CONCORRÊNCIA ---
                // Aqui você faria as chamadas para os Payment Processors.
                // Exemplo:
                // let response_default = call_payment_processor("http://payment-processor-default:8080/payments", &payment_req);
                //
                // Se o de cima falhar ou demorar muito, você pode tentar o fallback.
                // Você pode até mesmo fazer isso em paralelo usando `go!` se a estratégia permitir.
                // `may::coroutine::sleep` é uma função não-bloqueante, ótima para simular latência.
                may::coroutine::sleep(Duration::from_millis(50)); // Simula trabalho

                res.status_code(200, "OK").body("Pagamento processado");
            }

            ("GET", "/payments-summary") => {
                println!("Recebido GET em /payments-summary");

                // A consulta ao banco de dados 
                // também seria uma operação não-bloqueante.
                let summary = json!({
                    "default": {
                        "totalRequests": 10, // valor de exemplo
                        "totalAmount": 199.90 // valor de exemplo
                    },
                    "fallback": {
                        "totalRequests": 2, // valor de exemplo
                        "totalAmount": 39.80 // valor de exemplo
                    }
                });

                res.header("Content-Type: application/json");
                res.body_mut()
                    .extend_from_slice(summary.to_string().as_bytes());
            }

            _ => {
                res.status_code(404, "Not Found");
            }
        }
        Ok(())
    }
}

struct RinhaBackendFactory;

impl HttpServiceFactory for RinhaBackendFactory {
    type Service = RinhaBackend;

    fn new_service(&self, _: usize) -> Self::Service {
        RinhaBackend
    }
}

fn main() {
    println!("Iniciando servidor na porta 9999 usando corrotinas...");

    let server = RinhaBackendFactory.start("0.0.0.0:9999").unwrap();
    server.wait();
}
