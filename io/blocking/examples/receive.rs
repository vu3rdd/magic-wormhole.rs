extern crate hex;
extern crate magic_wormhole_core;
extern crate magic_wormhole_io_blocking;

use std::str;
use std::io;

// Can ws do hostname lookup? Use ip addr, not localhost, for now
const MAILBOX_SERVER: &'static str = "ws://relay.magic-wormhole.io:4000/v1";
const APPID: &'static str = "lothar.com/wormhole/text-or-file-xfer";

fn main() {
    let mailbox_server = String::from(MAILBOX_SERVER);
    let app_id = String::from(APPID);

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            match input.lines().next() {
                Some(line) => {
                    let msg = magic_wormhole_io_blocking::receive(mailbox_server, app_id, line.to_string());
                    println!("{}", msg);
                }
                None => ()
            }
        }
        Err(err) => println!("error: {}", err)
    }
}
