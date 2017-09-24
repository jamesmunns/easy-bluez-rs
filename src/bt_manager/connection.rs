use std::ops::DerefMut;
use std::time::{Duration, Instant};

use blurz::Adapter;
use basic_scheduler;

use bt_manager::AtomicBtDb;
use errors::*;
use whitelist::Whitelist;

pub struct ConnectionDb {
    pub db: AtomicBtDb,
    pub wl: Whitelist,
}

pub fn connect_task(data: &mut ConnectionDb) -> Option<basic_scheduler::Duration> {
    trace!("Connect Tick...");

    if let Ok(_) = data.manage_connection() {
        // data.manage_new_devices(devs);
        Some(basic_scheduler::Duration::seconds(3))
    } else {
        // An error has occurred, bail
        error!("Error, connect_task bailing");
        None
    }
}

impl ConnectionDb {
    pub fn manage_connection(&mut self) -> Result<()> {
        // Obtain locked inner data structure
        let mut db_l = self.db.lock().unwrap();
        let db = db_l.deref_mut();

        let mut drop_list = vec![];

        for (ref mac_addr, ref mut man_dev) in db.devices.iter_mut() {
            let too_idle = man_dev.last_connected.elapsed() > Duration::from_secs(30);

            if man_dev.bluez_handle.is_connected()? {
                man_dev.last_connected = Instant::now();

            // TODO: Pair with devices? Might be necessary for some behavior
            // if !man_dev.bluez_handle.is_paired()? {
            //     info!("Attempting to pair with {:?}", man_dev.bluez_handle);
            //     man_dev.bluez_handle.pair()?;
            // }
            } else if too_idle {
                warn!("Marking {:?} to be dropped", mac_addr);
                drop_list.push((*mac_addr).clone());
            } else {
                man_dev.connect();
            }
        }

        if drop_list.len() > 0 {
            let adapter: Adapter = Adapter::init()?;

            // Drop idle devices
            for d in drop_list {
                // TODO: a full remove may not be the best idea. If the device
                // does not gracefully handle re-pairing, it will fail to connect
                // until the device drops exisiting pair connections. Add to some
                // blacklist or slow-poll list?
                let rem_dev = db.devices.remove(&d).unwrap();
                adapter.remove_device(rem_dev.bluez_handle.get_id())?;
            }
        }

        Ok(())
    }
}
