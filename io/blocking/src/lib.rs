extern crate magic_wormhole_core;
extern crate serde_json;
extern crate url;
extern crate ws;
extern crate hex;
extern crate jni;
#[macro_use] extern crate log;
extern crate sha2;
extern crate sodiumoxide;

#[cfg(target_os = "android")]
extern crate android_logger;

extern crate byteorder;

use log::Level;
#[cfg(target_os = "android")]
use android_logger::Filter;

mod blocking;
use magic_wormhole_core::message;
use magic_wormhole_core::{file_ack, message_ack, OfferType, PeerMessage};
use std::str;
pub use blocking::*;
use std::panic;

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
use jni::sys::{jstring, jlong};

pub fn connect(app_id: String, mailbox_server: String) -> Wormhole {
    Wormhole::new(&app_id.as_str(), &mailbox_server.as_str())
}

pub fn get_code(w: &mut Wormhole) -> String {
    w.allocate_code(2);
    w.get_code()
}

pub fn send(w: &mut Wormhole, code: String, msg: String) {
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

pub fn receive(mailbox_server: String, app_id: String, code: String) -> String {
    trace!("connecting..");
    let mut w = connect(app_id.clone(), mailbox_server); //Wormhole::new(&app_id, &mailbox_server);
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
            // first derive a transit key.
            let k = w.derive_transit_key(&app_id);
            println!("Transit Message received: {:?}", transit);
            w.receive_file(&k, transit);
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
    let output = panic::catch_unwind(|| {
        receive(jvm_server, jvm_appid, jvm_code)
    });

    match output {
        Ok(msg) => {
            let joutput = env.new_string(msg.to_string())
                .expect("Couldn't create java string!");
            // Finally, extract the raw pointer to return.
            return joutput.into_inner();
        },
        Err(e) => {
            let _ = env.throw(("java/lang/Exception", format!("{:?}", e)));
            let joutput = env.new_string("".to_string())
                .expect("Couldn't create java string!");
            // Finally, extract the raw pointer to return.
            return joutput.into_inner();
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
#[cfg(target_os = "android")]
pub extern "system" fn Java_com_leastauthority_wormhole_WormholeActivity_init(_env: JNIEnv,
                                                                              _class: JClass) {
    android_logger::init_once(
        Filter::default().with_min_level(Level::Trace), Some("Wormhole"));
}


#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_leastauthority_wormhole_WormholeActivity_connect(env: JNIEnv,
                                                                                 _class: JClass,
                                                                                 appid: JString,
                                                                                 server: JString)
                                                                                 -> jlong {
    let jvm_server = env.get_string(server).expect("Couldn't get java string!").into();
    let jvm_appid = env.get_string(appid).expect("Couldn't get java string!").into();

    let ptr = connect(jvm_appid, jvm_server);

    Box::into_raw(Box::new(ptr)) as jlong
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_leastauthority_wormhole_WormholeActivity_getcode(env: JNIEnv,
                                                                                        _class: JClass,
                                                                                        ptr: jlong)
                                                                                        -> jstring {
    let w = &mut *(ptr as *mut Wormhole);
    let output = get_code(w); //jvm_ptr as &mut blocking::Wormhole);

    let joutput = env.new_string(output)
        .expect("Couldn't create java string!");
    // Finally, extract the raw pointer to return.
    return joutput.into_inner();
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_com_leastauthority_wormhole_WormholeActivity_send(env: JNIEnv,
                                                                                     _class: JClass,
                                                                                     ptr: jlong,
                                                                                     code: JString,
                                                                                     msg: JString) {
    // First, we have to get the string out of Java. Check out the `strings`
    // module for more info on how this works.
    trace!("sending ...");
    let jvm_code = env.get_string(code).expect("Couldn't get java string!").into();
    let jvm_message = env.get_string(msg).expect("Couldn't get java string!").into();

    let w = &mut *(ptr as *mut Wormhole);
    send(w, jvm_code, jvm_message);
}
