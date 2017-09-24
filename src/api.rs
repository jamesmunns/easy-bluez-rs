// #![allow(unused_variables)]

use basic_scheduler::{BasicEvent, Duration, Scheduler};
use std::sync::mpsc::{channel, Receiver, Sender};
use whitelist::BtMacAddress;
use std::collections::{HashSet, HashMap};
use errors::*;
use std::str::FromStr;
use uuid::Uuid;
use std::thread;

use bt_manager::SomethingItem;
use bt_manager::discovery::{discovery_task, DiscoveryData};
use bt_manager::connection::{connect_task, ConnectionDb};
use bt_manager::endpoints::{endpoints_task, EndpointsDb};
use bt_manager::data_poll::{data_poll_task, DataDb};

pub struct EasyBluez {
    scan_interval: Duration,
    scan_duration: Duration,
    connect_interval: Duration,
    endpoint_interval: Duration,
    poll_interval: Duration,
}

pub struct EasyBluezHandle {
    mac_sender: Sender<BtMacAddress>,
    poll_sender: Sender<(SomethingItem, Sender<Box<[u8]>>)>,
    write_sender: Sender<(SomethingItem, Receiver<Box<[u8]>>)>,
    scheduler: thread::JoinHandle<()>,

    _rx: Receiver<(String, bool)>,
}

impl EasyBluezHandle {
    pub fn writeable(
        &mut self,
        mac_s: &str,
        svc_s: &str,
        chrc_s: &str,
        rxer: Receiver<Box<[u8]>>,
    ) -> Result<()> {
        let mac = BtMacAddress::from_str(mac_s)?;
        let svc = Uuid::from_str(svc_s).chain_err(|| "not a UUID!")?;
        let chrc = Uuid::from_str(chrc_s).chain_err(|| "not a UUID!")?;

        self.mac_sender.send(mac.clone()).chain_err(|| "")?;

        let si = SomethingItem {
            mac: mac,
            svc: svc,
            chrc: chrc,
        };

        self.write_sender.send((si, rxer))
            .chain_err(|| "")?;

        Ok(())
    }

    pub fn poll(
        &mut self,
        mac_s: &str,
        svc_s: &str,
        chrc_s: &str,
        needs_pair: bool,
        dest: Sender<Box<[u8]>>,
    ) -> Result<()> {
        let mac = BtMacAddress::from_str(mac_s)?;
        let svc = Uuid::from_str(svc_s).chain_err(|| "not a UUID!")?;
        let chrc = Uuid::from_str(chrc_s).chain_err(|| "not a UUID!")?;

        self.mac_sender.send(mac.clone()).chain_err(|| "")?;

        let si = SomethingItem {
            mac: mac,
            svc: svc,
            chrc: chrc,
        };

        self.poll_sender.send((si, dest))
            .chain_err(|| "")?;

        Ok(())
    }
}

impl EasyBluez {
    pub fn new() -> Self {
        EasyBluez {
            scan_interval: Duration::seconds(10),
            scan_duration: Duration::seconds(5),
            connect_interval: Duration::seconds(3),
            endpoint_interval: Duration::seconds(3),
            poll_interval: Duration::seconds(2),
        }
    }

    ///////////////////////////////////////////////////////
    // Builder options
    ///////////////////////////////////////////////////////

    /// How often to scan for new BLE devices
    pub fn scan_interval(mut self, interval: Duration) -> Self {
        self.scan_interval = interval;
        self
    }

    /// How long to scan for new BLE devices
    pub fn scan_duration(mut self, duration: Duration) -> Self {
        self.scan_duration = duration;
        self
    }

    /// How often to attempt to connect to discovered devices
    pub fn connect_interval(mut self, interval: Duration) -> Self {
        self.connect_interval = interval;
        self
    }

    /// How often to find services/characteristics for connected devices
    pub fn endpoint_interval(mut self, interval: Duration) -> Self {
        self.endpoint_interval = interval;
        self
    }

    /// How often to poll readable endpoints
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    ///////////////////////////////////////////////////////
    // Run time
    ///////////////////////////////////////////////////////
    pub fn run(&mut self) -> EasyBluezHandle {
        self.spawn_events()
    }

    fn spawn_events(&mut self) -> EasyBluezHandle {
        let (tx_macs, rx_macs) = channel();
        let (tx_devs, rx_devs) = channel();
        let (tx_conn_evs, _rx_conn_evs) = channel();
        let (tx_poll, rx_poll) = channel();
        let (tx_write, rx_write) = channel();
        let (tx_poll_characs, rx_poll_characs) = channel();
        let (tx_write_characs, rx_write_characs) = channel();
        let (tx_edpts, rx_edpts) = channel();

        let discover_event = BasicEvent {
            task: |s: &mut DiscoveryData| discovery_task(s),
            state: DiscoveryData {
                db: HashSet::new(),
                wl: HashSet::new(),
                receiver: rx_macs,
                sender_connect: tx_devs,
                sender_endpoints: tx_edpts,
                scan_interval: self.scan_interval.clone(),
                scan_duration: self.scan_duration.clone(),
            },
        };

        let connection_event = BasicEvent {
            task: |s: &mut ConnectionDb| connect_task(s),
            state: ConnectionDb {
                connect_interval: self.connect_interval,
                db: vec![],
                incoming: rx_devs,
                outgoing: tx_conn_evs,
            },
        };

        let endpoints_event = BasicEvent {
            task: |s: &mut EndpointsDb| endpoints_task(s),
            state: EndpointsDb {
                endpoint_interval: self.endpoint_interval,

                rx_polls: rx_poll,
                rx_writes: rx_write,

                pending_poll: Vec::new(),
                pending_write: Vec::new(),

                tx_poll_characs: tx_poll_characs,
                tx_write_characs: tx_write_characs,

                rx_devs: rx_edpts,
                devices: HashMap::new(),

            },
        };

        let poll_event = BasicEvent {
            task: |s: &mut DataDb| data_poll_task(s),
            state: DataDb {
                poll_interval: self.poll_interval,
                polls: Vec::new(),
                poll_rx: rx_poll_characs,
            },
        };

        let mut scheduler = Scheduler::new();
        let hdl = scheduler.add_handle();
        hdl.send(Box::new(discover_event)).unwrap();
        hdl.send(Box::new(connection_event)).unwrap();
        hdl.send(Box::new(endpoints_event)).unwrap();
        hdl.send(Box::new(poll_event)).unwrap();

        EasyBluezHandle {
            scheduler: thread::spawn(move || {
                scheduler.run();
            }),
            mac_sender: tx_macs,
            poll_sender: tx_poll,
            write_sender: tx_write,
            _rx: _rx_conn_evs,
        }
    }
}
