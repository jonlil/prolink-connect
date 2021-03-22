use crate::keepalive::KeepAliveListener;

const ANNOUNCE_PORT: u16 = 50000;
const STATUS_PORT: u16 = 50002;

pub struct ProlinkNetwork {
    keep_alive_listener: KeepAliveListener,
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
        let keep_alive_listener = KeepAliveListener::bind(("0.0.0.0", ANNOUNCE_PORT))?;
        let mut device_manager = DeviceManager {
            devices: Vec::new(),
        };

        Ok(Self {
            keep_alive_listener,
            device_manager,
        })
    }

    pub fn connect(&self) {
        loop {
            match self.keep_alive_listener.recv_from() {
                Ok((message, _)) => eprintln!("{:#?}", message),
                Err(err) => eprintln!("{:#?}", err),
            };

            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}
