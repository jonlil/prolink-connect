mod network;
mod keepalive;

fn main() -> std::io::Result<()> {

    let network = network::ProlinkNetwork::new()?;
    network.connect();

    Ok(())
}
