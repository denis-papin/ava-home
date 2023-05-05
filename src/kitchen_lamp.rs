
use std::cell::{RefCell};
use std::net::TcpStream;

use std::sync::Arc;

use crate::{DeviceLock, DynDevice, publish};
use crate::messages::{DeviceMessage, LampRGB};


pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";

#[derive(Debug)]
pub (crate) struct KitchenLampDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
    pub setup : bool,
}

impl KitchenLampDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW KitchenLampDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl )),
            setup: false,
        }
    }

    pub fn get_name() -> &'static str {
        KITCHEN_LAMP
    }
}


impl DynDevice for KitchenLampDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        self.setup = setup;
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        LampRGB::from_json(msg)
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

        match r_info {
            Ok(lamp) => { Box::new(lamp) }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(LampRGB::new())
            }
        }
    }


    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("KitchenLamp tries to build its LambRGB message");
        origin_message.to_lamp_rgb(&last_message)
    }
}

