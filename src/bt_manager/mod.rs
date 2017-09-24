use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use blurz::{Device, GATTCharacteristic, GATTService};
use uuid::Uuid;

use whitelist::BtMacAddress;

use std::str::FromStr;

pub mod discovery;
pub mod connection;
pub mod endpoints;
pub mod data_poll;


type AtomicBtDb = Arc<Mutex<BtDb>>;
type CachedBtData = HashMap<Uuid, HashMap<Uuid, Option<PolledData>>>;

#[derive(Debug)]
pub struct PolledData {
    time: Instant,
    data: Box<[u8]>,
}

pub struct BtDb {
    pub devices: HashMap<BtMacAddress, ManagedDevice>,
}

impl BtDb {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct SomethingItem {
    pub mac: BtMacAddress,
    pub svc: Uuid,
    pub chrc: Uuid,
}

pub struct ManagedDevice {
    pub bluez_handle: Device,
    pub mac_addr: BtMacAddress,
    pub last_connected: Instant,
}

impl ManagedDevice {
    pub fn new(dev: Device) -> Self {
        Self {
            mac_addr: BtMacAddress::from_str(&dev.get_address().unwrap()).unwrap(),
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
