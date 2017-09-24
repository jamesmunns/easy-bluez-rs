extern crate easy_bluez;
use easy_bluez::{BtCharacteristic, BtDevice, BtService, Duration, EasyBluez};
use std::str::FromStr;
use easy_bluez::errors::*;

fn main() {
    let _ = run();
}

fn run() -> Result<()> {
    let _ez = EasyBluez::new()
        .scan_interval(Duration::seconds(10))
        .scan_duration(Duration::seconds(2))
        .run();

    // ez.

    let _device = BtDevice::from_str("CF:75:CE:86:6D:02")?
        .service(
            BtService::from_str("00000001-c001-de30-cabb-785feabcd123")?
                .characteristic(
                    BtCharacteristic::from_str("0000c01d-c001-de30-cabb-785feabcd123")?,
                ),
        )
        .service(
            BtService::from_str("0f050001-3225-44b1-b97d-d3274acb29de")?
                .characteristic(
                    BtCharacteristic::from_str("0f050002-3225-44b1-b97d-d3274acb29de")?,
                ),
        );

    Ok(())
}

