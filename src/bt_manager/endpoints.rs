use std::str::FromStr;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use blurz::{Device, GATTCharacteristic, GATTService};
use uuid::Uuid;

use Duration;
use BtMacAddress;
use bt_manager::SomethingItem;
use errors::*;

pub struct EndpointsDb {
    pub rx_polls: Receiver<(SomethingItem, Sender<Box<[u8]>>)>,
    pub rx_writes: Receiver<(SomethingItem, Receiver<Box<[u8]>>)>,

    pub pending_poll: Vec<(SomethingItem, Sender<Box<[u8]>>)>,
    pub pending_write: Vec<(SomethingItem, Receiver<Box<[u8]>>)>,

    pub tx_poll_characs: Sender<(GATTCharacteristic, Sender<Box<[u8]>>)>,
    pub tx_write_characs: Sender<(GATTCharacteristic, Receiver<Box<[u8]>>)>,

    pub rx_devs: Receiver<(BtMacAddress, Device)>,
    pub devices: HashMap<BtMacAddress, Device>,

    pub endpoint_interval: Duration,
}

pub fn endpoints_task(data: &mut EndpointsDb) -> Option<Duration> {
    trace!("Endpoint Tick...");

    if let Ok(_) = data.manage_endpoints() {
        // data.manage_new_devices(devs);
        Some(data.endpoint_interval)
    } else {
        // An error has occurred, bail
        error!("Error, endpoints_task bailing");
        None
    }
}

impl EndpointsDb {
    pub fn manage_endpoints(&mut self) -> Result<()> {
        // Add incoming devices
        while let Ok((mac, dev)) = self.rx_devs.try_recv() {
            self.devices.insert(mac, dev);
        }

        // Obtain locked inner data structure
        self.discover_services()
    }

    pub fn discover_services(&mut self) -> Result<()> {
        self.handle_polls()?;
        self.handle_writes()?;

        Ok(())
    }

    pub fn handle_polls(&mut self) -> Result<()> {
        while let Ok(p) = self.rx_polls.try_recv() {
            info!("Received Poll request: {:?}", p);
            self.pending_poll.push(p);
        }

        let mut rem = vec![];
        'polls: for (i, &(ref si, ref tx)) in self.pending_poll.iter().enumerate() {
            let dev = match self.devices.get(&si.mac) {
                Some(x) => x,
                _ => continue,
            };

            let mut svcs = dev.get_gatt_services()?;

            if svcs.len() == 0 {
                debug!("No services found, waiting");
                continue;
            }

            'servs: for serv in svcs.drain(..) {
                // Discover Services
                let service = GATTService::new(serv);
                let serv_uuid =
                    Uuid::from_str(&service.get_uuid()?).chain_err(|| "failed to parse svc uuid")?;

                if si.svc != serv_uuid {
                    continue 'servs;
                }

                // Discover characteristics
                'chrcs: for charac_str in service.get_gatt_characteristics()? {
                    let charac = GATTCharacteristic::new(charac_str);
                    let chr_uuid = Uuid::from_str(&charac.get_uuid()?)
                        .chain_err(|| "failed to parse chr uuid")?;

                    if si.chrc != chr_uuid {
                        continue 'chrcs;
                    }

                    // do something with charac
                    rem.push(i);
                    self.tx_poll_characs
                        .send((charac, tx.clone()))
                        .chain_err(|| "")?;
                    continue 'polls;
                }
            }
        }

        let mut rmvd: usize = 0;
        for r in rem {
            self.pending_poll.remove(r - rmvd);
            rmvd += 1;
        }

        Ok(())
    }

    pub fn handle_writes(&mut self) -> Result<()> {
        while let Ok(w) = self.rx_writes.try_recv() {
            self.pending_write.push(w);
        }

        let mut rem = vec![];
        let mut charcs_found = vec![];

        'polls: for (i, &(ref si, ref _rx)) in self.pending_write.iter().enumerate() {
            let dev = match self.devices.get(&si.mac) {
                Some(x) => x,
                _ => continue,
            };

            let mut svcs = dev.get_gatt_services()?;

            if svcs.len() == 0 {
                debug!("No services found, waiting");
                continue;
            }

            'servs: for serv in svcs.drain(..) {
                // Discover Services
                let service = GATTService::new(serv);
                let serv_uuid =
                    Uuid::from_str(&service.get_uuid()?).chain_err(|| "failed to parse svc uuid")?;

                if si.svc != serv_uuid {
                    continue 'servs;
                }

                // Discover characteristics
                'chrcs: for charac_str in service.get_gatt_characteristics()? {
                    let charac = GATTCharacteristic::new(charac_str);
                    let chr_uuid = Uuid::from_str(&charac.get_uuid()?)
                        .chain_err(|| "failed to parse chr uuid")?;

                    if si.chrc != chr_uuid {
                        continue 'chrcs;
                    }

                    // do something with charac
                    rem.push(i);
                    charcs_found.push(charac);
                    continue 'polls;
                }
            }
        }

        let mut rmvd: usize = 0;
        for r in rem {
            let (_, rx) = self.pending_write.remove(r - rmvd);
            self.tx_write_characs
                .send((charcs_found.remove(0), rx))
                .chain_err(|| "")?;
            rmvd += 1;
        }

        Ok(())
    }
}
