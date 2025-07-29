mod db;
mod handlers;
mod models;
mod repositories;
mod server;
mod services;

use db::create_optimized_pool;
use may_minihttp::HttpServiceFactory;
use server::RinhaBackendFactory;
use std::{env, sync::Arc};

fn main() {
    let db_path = "rinha.db";
    let pool = Arc::new(create_optimized_pool(db_path));
    let port = env::var("PORT").unwrap_or("14".to_string());
    // Cria o schema do banco, se não existir
    let conn = pool
        .get()
        .expect("Falha ao obter conexão para criar schema.");

    conn.execute_batch(include_str!("data/schema.sql"))
        .expect("Falha ao criar schema do banco de dados.");

    println!("Iniciando servidor na porta 14 usando corrotinas...");

    let server = RinhaBackendFactory::new(pool)
        .start(format!("0.0.0.0:{}", port))
        .unwrap();
    server.wait();
}
