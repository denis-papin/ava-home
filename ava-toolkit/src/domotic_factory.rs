use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{info, error};
use rumqttc::v5::mqttbytes::QoS;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use uuid::Uuid;
use crate::generic_device::{GenericDevice, Locality};
use crate::hard_loop::HardLoop;

#[derive(Debug, Deserialize)]
pub struct DeviceDefinition {
    family: String,
    name: String,
    message_type: String,       // ex: "LampRgb" → loads "LampRgb.json"
    process_same_message: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoopDefinition {
    loop_name: String,
    devices: Vec<String>,
}

/// Root configuration describing a module's setup.
#[derive(Debug, Deserialize)]
pub struct ConfigRoot {
    devices: Vec<DeviceDefinition>,
    loops: Vec<LoopDefinition>,
    #[serde(default)]
    devices_to_init: Vec<String>,
    #[serde(default)]
    devices_to_listen: Vec<String>,
}


/// -------------------------------------------------------------------------
/// JSON UTILITIES
/// -------------------------------------------------------------------------
fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> std::io::Result<T> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON error: {}", e))
    })?)
}

/// Loads a message object of type `T` (implements Locality) from a
/// JSON file named `<message_type>.json`.
pub fn  factory<T>(message_type: &str) -> T where T : Locality + DeserializeOwned {
    let path_to_json = format!(r"/home/denis/Projects/wks-ava-home/ava-home/ava-toolkit/resources/{}.json", message_type);
    let  object_json= fs::read_to_string(path_to_json).unwrap();
    let message : T =  serde_json::from_str(object_json.as_str()).unwrap(); // TODO
    message
}


#[derive(Debug, Clone)]
pub struct Channels {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(String, QoS)>,
    pub keep_alive :  u16,
}

fn generate_client_id() -> String {
    format!("{}", Uuid::new_v4())
}


/// -------------------------------------------------------------------------
/// Core factory type for domotic configuration-driven initialization.
/// -------------------------------------------------------------------------
#[derive(Debug)]
pub struct DomoticFactory<T: Locality> {
    config_path: PathBuf,
    devices: HashMap<String, Arc<RefCell<GenericDevice<T>>>>,
}

impl<T: Locality + Clone + DeserializeOwned> DomoticFactory<T> {
    /// Create a new factory bound to a config file (devices + loops + init info)
    pub fn new(config_path: impl AsRef<Path>) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            devices: HashMap::new(),
        }
    }

    /// Static
    /// Extract channel name from devices
    pub fn extract_channel_from_devices(devices : &Vec<Arc<RefCell<GenericDevice<T>>>>) -> Channels {
        let client_id = generate_client_id(); // CLIENT_ID.to_string();

        let mut channel_filters: Vec<(String, QoS)> = vec![];
        for dev in devices {
            let dd = dev.as_ref().borrow();
            let topic = dd.get_topic();
            channel_filters.push((topic, QoS::AtMostOnce));
        }

        Channels {
            server_addr : "raspberrypi.local".to_string(),
            client_id,
            channel_filters,
            keep_alive : 30_000,
        }
    }
    
    /// Load and build all devices based on the configuration file
    pub fn build_devices(&mut self) {
        let config: ConfigRoot =
            read_json_file(&self.config_path).expect("Cannot parse configuration");

        for def in &config.devices {
            let msg: T = factory(&def.message_type);
            let dev = GenericDevice::new(&def.family, &def.name, msg, def.process_same_message);
            self.devices.insert(def.name.clone(), Arc::new(RefCell::new(dev)));
        }
        info!("✅ Built {} device(s)", self.devices.len());
    }

    /// Return a reference to the device repository
    pub fn repo(&self) -> &HashMap<String, Arc<RefCell<GenericDevice<T>>>> {
        &self.devices
    }

    /// Return all devices that need initialization
    pub fn devices_to_init(&self) -> Vec<Arc<RefCell<GenericDevice<T>>>> {
        let config: ConfigRoot =
            read_json_file(&self.config_path).expect("Cannot parse configuration");
        config
            .devices_to_init
            .into_iter()
            .filter_map(|name| self.devices.get(&name).cloned())
            .collect()
    }

    pub fn devices_to_listen(&self) -> Vec<Arc<RefCell<GenericDevice<T>>>> {
        let config: ConfigRoot =
            read_json_file(&self.config_path).expect("Cannot parse configuration");
        config
            .devices_to_listen
            .into_iter()
            .filter_map(|name| self.devices.get(&name).cloned())
            .collect()
    }

    /// Build loops defined in the configuration file
    pub fn build_loops(&self) -> Vec<HardLoop<T>> {
        let config: ConfigRoot =
            read_json_file(&self.config_path).expect("Cannot parse configuration");
        let mut loops = Vec::new();

        for def in config.loops {
            let devices: Vec<_> = def
                .devices
                .into_iter()
                .filter_map(|n| {
                    if let Some(d) = self.devices.get(&n) {
                        Some(d.clone())
                    } else {
                        error!("Unknown device '{n}' in loop '{}'", def.loop_name);
                        None
                    }
                })
                .collect();

            loops.push(HardLoop::new(def.loop_name.clone(), devices));
        }
        info!("🔁 Built {} loop(s)", loops.len());
        loops
    }
}
