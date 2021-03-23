use ipnetwork::{IpNetwork, Ipv4Network};
use pnet::datalink::interfaces;
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::mpsc::{Receiver, RecvError},
};

use crate::keepalive::{Device, KeepAliveListener};

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

    fn recv(&self) -> Result<(Device, SocketAddr), RecvError> {
        self.rx.recv()
    }
}

pub struct NetworkState {
    connected: bool,
    network: Option<Ipv4Network>,
}

pub struct DeviceManager {
    pub devices: HashMap<u8, Device>,
}

impl DeviceManager {
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
    network_state: NetworkState,
}

impl ProlinkNetwork {
    pub fn new() -> std::io::Result<Self> {
        let keepalive_manager = KeepAliveManager::new()?;
        let mut device_manager = DeviceManager {
            devices: HashMap::new(),
        };
        let mut network_state = NetworkState {
            connected: false,
            network: None,
        };

        Ok(Self {
            device_manager,
            keepalive_manager,
            network_state,
        })
    }

    pub fn run(&mut self) {
        loop {
            match self.keepalive_manager.recv() {
                Ok((device, _peer)) => {
                    if self.network_state.connected == false {
                        if let Some(network) = find_ipv4_network_interface(&device.ip_address) {
                            // connect to a pioneer network
                            self.network_state.connected = true;
                            self.network_state.network = Some(network);

                            eprintln!("Connected to network: {:#?}", network);
                        }
                    }

                    if self.device_manager.contains(&device) == false {
                        self.device_manager.insert(device);
                    }
                }
                Err(err) => eprintln!("{:?}", err),
            };
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}

fn find_ipv4_network_interface(address: &Ipv4Addr) -> Option<Ipv4Network> {
    interfaces()
        .iter()
        .flat_map(|interface| {
            interface.ips.iter().filter_map(|ip| match ip {
                IpNetwork::V4(ip) => Some(*ip),
                _ => None,
            })
        })
        .find(|network: &Ipv4Network| network.contains(*address))
}
