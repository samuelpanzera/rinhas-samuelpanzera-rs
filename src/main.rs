use may_minihttp::{HttpServer, HttpService, Request, Response};
use std::env;
use std::io;

#[derive(Clone)]
struct ServerIdentifier;

impl HttpService for ServerIdentifier {
    fn call(&mut self, req: Request, rsp: &mut Response) -> io::Result<()> {
        if req.path() == "/health" {
            rsp.status_code(200, "OK").body_vec("healthy".into());
            return Ok(());
        }

        let server_id = env::var("SERVER_ID").unwrap_or_else(|_| "ID_Nao_Definido".to_string());
        let response_body = format!("Servidor respondendo com ID: {}\n", server_id);

        rsp.header("Content-Type: text/plain")
            .body_vec(response_body.into_bytes());

        Ok(())
    }
}

fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "1313".to_string());
    let address = format!("0.0.0.0:{}", port);

    let server = HttpServer(ServerIdentifier).start(&address).unwrap();

    let server_id = env::var("SERVER_ID").unwrap_or_else(|_| "ID_Nao_Definido".to_string());
    println!("Servidor {} iniciado e escutando em {}", server_id, address);

    server.wait();
}
