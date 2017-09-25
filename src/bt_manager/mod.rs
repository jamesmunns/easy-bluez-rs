use std::time::Instant;

use blurz::Device;
use uuid::Uuid;
use BtMacAddress;

pub mod discovery;
pub mod connection;
pub mod endpoints;
pub mod data_poll;
pub mod data_write;


#[derive(Debug)]
pub struct SomethingItem {
    pub mac: BtMacAddress,
    pub svc: Uuid,
    pub chrc: Uuid,
}


pub struct Connectable {
    pub bluez_handle: Device,
    pub mac_addr: String,
    pub last_connected: Instant,
}

impl Connectable {
    pub fn new(dev: Device) -> Self {
        Self {
            mac_addr: dev.get_address().unwrap_or(String::from("Unknown")),
            bluez_handle: dev,
            last_connected: Instant::now(),
        }
    }

    /// Attempt to initiate a connect
    pub fn connect(&mut self) {
        debug!("Attempting to connect to {:?}", self.bluez_handle);
        if let Ok(_) = self.bluez_handle.connect() {
            info!("connected to {:?}", self.bluez_handle);
            self.last_connected = Instant::now();
        }
    }
}
