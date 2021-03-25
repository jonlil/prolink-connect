mod keepalive;
mod network;
mod status;

fn main() -> std::io::Result<()> {
    let mut network = network::ProlinkNetwork::new()?;
    network.run();

    Ok(())
}
