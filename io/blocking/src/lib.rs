extern crate magic_wormhole_core;
extern crate serde_json;
extern crate url;
extern crate ws;
extern crate hex;

mod blocking;
use magic_wormhole_core::message;
use magic_wormhole_core::{file_ack, message_ack, OfferType, PeerMessage};
use std::str;
pub use blocking::*;

#[repr(C)]
pub struct Env {
    pub mailbox_server: String,
    pub app_id: String,
}

#[no_mangle]
pub fn send(env: Env, msg: String) {
    let mut w = Wormhole::new(&env.app_id, &env.mailbox_server);
    println!("connecting..");
    // w.set_code("4-purple-sausages");
    w.allocate_code(2);
    let code = w.get_code();
    println!("code is: {}", code);
    println!("sending..");
    w.send_message(message(&msg).serialize().as_bytes());
    println!("sent..");
    // if we close right away, we won't actually send anything. Wait for at
    // least the verifier to be printed, that ought to give our outbound
    // message a chance to be delivered.
    let verifier = w.get_verifier();
    println!("verifier: {}", hex::encode(verifier));
    println!("got verifier, closing..");
    w.close();
    println!("closed");
}

#[no_mangle]
pub fn receive(env: Env, code: String) -> String {
    let mut w = Wormhole::new(&env.app_id, &env.mailbox_server);
    println!("connecting..");
    w.set_code(&code);
    let verifier = w.get_verifier();
    println!("verifier: {}", hex::encode(verifier));
    println!("receiving..");
    let msg = w.get_message();
    let actual_message =
        PeerMessage::deserialize(str::from_utf8(&msg).unwrap());
    let remote_msg = match actual_message {
        PeerMessage::Offer(offer) => match offer {
            OfferType::Message(msg) => {
                println!("{}", msg);
                w.send_message(message_ack("ok").serialize().as_bytes());
                msg.to_string()
            }
            OfferType::File { .. } => {
                println!("Received file offer {:?}", offer);
                w.send_message(file_ack("ok").serialize().as_bytes());
                "".to_string()
            }
            OfferType::Directory { .. } => {
                println!("Received directory offer: {:?}", offer);
                // TODO: We are doing file_ack without asking user
                w.send_message(file_ack("ok").serialize().as_bytes());
                "".to_string()
            }
        },
        PeerMessage::Answer(_) => {
            panic!("Should not receive answer type, I'm receiver")
        },
        PeerMessage::Error(err) => {
            println!("Something went wrong: {}", err);
            "".to_string()
        },
        PeerMessage::Transit(transit) => {
            // TODO: This should start transit server connection or direct file transfer
            println!("Transit Message received: {:?}", transit);
            "".to_string()
        }
    };
    println!("closing..");
    w.close();
    println!("closed");

    remote_msg
}
