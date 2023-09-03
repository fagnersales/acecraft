use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::websocket_server::{self, Server};

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

pub struct WebSocketSession {
    pub heartbeat: Instant,
    pub id: usize,
    pub addr: Addr<Server>,
}

impl WebSocketSession {
    fn start_heartbeating(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<WebSocketSession>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeating(ctx);

        let addr = ctx.address();

        self.addr
            .send(websocket_server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,

                    _ => ctx.stop(),
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr
            .do_send(websocket_server::Disconnect { id: self.id });

        Running::Stop
    }
}

impl Handler<websocket_server::Message> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, message_received: websocket_server::Message, ctx: &mut Self::Context) {
        ctx.text(message_received.0)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(
        &mut self,
        message_received: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let message = match message_received {
            Err(_) => {
                ctx.stop();
                return;
            }

            Ok(message) => message,
        };

        match message {
            ws::Message::Ping(ping_message) => {
                self.heartbeat = Instant::now();
                ctx.pong(&ping_message);
            }

            ws::Message::Pong(_) => {
                self.heartbeat = Instant::now();
            }

            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop()
            }

            ws::Message::Text(text) => self.addr.do_send(websocket_server::ClientMessage {
                id: self.id,
                message: text.to_owned().to_string(),
            }),

            _ => (),
        }
    }
}
