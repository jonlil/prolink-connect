use crate::network::NetworkInterface;
use error::Error;
use protocol::UdpMagic;
use std::{
    net::{Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
    sync::mpsc::Receiver,
};

mod error;
mod protocol;

pub use protocol::KeepAliveMessage;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub id: u8,
    pub mac_addr: protocol::MacAddr,
    pub ip_address: Ipv4Addr,
    pub device_type: protocol::DeviceType,
}

pub fn virtual_cdj(network: NetworkInterface) -> Device {
    Device {
        id: 5,
        name: "VirtualCDJ".to_string(),
        device_type: protocol::DeviceType::Cdj,
        ip_address: network.ip_network.ip(),
        mac_addr: network.mac,
    }
}

trait KeepAliveEventEmitter {
    fn on_event(message: KeepAliveMessage);
}

pub struct KeepAliveListener(UdpSocket);
impl KeepAliveListener {
    fn bind<T: ToSocketAddrs>(address: T) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(address)?;
        socket.set_broadcast(true)?;

        Ok(Self(socket))
    }

    pub fn run<T: ToSocketAddrs>(
        address: T,
    ) -> Result<(Receiver<(Device, SocketAddr)>), std::io::Error> {
        let (tx, rx) = std::sync::mpsc::channel();
        let socket = Self::bind(address)?;
        std::thread::spawn(move || loop {
            socket.recv_from(tx.clone());
            std::thread::sleep(std::time::Duration::from_millis(250));
        });

        Ok(rx)
    }

    pub fn recv_from(&self, tx: std::sync::mpsc::Sender<(Device, SocketAddr)>) {
        let mut buffer = [0u8; 256];
        match self.0.recv_from(&mut buffer) {
            Ok((nob, peer)) => match process_keep_alive_message(&buffer[..nob]) {
                Ok(message) => {
                    match message.msg_value {
                        protocol::MessageType::Status(status) => {
                            match tx.send((
                                Device {
                                    id: status.player_number,
                                    name: message.model_name,
                                    mac_addr: status.mac_address,
                                    ip_address: status.ip_addr,
                                    device_type: message.device_type,
                                },
                                peer,
                            )) {
                                Ok(_) => {}
                                Err(_) => {}
                            };
                        }
                        _ => {}
                    };
                }
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            },
            Err(_) => eprintln!("Failed reading keep alive network."),
        };
    }
}

pub fn process_keep_alive_message(input: &[u8]) -> Result<KeepAliveMessage, Error> {
    match UdpMagic::decode(&input) {
        Ok((input, _)) => match KeepAliveMessage::parse(&input) {
            Ok((_, message)) => Ok(message),
            Err(_) => Err(Error::ParseError),
        },
        Err(_) => Err(Error::MissingHeaderError),
    }
}
