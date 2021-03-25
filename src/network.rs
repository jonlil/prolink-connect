use ipnetwork::{IpNetwork, Ipv4Network};
use pnet::datalink::interfaces;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::mpsc::{Receiver, TryRecvError},
};

use crate::keepalive::{virtual_cdj, Device, KeepAliveListener};
use crate::status::StatusEventServer;

const ANNOUNCE_PORT: u16 = 50000;
const STATUS_PORT: u16 = 50002;

pub struct KeepAliveManager {
    rx: Receiver<(Device, SocketAddr)>,
}

impl KeepAliveManager {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            rx: KeepAliveListener::run(("0.0.0.0", ANNOUNCE_PORT))?,
        })
    }

    fn try_recv(&self) -> Result<(Device, SocketAddr), TryRecvError> {
        self.rx.try_recv()
    }
}

struct StatusManager {
    rx: Receiver<()>,
}

impl StatusManager {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            rx: StatusEventServer::run(("0.0.0.0", STATUS_PORT))?,
        })
    }

    fn try_recv(&self) -> Result<(), TryRecvError> {
        self.rx.try_recv()
    }
}

pub struct NetworkState {
    connected: bool,
    network: Option<Ipv4Network>,
}

impl NetworkState {
    fn new() -> Self {
        Self {
            connected: false,
            network: None,
        }
    }
}

pub struct DeviceManager {
    pub devices: HashMap<u8, Device>,
}

impl DeviceManager {
    fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    fn insert(&mut self, device: Device) {
        self.devices.insert(device.id, device);
    }

    fn contains(&self, device: &Device) -> bool {
        eprintln!("Device {:} already exists", device.id);
        self.devices.contains_key(&device.id)
    }
}

pub struct ProlinkNetwork {
    device_manager: DeviceManager,
    keepalive_manager: KeepAliveManager,
    status_manager: StatusManager,
    network_state: NetworkState,
    vcdj: Option<Device>,
}

impl ProlinkNetwork {
    pub fn new() -> std::io::Result<Self> {
        let keepalive_manager = KeepAliveManager::new()?;
        let status_manager = StatusManager::new()?;
        let device_manager = DeviceManager::new();
        let network_state = NetworkState::new();

        Ok(Self {
            device_manager,
            keepalive_manager,
            network_state,
            status_manager,
            vcdj: None,
        })
    }

    pub fn run(&mut self) {
        loop {
            match self.keepalive_manager.try_recv() {
                Ok((device, _peer)) => self.on_device(device),
                Err(_err) => {}
            };

            match self.status_manager.try_recv() {
                Ok(_) => {}
                Err(_err) => {}
            };

            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    fn on_device(&mut self, device: Device) {
        if self.network_state.connected == false {
            if self.network_state.connected == false {
                if let Some(network) = find_ipv4_network_interface(&device.ip_address) {
                    // connect to a pioneer network
                    self.network_state.connected = true;
                    self.network_state.network = Some(network.ip_network);

                    // TODO: Implement announcement of virtual device
                    self.vcdj = Some(virtual_cdj(&network));
                    eprintln!("Connected to network: {:#?}", network.ip_network);
                }
            }
        }

        if self.device_manager.contains(&device) == false {
            self.device_manager.insert(device);
        }
    }
}

pub struct NetworkInterface {
    pub ip_network: Ipv4Network,
    pub mac: pnet::datalink::MacAddr,
}

fn find_ipv4_network_interface(address: &Ipv4Addr) -> Option<NetworkInterface> {
    interfaces()
        .iter()
        .filter(|interface| interface.mac.is_some())
        .flat_map(|interface| {
            interface.ips.iter().filter_map(move |ip| match ip {
                IpNetwork::V4(ip) => Some(NetworkInterface {
                    ip_network: *ip,
                    mac: interface.mac.unwrap(),
                }),
                _ => None,
            })
        })
        .find(|network: &NetworkInterface| network.ip_network.contains(*address))
}
