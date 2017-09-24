#![allow(unused_doc_comment)]
#![allow(unused_extern_crates)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate basic_scheduler;
extern crate blurz;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate eui48;
#[macro_use]
extern crate log;
extern crate mvdb;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use basic_scheduler::{BasicEvent, Scheduler};

pub mod errors;
mod whitelist;
mod bt_manager;
mod api;

pub use api::*;
pub use basic_scheduler::Duration;
pub use whitelist::{BtCharacteristic, BtDevice, BtMacAddress, BtService};
pub use uuid::Uuid;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
