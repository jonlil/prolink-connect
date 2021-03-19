use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::devices::{KeepAliveMessage, process_keep_alive_message};

const ANNOUNCE_PORT: u16 = 50000;
const STATUS_PORT: u16 = 50002;

pub struct ProlinkNetwork {
    status_socket: StatusSocket,
    device_manager: DeviceManager,
}

pub struct NetworkState {
    connected: bool,
}

pub struct Device;

pub struct DeviceManager {
    pub devices: Vec<Device>,
}

impl ProlinkNetwork {
    pub fn new() -> std::io::Result<Self> {
        let status_socket = StatusSocket::bind(("0.0.0.0", ANNOUNCE_PORT))?;
        let mut device_manager = DeviceManager {
            devices: Vec::new(),
        };

        Ok(Self {
            status_socket,
            device_manager,
        })
    }

    pub fn connect(&self) {
        loop {
            match self.status_socket.recv_from() {
                Ok((message, _)) => eprintln!("{:#?}", message),
                Err(err) => eprintln!("{:#?}", err),
            };

            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}

struct StatusSocket(UdpSocket);
impl StatusSocket {
    fn bind<T: ToSocketAddrs>(address: T) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(address)?;
        socket.set_broadcast(true)?;

        Ok(StatusSocket(socket))
    }

    fn recv_from(&self) -> Result<(KeepAliveMessage, SocketAddr), &str> {
        let mut buffer = [0u8; 256];
        match self.0.recv_from(&mut buffer) {
            Ok((nob, peer)) => {
                match process_keep_alive_message(&buffer[..nob]) {
                    Ok(message) => Ok((message, peer)),
                    Err(err) => {
                        eprintln!("{:?}", err);
                        Err("Failed parsing KeepAliveMessage")
                    },
                }
            },
            Err(_) => Err("Failed reading StatusSocket")
        }
    }
}
