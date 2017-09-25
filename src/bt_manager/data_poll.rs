use std::sync::mpsc::{Receiver, Sender};

use Duration;
use blurz::GATTCharacteristic;

use errors::*;

pub struct DataDb {
    pub poll_interval: Duration,
    pub poll_rx: Receiver<(GATTCharacteristic, Sender<Box<[u8]>>)>,
    pub polls: Vec<(GATTCharacteristic, Sender<Box<[u8]>>)>,
}

pub fn data_poll_task(data: &mut DataDb) -> Option<Duration> {
    trace!("DataPoll Tick...");

    if let Ok(_) = data.poll_data() {
        Some(data.poll_interval)
    } else {
        // An error has occurred, bail
        error!("Error, data_poll_task bailing");
        None
    }
}

impl DataDb {
    pub fn poll_data(&mut self) -> Result<()> {
        while let Ok(new_poll) = self.poll_rx.try_recv() {
            self.polls.push(new_poll);
        }

        for &(ref blurz_chr, ref txer) in self.polls.iter() {
            match blurz_chr.read_value(None) {
                Ok(new_data) => {
                    txer.send(new_data.into_boxed_slice()).unwrap();
                }
                Err(e) => {
                    error!("Failed to read, {:?}", e);
                }
            }
        }

        Ok(())
    }
}
