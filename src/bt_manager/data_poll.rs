use std::time::{Duration, Instant};
use std::ops::DerefMut;

use basic_scheduler;
use bt_manager::{AtomicBtDb, PolledData};
use errors::*;

pub struct DataDb {
    pub db: AtomicBtDb,
}

pub fn data_poll_task(data: &mut DataDb) -> Option<basic_scheduler::Duration> {
    trace!("DataPoll Tick...");

    if let Ok(_) = data.poll_data() {
        // data.manage_new_devices(devs);
        Some(basic_scheduler::Duration::seconds(1))
    } else {
        // An error has occurred, bail
        error!("Error, data_poll_task bailing");
        None
    }
}

impl DataDb {
    pub fn poll_data(&mut self) -> Result<()> {
        // // Obtain locked inner data structure
        let mut db_l = self.db.lock().unwrap();
        let db = db_l.deref_mut();

        for (_bt_mac, dev) in db.devices.iter_mut() {
            for (svc, characs) in dev.cached_data.iter_mut() {
                for (charac, data) in characs {
                    // get the characteristic

                    let bluez_charac = dev.charac_map.get(&(svc.clone(), charac.clone())).unwrap();
                    match bluez_charac.read_value(None) {
                        Ok(new_data) => {
                            debug!("Updated {}:{}", svc.hyphenated(), charac.hyphenated());
                            *data = Some(PolledData {
                                time: Instant::now(),
                                data: new_data.into_boxed_slice(),
                            });
                        }
                        Err(e) => {
                            error!(
                                "Failed to read {}:{} => {}",
                                svc.hyphenated(),
                                charac.hyphenated(),
                                e
                            );

                            // Failed to poll. Is the data stale?
                            let timed_out = if let &mut Some(ref inner_data) = data {
                                inner_data.time.elapsed() > Duration::from_secs(10)
                            } else {
                                // No data can't time out
                                false
                            };

                            if timed_out {
                                *data = None;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
