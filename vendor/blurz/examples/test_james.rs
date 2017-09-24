extern crate blurz;

use std::time::Duration;
use std::thread;
use blurz::{Adapter, Device, DiscoverySession, GATTService, GATTCharacteristic};
use blurz::errors::*;

fn test3() -> Result<()> {
    let adapter: Adapter = Adapter::init()?;
    adapter.set_powered(true)?;

    // reverse cleanup
    println!("Cleaning up...");
    let mut devices = adapter.get_device_list()?;
    println!("{} device(s) found", devices.len());
    for d in devices.drain(..) {
        let device = Device::new(d);
        adapter.remove_device(device.get_id())?;
    }

    let mut my_dev = None;

    loop {
        let session = DiscoverySession::create_session(adapter.get_id())?;
        thread::sleep(Duration::from_millis(200));

        session.start_discovery()?;
        thread::sleep(Duration::from_millis(800));

        let mut devices = adapter.get_device_list()?;
        println!("{} device(s) found", devices.len());

        for d in devices.drain(..) {
            let device = Device::new(d);
            println!(
                "{} {:?} {:?}",
                device.get_id(),
                device.get_address(),
                device.get_gatt_services()
            );
            match device.get_address() {
                Ok(ref id) if id == "CF:75:CE:86:6D:02" => {
                    my_dev = Some(device);
                    println!("FOUND!");
                    break;
                }
                _ => {
                    adapter.remove_device(device.get_id())?;
                }
            }
        }

        session.stop_discovery()?;

        if my_dev.is_some() {
            break;
        }
    }

    let device = my_dev.unwrap();

    println!("Connecting...");
    while !device.is_connected()? {
        match device.connect() {
            Ok(_) => break,
            Err(e) => {
                println!("Err, retrying, {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(1500));
    }

    // remove CF:75:CE:86:6D:02

    let mut servs = None;

    while let Ok(svcs) = device.get_gatt_services() {
        if !device.is_connected()? {
            println!("Other side hung up.");
            return Ok(());
        }
        if svcs.len() == 0 {
            println!("Waiting...");
            thread::sleep(Duration::from_millis(1500));
        } else {
            servs = Some(svcs);
            println!("Services: {:?}", servs);
            println!("uuids: {:?}", device.get_uuids()?);
            break;
        }
    }

    println!("");

    println!("Pairing...");
    while !device.is_paired()? {
        match device.pair() {
            Ok(_) => break,
            Err(e) => {
                println!("Err, retrying, {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(1500));
    }

    println!("");
    thread::sleep(Duration::from_millis(1500));

    for s in servs.unwrap() {
        let service = GATTService::new(s.clone());
        println!("{:?}", service.get_uuid()?);
        let chars = service.get_gatt_characteristics()?;
        for c in chars {
            let charac = GATTCharacteristic::new(c);
            println!("\t{:?}", charac.get_uuid()?);

            match charac.read_value(None) {
                Ok(data) => {
                    println!("\t\t{:?}", data);
                }
                Err(e) => {
                    println!("\t\t{:?}", e);
                }
            }
            thread::sleep(Duration::from_millis(1500));
        }
    }


    Ok(())
}

fn main() {
    match test3() {
        Ok(_) => (),
        Err(e) => println!("Fatal: {:?}", e),
    }
}
