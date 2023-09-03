use std::time::Instant;

use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};

use actix_web_actors::ws;
mod websocket_server;
mod websocket_session;

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<websocket_server::Server>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        websocket_session::WebSocketSession {
            id: 0,
            heartbeat: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = websocket_server::Server::new().start();

    println!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/ws", web::get().to(ws_route))
    })
    .workers(2)
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .await
}
