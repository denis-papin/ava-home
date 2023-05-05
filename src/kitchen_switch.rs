
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


    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        InterSwitch::from_json(msg)
    }


    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<InterSwitch, _> = serde_json::from_str(msg);

        match r_info {
            Ok(lamp) => { Box::new(lamp) }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(InterSwitch::new())
            }
        }
    }


    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"state":""}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, _last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_switch()
    }


}
