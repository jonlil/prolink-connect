mod network;
mod keepalive;


fn main() -> std::io::Result<()> {
    let mut network = network::ProlinkNetwork::new()?;
    network.run();

    Ok(())
}
