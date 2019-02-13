extern crate hex;
extern crate magic_wormhole_core;
extern crate magic_wormhole_io_blocking;

use std::str;
use magic_wormhole_io_blocking::Env;

// Can ws do hostname lookup? Use ip addr, not localhost, for now
const MAILBOX_SERVER: &'static str = "ws://127.0.0.1:4000/v1";
const APPID: &'static str = "lothar.com/wormhole/text-or-file-xfer";

fn main() {
    let env = Env {
        mailbox_server: String::from(MAILBOX_SERVER),
        app_id: String::from(APPID)
    };

    let msg = magic_wormhole_io_blocking::receive(env, "4-purple-sausages".to_string());
    
    println!("{}", msg);
}
