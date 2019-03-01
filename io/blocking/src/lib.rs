extern crate magic_wormhole_core;
extern crate serde_json;
extern crate url;
extern crate ws;
extern crate hex;
extern crate jni;
#[macro_use] extern crate log;

#[cfg(target_os = "android")]
extern crate android_logger;

use log::Level;
#[cfg(target_os = "android")]
use android_logger::Filter;

mod blocking;
use magic_wormhole_core::message;
use magic_wormhole_core::{file_ack, message_ack, OfferType, PeerMessage};
use std::str;
pub use blocking::*;

// This is the interface to the JVM that we'll call the majority of our
// methods on.
use jni::JNIEnv;

// These objects are what you should use as arguments to your native
// function. They carry extra lifetime information to prevent them escaping
// this context and getting used after being GC'd.
use jni::objects::{JClass, JString};

// This is just a pointer. We'll be returning it from our function. We
// can't return one of the objects with lifetime information because the
// lifetime checker won't let us.
use jni::sys::jstring;

fn send(mailbox_server: String, app_id: String, msg: String) {
    let mut w = Wormhole::new(&app_id, &mailbox_server);
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

fn receive(mailbox_server: String, app_id: String, code: String) -> String {
    trace!("connecting..");
    let mut w = Wormhole::new(&app_id, &mailbox_server);
    w.set_code(&code);
    let verifier = w.get_verifier();
    trace!("verifier: {}", hex::encode(verifier));
    trace!("receiving..");
    let msg = w.get_message();
    let actual_message =
        PeerMessage::deserialize(str::from_utf8(&msg).unwrap());
    let remote_msg = match actual_message {
        PeerMessage::Offer(offer) => match offer {
            OfferType::Message(msg) => {
                trace!("{}", msg);
                w.send_message(message_ack("ok").serialize().as_bytes());
                msg.to_string()
            }
            OfferType::File { .. } => {
                trace!("Received file offer {:?}", offer);
                w.send_message(file_ack("ok").serialize().as_bytes());
                "".to_string()
            }
            OfferType::Directory { .. } => {
                trace!("Received directory offer: {:?}", offer);
                // TODO: We are doing file_ack without asking user
                w.send_message(file_ack("ok").serialize().as_bytes());
                "".to_string()
            }
        },
        PeerMessage::Answer(_) => {
            panic!("Should not receive answer type, I'm receiver")
        },
        PeerMessage::Error(err) => {
            trace!("Something went wrong: {}", err);
            "".to_string()
        },
        PeerMessage::Transit(transit) => {
            // TODO: This should start transit server connection or direct file transfer
            trace!("Transit Message received: {:?}", transit);
            "".to_string()
        }
    };
    trace!("closing..");
    w.close();
    trace!("closed");

    //let remote_msg = "foobar".to_string();
    remote_msg
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_leastauthority_wormhole_WormholeActivity_receive(env: JNIEnv,
                                                                                 _class: JClass,
                                                                                 server: JString,
                                                                                 appid: JString,
                                                                                 code: JString)
                                                                                 -> jstring {
    // First, we have to get the string out of Java. Check out the `strings`
    // module for more info on how this works.
    trace!("receiving ...");
    let jvm_server = env.get_string(server).expect("Couldn't get java string!").into();

    let jvm_appid = env.get_string(appid).expect("Couldn't get java string!").into();

    let jvm_code = env.get_string(code).expect("Couldn't get java string!").into();

    // Then we have to create a new Java string to return. Again, more info
    // in the `strings` module.
    let output = receive(jvm_server, jvm_appid, jvm_code);
    let joutput = env.new_string(output)
        .expect("Couldn't create java string!");
    // Finally, extract the raw pointer to return.
    joutput.into_inner()
}


#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
pub extern "system" fn Java_com_leastauthority_wormhole_WormholeActivity_init(_env: JNIEnv,
                                                                              _class: JClass) {
    android_logger::init_once(
        Filter::default().with_min_level(Level::Trace), Some("Wormhole"));
}

