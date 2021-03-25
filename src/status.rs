use std::net::ToSocketAddrs;
use std::net::UdpSocket;
use std::sync::mpsc;

pub struct StatusEventServer(UdpSocket);

impl StatusEventServer {
    fn bind<T: ToSocketAddrs>(addr: T) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(addr)?;

        Ok(Self(socket))
    }

    pub fn run<T: ToSocketAddrs>(addr: T) -> Result<mpsc::Receiver<()>, std::io::Error> {
        let (tx, rx) = mpsc::channel();
        let socket = Self::bind(addr)?;

        std::thread::spawn(move || loop {
            socket.recv_from(tx.clone());
            std::thread::sleep(std::time::Duration::from_millis(50));
        });

        Ok(rx)
    }

    fn recv_from(&self, _tx: mpsc::Sender<()>) {
        let mut buffer = [0u8; 256];
        match self.0.recv_from(&mut buffer) {
            Ok((nob, peer)) => {
                eprintln!("{:#?}", buffer);
            }
            Err(err) => eprintln!("{:#?}", err),
        };
    }
}
