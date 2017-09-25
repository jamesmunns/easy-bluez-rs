extern crate easy_bluez;
use easy_bluez::EasyBluez;

use std::sync::mpsc::channel;

extern crate env_logger;

fn main() {
    env_logger::init().expect("Failed to initalize logging");
    let (tx_poll, rx_poll) = channel();
    let (tx_write, rx_write) = channel();

    let mut ez = EasyBluez::new().run();

    ez.poll(
        "CF:75:CE:86:6D:02",                    // MAC Address
        "00000001-c001-de30-cabb-785feabcd123", // Service
        "0000c01d-c001-de30-cabb-785feabcd123", // Characteristic
        false,                                  // Does this device need to be paired?
        tx_poll,                                // Where does the data go?
    ).expect("Bad data!");

    ez.writeable(
        "CF:75:CE:86:6D:02",                    // MAC Address
        "00000001-c001-de30-cabb-785feabcd123", // Service
        "0000da7a-c001-de30-cabb-785feabcd123", // Characteristic
        rx_write,                               // Where does the data come from?
    ).expect("Bad data!");

    let mut ctr = 0u8;

    // Block between messages
    while let Ok(data) = rx_poll.recv() {
        println!("!!!: {:?}", data);
        ctr += 1;
        // Every other incoming message, send a message to toggle the LED
        match ctr {
            2 => {
                tx_write.send(Box::new([0x00])).unwrap();
            }
            4 => {
                tx_write.send(Box::new([0x01])).unwrap();
                ctr = 0;
            }
            _ => {}
        }
    }
}
