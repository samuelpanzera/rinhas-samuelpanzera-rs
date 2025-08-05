mod handlers;
mod models;
mod processor;
mod server;
mod shared_mem;

use may_minihttp::HttpServiceFactory;
use server::RinhaBackendFactory;
use std::env;

fn main() {
    let port = env::var("PORT").unwrap_or("14".to_string());

    println!(
        "🚀 Iniciando servidor na porta {} usando corrotinas...",
        port
    );
    let factory = RinhaBackendFactory::new();

    let server = factory
        .start(format!("0.0.0.0:{}", port))
        .expect("Falha ao iniciar o servidor");

    println!(
        "✅ Servidor iniciado com sucesso no endereço 0.0.0.0:{}",
        port
    );
    server.wait();
}
