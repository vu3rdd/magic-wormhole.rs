extern crate websocket;
extern crate futures;
extern crate tokio_core;

use tokio_core::reactor::Core;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use websocket::{ClientBuilder, OwnedMessage, Message};
use websocket::async::client::Client;
use websocket::async::TcpStream;

const MAILBOX_SERVER: &'static str = "ws://127.0.0.1:4000/v1"; // not localhost

/*

// Stream has 
*/

fn main() {
    println!("Connecting to {}", MAILBOX_SERVER);
    let mut core = Core::new().unwrap();

    let runner = ClientBuilder::new(MAILBOX_SERVER).unwrap()
        //.add_protocol("rust-websocket")
        .async_connect(None, &core.handle()) // future of (client,headers)
	.and_then(|(client, _headers)| {
            // client is Client<Stream+Send>
            &client.send(Message::text("{\"type\": \"bind\", \"appid\": \"rs\", \"side\": \"myside\"}").into());
	    let (sink, stream) = client.split();
            //let client: Client<TcpStream> = client;
            sink.send(Message::text("{\"type\": \"bind\", \"appid\": \"rs\", \"side\": \"myside\"}").into());
            //client.send(OwnedMessage::text("{\"type\": \"bind\", \"appid\": \"rs\", \"side\": \"myside\"}"));
            
	    stream.filter_map(|message| {
		println!("Received Message: {:?}", message);
		match message {
		    OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
		    OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
		    _ => None,
		}
	    })
		.forward(sink)
	});
    core.run(runner).unwrap();        
    println!("Hello, world!");
}
