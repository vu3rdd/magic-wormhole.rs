
// extern crate magic_wormhole_core;
extern crate magic_wormhole_io_blocking;

// Can ws do hostname lookup? Use ip addr, not localhost, for now
const MAILBOX_SERVER: &'static str = "ws://relay.magic-wormhole.io:4000/v1";
const APPID: &'static str = "lothar.com/wormhole/text-or-file-xfer";

fn main() {
    let mailbox_server = String::from(MAILBOX_SERVER);
    let app_id = String::from(APPID);

    let mut w = magic_wormhole_io_blocking::connect(app_id, mailbox_server);
    let code = magic_wormhole_io_blocking::get_code(&mut w);
    println!("got the code: {}", code);
    magic_wormhole_io_blocking::send(&mut w, code, "hello from rust".to_string());
}
