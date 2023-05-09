
use std::net::TcpStream;

use crate::{DynDevice, publish, InterSwitch};
use crate::messages::{DeviceMessage};

pub (crate) const KITCHEN_SWITCH : &str = "kitchen_switch";

#[derive(Debug)]
pub (crate) struct KitchenSwitchDevice {
    pub setup : bool,
}

impl KitchenSwitchDevice {
    pub(crate) fn _new() -> Self {
        Self {setup: false}
    }

    pub fn get_name() -> &'static str {
        KITCHEN_SWITCH
    }
}

impl DynDevice for KitchenSwitchDevice {

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( InterSwitch::from_json(msg)? ))
    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"state":""}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, _last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_switch()
    }

}
