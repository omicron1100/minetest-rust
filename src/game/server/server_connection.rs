use std::{collections::HashMap, net::ToSocketAddrs, time::Duration};

use message_io::{
  events::EventReceiver,
  network::{Endpoint, Transport},
  node::{self, NodeHandler, NodeTask, StoredNetEvent, StoredNodeEvent},
};



///
/// ServerConnection and Server can be considered 1 entity.
///
/// This is why server_pointer is not an Option<>.
///
pub struct ServerConnection {
  address: String,
  port: i32,

  task: Option<NodeTask>,
  handler: Option<NodeHandler<()>>,
  event_receiver: Option<EventReceiver<StoredNodeEvent<()>>>,
  clients: HashMap<Endpoint, String>,
}

impl ServerConnection {
  pub fn new(address: String, port: i32) -> Self {
    let mut new_server_connection = ServerConnection {
      address,
      port,

      task: None,
      handler: None,
      event_receiver: None,
      clients: HashMap::new(),
    };

    new_server_connection.initialize();

    new_server_connection
  }

  ///
  /// Change the address that the server connection will utilize.
  ///
  pub fn set_address(&mut self, new_address: String) {
    self.address = new_address;
  }

  ///
  /// Change the port that the server connection will utilize.
  ///
  pub fn set_port(&mut self, new_port: i32) {
    self.port = new_port;
  }

  ///
  /// Construct the address & port into a parsable socket string.
  ///
  pub fn get_socket(&self) -> String {
    let mut socket = self.address.clone();
    socket.push(':');
    socket.push_str(self.port.to_string().as_str());

    socket
  }

  ///
  /// Send raw data to an EndPoint (ClientConnection).
  ///
  fn send_data(&self, end_point: Endpoint, data: &str) {
    self
      .handler
      .clone()
      .unwrap()
      .network()
      .send(end_point, data.as_bytes());
  }

  ///
  /// A procedure to react to a network event.
  ///
  /// Returns if the connection received the shutdown signal from a client
  ///
  pub fn event_reaction(&mut self, event: StoredNetEvent) -> bool {
    let mut term_signal = false;
    // We don't need to match, we're using UDP which is connectionless.
    if let StoredNetEvent::Message(end_point, raw_message) = event {
      // todo: use https://github.com/serde-rs/bytes
      let receieved_string = match String::from_utf8(raw_message) {
        Ok(new_string) => new_string,
        Err(_) => {
          println!("minetest: message buffer attack detected, bailing on deserialization!");
          "".to_string()
        }
      };

      println!("minetest: Server received message: {}", receieved_string);

      match receieved_string.as_str() {
        "hi" => self.send_data(end_point, "hi there!"),
        "MINETEST_HAND_SHAKE" => self.send_data(end_point, "MINETEST_HAND_SHAKE_CONFIRMED"),
        // ! Oh yeah, just accept any shut down request.
        // ! I'm sure there's no way this can go wrong.
        // ! If it's not obvious [THIS IS DEBUGGING]
        "MINETEST_SHUT_DOWN_REQUEST" => {
          term_signal = true;
          println!("minetest: shutdown request received! Shutting down [now].");
        }
        _ => (),
      }
    }

    term_signal
  }

  ///
  /// Non-blocking event receiver for network events.
  ///
  /// Returns the EndPoint (ClientConnection) that requested to shut
  /// down the server.
  ///
  /// ! Return is UNSAFE at the moment.
  ///
  pub fn receive(&mut self) -> Option<Endpoint> {
    let mut term_signal: Option<Endpoint> = None;
    let mut has_new_event = true;
    // We want to grind through ALL the events.
    while has_new_event {
      match &mut self.event_receiver {
        Some(event_receiver) => {
          if let Some(event) = event_receiver.receive_timeout(Duration::new(0, 0)) {
            match event {
              StoredNodeEvent::Network(new_event) => {
                if self.event_reaction(new_event.clone()) {
                  if let StoredNetEvent::Message(term_end_point, _) = new_event.clone() {
                    term_signal = Some(term_end_point);
                  };
                }
              }
              // todo: figure out what a signal is!
              StoredNodeEvent::Signal(_) => todo!(),
            }
          } else {
            has_new_event = false;
          }
        }
        None => panic!("minetest: ServerConnection listener does not exist!"),
      }
    }
    term_signal
  }

  ///
  /// Internal initializer procedure automatically run on a new ServerConnection.
  ///
  fn initialize(&mut self) {
    let socket_address = self.get_socket().to_socket_addrs().unwrap().next().unwrap();
    let transport_protocol = Transport::Udp;

    let (handler, listener) = node::split::<()>();

    // todo: fixme: this is udp, why are we doing a match here?
    // todo: If this fails, the server probably doesn't have a network
    // todo: adapter! Why is it a server?!
    match handler.network().listen(transport_protocol, socket_address) {
      Ok((id, real_address)) => {
        println!(
          "minetest: connection created at id [{}], real address [{}]",
          id, real_address
        );
      }
      Err(e) => panic!("{}", e),
    }

    let (task, event_receiver) = listener.enqueue();
    self.handler = Some(handler);
    self.task = Some(task);
    self.event_receiver = Some(event_receiver);
  }
}

impl Drop for ServerConnection {
  fn drop(&mut self) {
    // Need to close server connection, maybe?
    // Might need to send out the disconnect signal.
    // todo: experiment with this.
    println!("ServerConnection dropped!")
  }
}
