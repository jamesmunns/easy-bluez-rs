extern crate basic_scheduler;
extern crate blurz;
#[macro_use]
extern crate error_chain;
extern crate eui48;
#[macro_use]
extern crate log;
extern crate mvdb;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

pub mod errors;
mod bt_manager;
mod api;

pub use api::*;
pub use basic_scheduler::Duration;
pub use uuid::Uuid;

use std::hash::{Hash, Hasher};
use std::str::FromStr;

use errors::*;
use eui48::MacAddress;

impl Hash for BtMacAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state)
    }
}

impl From<MacAddress> for BtMacAddress {
    fn from(other: MacAddress) -> Self {
        BtMacAddress(other)
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


#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct BtMacAddress(MacAddress);
