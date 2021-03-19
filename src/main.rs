use std::net::UdpSocket;

mod devices;
mod network;

fn main() -> std::io::Result<()> {

    let network = network::ProlinkNetwork::new()?;
    network.connect();

    Ok(())
}
