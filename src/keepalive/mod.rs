use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use protocol::{KeepAliveMessage, UdpMagic};
use error::Error;

mod error;
mod protocol;

pub struct KeepAliveListener(UdpSocket);
impl KeepAliveListener {
    pub fn bind<T: ToSocketAddrs>(address: T) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(address)?;
        socket.set_broadcast(true)?;

        Ok(Self(socket))
    }

    pub fn recv_from(&self) -> Result<(KeepAliveMessage, SocketAddr), &str> {
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
            Err(_) => Err("Failed reading keep alive network.")
        }
    }
}

pub fn process_keep_alive_message(input: &[u8]) -> Result<KeepAliveMessage, Error> {
    match UdpMagic::decode(&input) {
        Ok((input, _)) => {
            match KeepAliveMessage::parse(&input) {
                Ok((_, message)) => Ok(message),
                Err(_) => Err(Error::ParseError),
            }
        },
        Err(_) => Err(Error::MissingHeaderError),
    }
}
