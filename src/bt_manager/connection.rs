use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use std::time::Duration as OldDuration;

use Duration;
use bt_manager::Connectable;

use errors::*;

pub struct ConnectionDb {
    pub incoming: Receiver<Connectable>,
    pub outgoing: Sender<(String, bool)>, // TODO better types

    // TODO sender for connect events
    // TODO sender for disconnect
    pub db: Vec<Connectable>,
    pub connect_interval: Duration,
}

pub fn connect_task(data: &mut ConnectionDb) -> Option<Duration> {
    trace!("Connect Tick...");

    match data.manage_connection() {
        Ok(()) => Some(data.connect_interval),
        Err(e) => {
            // An error has occurred, bail
            error!("Error, connect_task bailing, {:?}", e);
            None
        }
    }
}

impl ConnectionDb {
    pub fn manage_connection(&mut self) -> Result<()> {
        while let Ok(new_dev) = self.incoming.try_recv() {
            self.db.push(new_dev);
        }

        for man_dev in self.db.iter_mut() {
            let too_idle = man_dev.last_connected.elapsed() > OldDuration::from_secs(30);

            let is_connected = if man_dev.bluez_handle.is_connected().map_err(|e| e.to_string())? {
                trace!("{} is connected :)", man_dev.mac_addr);
                man_dev.last_connected = Instant::now();
                true

            // TODO: Pair with devices? Might be necessary for some behavior
            // if !man_dev.bluez_handle.is_paired()? {
            //     info!("Attempting to pair with {:?}", man_dev.bluez_handle);
            //     man_dev.bluez_handle.pair()?;
            // }
            } else {
                trace!("{} isn't connected :(", man_dev.mac_addr);
                if too_idle {
                    warn!("Device {} is missing!", man_dev.mac_addr);
                }
                man_dev.connect();
                false
            };

            self.outgoing
                .send((man_dev.mac_addr.clone(), is_connected))
                .chain_err(|| "failed to send!")?;
        }

        Ok(())
    }
}
