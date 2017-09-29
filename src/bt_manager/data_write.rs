use std::sync::mpsc::Receiver;

use Duration;
use blurz::BluetoothGATTCharacteristic;

use errors::*;

pub struct DataWDb {
    pub write_interval: Duration,
    pub write_rx: Receiver<(BluetoothGATTCharacteristic, Receiver<Box<[u8]>>)>,
    pub writes: Vec<(BluetoothGATTCharacteristic, Receiver<Box<[u8]>>)>,
}

pub fn data_write_task(data: &mut DataWDb) -> Option<Duration> {
    trace!("DataWrite Tick...");

    if let Ok(_) = data.write_data() {
        Some(data.write_interval)
    } else {
        // An error has occurred, bail
        error!("Error, data_write_task bailing");
        None
    }
}

impl DataWDb {
    pub fn write_data(&mut self) -> Result<()> {
        while let Ok(new_write) = self.write_rx.try_recv() {
            self.writes.push(new_write);
        }

        for &(ref blurz_chr, ref rxer) in self.writes.iter() {
            if let Ok(msg) = rxer.try_recv() {
                blurz_chr.write_value(msg.to_vec())
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }
}
