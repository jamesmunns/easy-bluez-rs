use std::collections::hash_map::Keys;
use std::collections::hash_set::Iter;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::str::FromStr;

use errors::*;
use eui48::MacAddress;
use mvdb::helpers::just_load;
use uuid::Uuid;

impl Hash for BtMacAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state)
    }
}

impl Hash for BtCharacteristic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state)
    }
}

impl From<MacAddress> for BtMacAddress {
    fn from(other: MacAddress) -> Self {
        BtMacAddress(other)
    }
}

impl From<Uuid> for BtCharacteristic {
    fn from(other: Uuid) -> Self {
        BtCharacteristic(other)
    }
}

impl FromStr for BtMacAddress {
    type Err = Error;
    /// Create a MacAddress from String
    fn from_str(us: &str) -> Result<BtMacAddress> {
        Ok(Self {
            0: MacAddress::parse_str(us).chain_err(|| "Not a MAC address!")?,
        })
    }
}

pub struct BtService {
    id: Uuid,
    chars: BtCharacteristics,
}

impl BtService {
    pub fn new(svc_id: Uuid) -> Self {
        BtService {
            id: svc_id,
            chars: BtCharacteristics(HashSet::new()),
        }
    }

    pub fn characteristic(mut self, charac: BtCharacteristic) -> Self {
        self.chars.0.insert(charac);
        self
    }
}

impl FromStr for BtService {
    type Err = Error;

    fn from_str(us: &str) -> Result<BtService> {
        Ok(Self::new(Uuid::from_str(us).chain_err(|| "Not a Uuid!")?))
    }
}

pub struct BtDevice {
    mac: BtMacAddress,
    svc_map: BtServices,
}

impl FromStr for BtDevice {
    type Err = Error;
    /// Create a MacAddress from String
    fn from_str(us: &str) -> Result<BtDevice> {
        Ok(Self::new(BtMacAddress(
            MacAddress::parse_str(us).chain_err(|| "Not a MAC address!")?,
        )))
    }
}

impl BtDevice {
    pub fn new(mac: BtMacAddress) -> Self {
        BtDevice {
            mac: mac,
            svc_map: BtServices(HashMap::new()),
        }
    }

    pub fn service(mut self, service: BtService) -> Self {
        self.svc_map.0.insert(service.id, service.chars);
        self
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct BtMacAddress(MacAddress);

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BtServices(HashMap<Uuid, BtCharacteristics>);

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BtCharacteristics(HashSet<BtCharacteristic>);

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct BtCharacteristic(Uuid);

impl FromStr for BtCharacteristic {
    type Err = Error;

    fn from_str(us: &str) -> Result<BtCharacteristic> {
        Ok(BtCharacteristic(
            Uuid::from_str(us).chain_err(|| "Not a Uuid!")?,
        ))
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Whitelist(HashMap<BtMacAddress, BtServices>);

impl Whitelist {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        Ok(Whitelist {
            0: just_load(path)?,
        })
    }

    pub fn contains_device(&self, id: &str) -> bool {
        if let Ok(bt_mac) = BtMacAddress::from_str(id) {
            self.0.contains_key(&bt_mac)
        } else {
            false
        }
    }

    pub fn get_device(&self, id: &str) -> Option<&BtServices> {
        if let Ok(bt_mac) = BtMacAddress::from_str(id) {
            self.0.get(&bt_mac)
        } else {
            None
        }
    }

    pub fn get_device_btmac(&self, id: &BtMacAddress) -> Option<&BtServices> {
        self.0.get(id)
    }

    pub fn devices(&self) -> Keys<BtMacAddress, BtServices> {
        self.0.keys()
    }
}

impl BtServices {
    pub fn contains_service(&self, id: &Uuid) -> bool {
        self.0.contains_key(id)
    }

    pub fn services(&self) -> Keys<Uuid, BtCharacteristics> {
        self.0.keys()
    }

    pub fn get_service(&self, id: &Uuid) -> Option<&BtCharacteristics> {
        self.0.get(id)
    }
}

impl BtCharacteristics {
    pub fn contains_characteristic(&self, id: &Uuid) -> bool {
        self.0.contains(&BtCharacteristic::from(id.clone()))
    }

    pub fn characteristics(&self) -> Iter<BtCharacteristic> {
        self.0.iter()
    }

    pub fn get_characteristic(&self, id: &Uuid) -> Option<&BtCharacteristic> {
        self.0.get(&BtCharacteristic::from(id.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    extern crate serde_json;

    #[test]
    fn e2e_test() {
        let wl: Whitelist = serde_json::from_str(
            r#"
            {
                "CF:75:CE:86:6D:02": {
                    "00000001-c001-de30-cabb-785feabcd123": [
                        "0000c01d-c001-de30-cabb-785feabcd123"
                    ],
                    "0f050001-3225-44b1-b97d-d3274acb29de": [
                        "0f050002-3225-44b1-b97d-d3274acb29de"
                    ]
                }
            }
        "#,
        ).unwrap();

        // Good matches
        let mac = "CF:75:CE:86:6D:02";
        let svc_uuid = Uuid::from_str("0f050001-3225-44b1-b97d-d3274acb29de").unwrap();
        let chr_uuid = Uuid::from_str("0f050002-3225-44b1-b97d-d3274acb29de").unwrap();

        // Bad matches
        let mac_bad = "CF:75:CE:86:6D:00";
        let svc_uuid_bad = Uuid::from_str("0f050001-3225-44b1-b97d-d3274acb29d0").unwrap();
        let chr_uuid_bad = Uuid::from_str("0f050002-3225-44b1-b97d-d3274acb29d0").unwrap();

        // Check MAC/device
        assert!(wl.contains_device(mac));
        assert!(!wl.contains_device(mac_bad));

        // Check Service in MAC/device
        let dev = wl.get_device(mac).unwrap();
        assert!(dev.contains_service(&svc_uuid));
        assert!(!dev.contains_service(&svc_uuid_bad));

        // Check Char in Service
        let svc = dev.get_service(&svc_uuid).unwrap();
        assert!(svc.contains_characteristic(&chr_uuid));
        assert!(!svc.contains_characteristic(&chr_uuid_bad));

        // Check Char is actually a match
        let chr = svc.get_characteristic(&chr_uuid).unwrap();
        assert_eq!(chr.0, chr_uuid);
        assert_ne!(chr.0, chr_uuid_bad)
    }

}
