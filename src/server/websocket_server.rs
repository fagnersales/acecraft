use std::collections::HashMap;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
  pub id: usize,
  pub message: String,
}

#[derive(Debug)]
pub struct Server {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl Server {
    pub fn new() -> Self {
        Server {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Server {
    fn broadcast_message(&mut self, message: &str, skip_id: usize) {
        for (id, _recipient) in &self.sessions {
          if *id != skip_id {
            if let Some(addr) = self.sessions.get(&id) {
              addr.do_send(Message(message.to_owned()));
            }
          }
        }
    }
}

impl Handler<Connect> for Server {
    type Result = usize;

    fn handle(&mut self, message_received: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, message_received.addr);
        id
    }
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, message_received: Disconnect, _ctx: &mut Context<Self>) -> Self::Result {
        self.sessions.remove(&message_received.id);
    }
}

impl Handler<ClientMessage> for Server {
  type Result = ();
  
  fn handle(&mut self, message_received: ClientMessage, _ctx: &mut Context<Self>) -> () {
    self.broadcast_message(message_received.message.as_str(), message_received.id);
  }
}