use std::collections::HashMap;
use log::info;
use serde_derive::{Deserialize, Serialize};
use ava_toolkit::device_message::{LampRgbMsg, SimpleSwitchMsg};
use ava_toolkit::generic_device::Locality;
use crate::message_enum::MessageEnum::{LampRgb, SimpleSwitch};

#[macro_export]
macro_rules! ensure_specific_enum {
    ($ret:expr, $enum_type:ident) => {
        match &$ret {
            $enum_type(_) => {
                $ret
            }
            _ => {
                panic!(concat!("The return type must be ", stringify!($enum_type)));
            }
        }
    };
}

/// All kind of messages we can encounter in the pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub (crate) enum MessageEnum {
    LampRgb(LampRgbMsg),
    SimpleSwitch(SimpleSwitchMsg),
}

impl Locality for MessageEnum {

    fn query_for_state(&self) -> String {
        match self {
            LampRgb(_) => {
                let msg =  r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            SimpleSwitch(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    fn find_set_topic(&self, topic: &str) -> String {
        match self {
            LampRgb(_) => {
                format!("{}/set", topic)
            }
            SimpleSwitch(_) => {
                topic.to_string()
            }
        }
    }
    
    fn raw_message(&self) -> String {
        match self {
            LampRgb(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            SimpleSwitch(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    /// Convert the original message to the type of the current Self
   fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            LampRgb(_) => {
                original_message.to_lamp_rgb(&last_message)
            }
            SimpleSwitch(_) => {
                original_message.to_simple_switch(&last_message)
            }
        }
    }

    fn to_local_with_data(&self, original_message: &Self, last_message: &Self, _ext_data: Option<&HashMap<String, f64>>, _topic: Option<&str>) -> Self {
        self.to_local(original_message, last_message)
    }
    
    fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            LampRgb(_) => {
                Ok(LampRgb(LampRgbMsg::from_json(json_msg)?))
            }
            SimpleSwitch(_) => {
                Ok(SimpleSwitch(SimpleSwitchMsg::from_json(json_msg)?))
            }
        }
    }

    async fn process(&self, _topic: &str, _args: &[String]) {
        match self {
            LampRgb(msg) => {
                info!("Run the default empty process for LampRgbMsg, message=[{:?}]", msg);
            }
            SimpleSwitch(msg) => {
                info!("Run the default empty process for SimpleSwitchMsg, message=[{:?}]", msg);
            }
        }
    }

    async fn compute(&self) -> Option<HashMap<String, f64>> {
        None
    }
}


impl MessageEnum {
    
    /// Convert the current type of message to LampRGB
    fn to_lamp_rgb(&self, last_message: &MessageEnum) -> Self {
        // We know the "last_message" is of type LAMP_RGB
        let rgb = match last_message {
            LampRgb(rgb) => {
                rgb
            }
            _ => {
                panic!("last message must be of type LAMP_RGB")
            }
        };

        let ret = match self {
            LampRgb(msg) => {
                LampRgb(LampRgbMsg {
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: msg.state.clone(),
                })
            }
            SimpleSwitch(_msg) => {
                LampRgb(LampRgbMsg {
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: if rgb.state == "ON" { "OFF" } else { "ON" }.to_string(),
                })
            }
        };
        let ret = ensure_specific_enum!(ret, LampRgb);
        ret
    }

    fn to_simple_switch(&self, last_message: &MessageEnum) -> Self {
        let move_sensor = match last_message {
            SimpleSwitch(msg) => {
                msg
            }
            _ => {
                panic!("last message must be of type SimpleSwitch")
            }
        };
        let ret = SimpleSwitch(move_sensor.clone());
        let ret = ensure_specific_enum!(ret, SimpleSwitch);
        ret
    }

}