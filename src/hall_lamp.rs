
use std::cell::{RefCell};
use std::net::TcpStream;

use std::sync::Arc;

use crate::{DeviceLock, DynDevice, publish};
use crate::messages::{DeviceMessage, LampRGB};

pub(crate) const HALL_LAMP : &str = "hall_lamp";

#[derive(Debug)]
pub(crate) struct HallLampDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
    pub setup : bool,
}

// TODO generalise the struct to handle all the "Lamp" family, pass the name in the constructor.
impl HallLampDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW HallLampDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl )),
            setup: false,
        }
    }
    pub fn get_name() -> &'static str {
        HALL_LAMP
    }
}

impl DynDevice for HallLampDevice {

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

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( LampRGB::from_json(msg)? ))
    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("HallLamp tries to build its LambRGB message");
        origin_message.to_lamp_rgb(last_message)
    }

}
