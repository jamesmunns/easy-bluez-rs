use std::str::FromStr;
use std::thread;
use std::time::Duration as OldDuration;
use std::ops::DerefMut;
use std::sync::mpsc::{Receiver, Sender};
use std::collections::HashSet;

use blurz::{Adapter, Device, DiscoverySession};

use basic_scheduler::Duration;
use whitelist::BtMacAddress;
use bt_manager::Connectable;
use errors::*;

pub struct DiscoveryData {
    pub db: HashSet<BtMacAddress>,
    pub wl: HashSet<BtMacAddress>,
    pub receiver: Receiver<BtMacAddress>,
    pub sender_connect: Sender<Connectable>,
    pub sender_endpoints: Sender<(BtMacAddress, Device)>,
    pub scan_interval: Duration,
    pub scan_duration: Duration,
}

pub fn discovery_task(data: &mut DiscoveryData) -> Option<Duration> {
    trace!("Discovery Tick...");

    // Process any new whitelist items
    while let Ok(new_wl) = data.receiver.try_recv() {
        info!("Adding {:?}", new_wl);
        data.wl.insert(new_wl);
    }

    if data.wl.len() == 0 {
        // No whitelist items, no point in scanning
        warn!("No whitelist items, skipping scan");
        return Some(data.scan_interval);
    }

    if let Ok(devs) = data.discover_new() {
        data.manage_new_devices(devs);
        Some(data.scan_interval)
    } else {
        // An error has occurred, bail
        error!("Error, discovery_task bailing");
        None
    }
}

impl DiscoveryData {
    fn manage_new_devices(&mut self, devs: Vec<Device>) {
        // Avoid action if no devices found
        if devs.len() == 0 {
            return;
        }

        // Add each new device
        for d in devs {
            let btm = BtMacAddress::from_str(&d.get_address().unwrap()).unwrap();
            if !self.db.contains(&btm) {
                info!("Adding {:?}", btm);
                self.db.insert(btm.clone());

                // trigger a connect, and pass on for later handling
                let mut new_dev = Connectable::new(d.clone());
                new_dev.connect();
                self.sender_connect.send(new_dev).unwrap();
                self.sender_endpoints.send((btm, d)).unwrap();
            }
        }
    }

    fn discover_new(&self) -> Result<Vec<Device>> {
        let adapter: Adapter = Adapter::init()?;

        let session = DiscoverySession::create_session(adapter.get_id())?;
        thread::sleep(OldDuration::from_millis(200));

        session.start_discovery()?;
        thread::sleep(Duration::to_std(&self.scan_duration).chain_err(|| "")?);

        let mut devices = adapter.get_device_list()?;
        let mut new_devices = vec![];

        for d in devices.drain(..) {
            let device = Device::new(d);

            match device.get_address() {
                Ok(ref id) if self.wl.contains(&BtMacAddress::from_str(id).unwrap()) => {
                    trace!("Found device {} from whitelist", id);
                    new_devices.push(device);
                }
                Ok(ref id) => {
                    trace!("Ignoring device {} not on whitelist", id);
                }
                _ => {
                    trace!("Ignoring device with no id");
                }
            }
        }

        session.stop_discovery()?;

        Ok(new_devices)
    }
}
