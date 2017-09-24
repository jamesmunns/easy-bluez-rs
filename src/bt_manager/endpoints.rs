use std::str::FromStr;
use std::ops::DerefMut;
use std::collections::HashMap;

use blurz::{GATTCharacteristic, GATTService};
use uuid::Uuid;

use basic_scheduler;
use whitelist::{BtServices, Whitelist};
use bt_manager::{AtomicBtDb, ManagedDevice};
use errors::*;

pub struct EndpointsDb {
    pub db: AtomicBtDb,
    pub wl: Whitelist,
}

pub fn endpoints_task(data: &mut EndpointsDb) -> Option<basic_scheduler::Duration> {
    trace!("Endpoint Tick...");

    if let Ok(_) = data.manage_endpoints() {
        // data.manage_new_devices(devs);
        Some(basic_scheduler::Duration::seconds(5))
    } else {
        // An error has occurred, bail
        error!("Error, endpoints_task bailing");
        None
    }
}

impl EndpointsDb {
    pub fn manage_endpoints(&mut self) -> Result<()> {
        // Obtain locked inner data structure
        let mut db_l = self.db.lock().unwrap();
        let db = db_l.deref_mut();

        for (ref mac_addr, ref mut man_dev) in db.devices.iter_mut() {
            Self::discover_services(man_dev, self.wl.get_device_btmac(mac_addr).unwrap())?;
        }

        Ok(())
    }

    pub fn discover_services(dev: &mut ManagedDevice, serv_wl: &BtServices) -> Result<()> {
        let mut svcs = dev.bluez_handle.get_gatt_services()?;

        if svcs.len() == 0 {
            debug!("No services found, waiting");
            return Ok(());
        }

        for serv in svcs.drain(..) {
            // Discover Services
            let service = GATTService::new(serv);
            let serv_uuid =
                Uuid::from_str(&service.get_uuid()?).chain_err(|| "failed to parse svc uuid")?;

            if !serv_wl.contains_service(&serv_uuid) {
                continue;
            }

            // Ensure service exists in cache
            if !dev.cached_data.contains_key(&serv_uuid) {
                info!(
                    "Adding Service {} to {:?}",
                    serv_uuid.hyphenated(),
                    dev.bluez_handle
                );
                dev.cached_data.insert(serv_uuid.clone(), HashMap::new());
                dev.service_map.insert(serv_uuid.clone(), service.clone());
            }

            let wl_chars = serv_wl.get_service(&serv_uuid).unwrap();

            // Discover characteristics
            for charac_str in service.get_gatt_characteristics()? {
                let charac = GATTCharacteristic::new(charac_str);
                let chr_uuid =
                    Uuid::from_str(&charac.get_uuid()?).chain_err(|| "failed to parse chr uuid")?;

                if !wl_chars.contains_characteristic(&chr_uuid) {
                    continue;
                }

                let svc_map_handle = dev.cached_data.get_mut(&serv_uuid).unwrap();

                if !svc_map_handle.contains_key(&chr_uuid) {
                    info!(
                        "Adding Characteristic {} to {} for {:?}",
                        chr_uuid.hyphenated(),
                        serv_uuid.hyphenated(),
                        dev.bluez_handle
                    );

                    svc_map_handle.insert(chr_uuid.clone(), None);

                    dev.charac_map
                        .insert((serv_uuid.clone(), chr_uuid.clone()), charac);
                }
            }
        }

        Ok(())
    }
}
