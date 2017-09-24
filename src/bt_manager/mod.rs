use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use blurz::{Device, GATTCharacteristic, GATTService};
use uuid::Uuid;

use whitelist::BtMacAddress;

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

pub struct ManagedDevice {
    pub bluez_handle: Device,
    pub last_connected: Instant,
    pub cached_data: CachedBtData,
    pub service_map: HashMap<Uuid, GATTService>,
    pub charac_map: HashMap<(Uuid, Uuid), GATTCharacteristic>,
}

impl ManagedDevice {
    pub fn new(dev: Device) -> Self {
        Self {
            bluez_handle: dev,
            last_connected: Instant::now(),
            cached_data: HashMap::new(),
            service_map: HashMap::new(),
            charac_map: HashMap::new(),
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
