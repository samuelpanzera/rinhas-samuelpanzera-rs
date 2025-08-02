mod handlers;
mod models;
mod processor;
mod server;
mod shared_mem;

// Removed db module - using simplified approach
use may_minihttp::HttpServiceFactory;
use server::RinhaBackendFactory;
use std::env;

fn main() {
    let port = env::var("PORT").unwrap_or("14".to_string());

    println!(
        "🚀 Iniciando servidor na porta {} usando corrotinas...",
        port
    );
    println!("🔍 DEBUG: Rotas disponíveis:");
    println!("  - POST /payments");
    println!("  - GET /payments-summary");

    let factory = RinhaBackendFactory::new();
    println!("🔍 DEBUG: RinhaBackendFactory criado com sucesso");

    let server = factory
        .start(format!("0.0.0.0:{}", port))
        .expect("Falha ao iniciar o servidor");

    println!(
        "✅ Servidor iniciado com sucesso no endereço 0.0.0.0:{}",
        port
    );
    server.wait();
}
