use basic_scheduler::{BasicEvent, Duration, Scheduler};
use bt_manager::discovery::{discovery_task, DiscoveryData};
use std::sync::mpsc::{channel, Receiver, Sender};
use whitelist::BtMacAddress;
use std::collections::HashSet;

pub struct EasyBluez {
    scan_interval: Duration,
    scan_duration: Duration,
    connect_interval: Duration,
    endpoint_interval: Duration,
    poll_interval: Duration,
}

pub struct EasyBluezHandle {
    mac_sender: Sender<BtMacAddress>,
    scheduler: Scheduler,
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
        let (tx_devs, _rx_devs) = channel();

        let discover_event = BasicEvent {
            task: |s: &mut DiscoveryData| discovery_task(s),
            state: DiscoveryData {
                db: HashSet::new(),
                wl: HashSet::new(),
                receiver: rx_macs,
                sender: tx_devs,
                scan_interval: self.scan_interval.clone(),
                scan_duration: self.scan_duration.clone(),
            },
        };

        let scheduler = Scheduler::new();
        let hdl = scheduler.add_handle();
        hdl.send(Box::new(discover_event)).unwrap();

        EasyBluezHandle {
            scheduler: scheduler,
            mac_sender: tx_macs,
        }
    }
}
