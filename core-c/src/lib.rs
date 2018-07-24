extern crate magic_wormhole_core;

use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use magic_wormhole_core::{
    WormholeCore
};

const MAILBOX_SERVER: &'static str = "ws://127.0.0.1:4000/v1";
const APPID: &'static str = "lothar.com/wormhole/text-or-file-xfer";

#[no_mangle]
pub extern fn wormhole_core_new(appid: *const c_char, relay_url: *const c_char) -> WormholeCore {
    WormholeCore::new(APPID, MAILBOX_SERVER)
}

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    CString::new("Hello ".to_owned() + recipient).unwrap().into_raw()
}

#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;
    //extern crate magic_wormhole_core;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::{JClass, JString};
    use self::jni::sys::{jstring, jobject};

    #[no_mangle]
    pub unsafe extern fn Java_com_leastauthority_RustGreetings_greeting(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
        // Our Java companion code might pass-in "world" as a string, hence the name.
        let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
        // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
        let world_ptr = CString::from_raw(world);
        let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

        output.into_inner()
    }

    #[no_mangle]
    pub unsafe extern fn Java_com_leastauthority_wormholecore_new(env: JNIEnv, _: JClass) -> magic_wormhole_core::WormholeCore {
        wormhole_core_new("foo".as_ptr(), "bar".as_ptr())
    }
}
