extern crate easy_bluez;
use easy_bluez::EasyBluez;

use std::sync::mpsc::channel;

extern crate env_logger;

fn main() {
    env_logger::init().expect("Failed to initalize logging");
    let (tx, rx) = channel();

    let mut ez = EasyBluez::new()
        .run();

    let _hdl = ez.poll(
        "CF:75:CE:86:6D:02",                    // MAC Address
        "00000001-c001-de30-cabb-785feabcd123", // Service
        "0000c01d-c001-de30-cabb-785feabcd123", // Characteristic
        false,                                  // Does this device need to be paired?
        tx,                                     // Where does the data go?
    ).expect("Bad data!");

    // Block between messages
    while let Ok(data) = rx.recv() {
        println!("!!!: {:?}", data);
    }
}
