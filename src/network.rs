use std::{net::SocketAddr, sync::mpsc::{Receiver, RecvError}};

use crate::keepalive::{KeepAliveListener, Device};

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
}

pub struct DeviceManager {
    pub devices: Vec<Device>,
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
            devices: Vec::new(),
        };
        let mut network_state = NetworkState {
            connected: false,
        };

        Ok(Self {
            device_manager,
            keepalive_manager,
            network_state,
        })
    }

    pub fn run(&self) {
        loop {
            match self.keepalive_manager.recv() {
                Ok(msg) => eprintln!("{:#?}", msg),
                Err(err) => eprintln!("{:?}", err),
            };
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}
