use std::collections::HashMap;
use ava_toolkit::device_message::{InterDimMsg, InterSwitchMsg, LampRgbMsg};
use ava_toolkit::generic_device::Locality;
use crate::message_enum::MessageEnum::{InterDimmer, InterSwitch, LampRgb};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    LampRgb(LampRgbMsg),
    InterDimmer(InterDimMsg),
    InterSwitch(InterSwitchMsg),
}

impl Locality for MessageEnum {

    fn query_for_state(&self) -> String {
        match self {
            LampRgb(_) => {
                let msg =  r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            InterDimmer(_) => {
                let msg = r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            InterSwitch(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }
    fn raw_message(&self) -> String {
        match self {
            LampRgb(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
                // msg.to_owned()
            }
            InterDimmer(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            InterSwitch(msg) => {
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
            InterDimmer(_) => {
                original_message.to_inter_dim(&last_message)
            }
            InterSwitch(_) => {
                original_message.to_inter_switch(&last_message)
            }
        }
    }

    fn to_local_with_data(&self, _original_message: &Self, _last_message: &Self, _ext_data: Option<&HashMap<String, f64>>, _topic: Option<&str>) -> Self {
        todo!()
    }

    fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            LampRgb(_) => {
                Ok(LampRgb(LampRgbMsg::from_json(json_msg)?))
            }
            InterDimmer(_) => {
                Ok(InterDimmer(InterDimMsg::from_json(json_msg)?))
            }
            InterSwitch(_) => {
                Ok(InterSwitch(InterSwitchMsg::from_json(json_msg)?))
            }
        }
    }

    async fn process(&self, _topic: &str, _args: &[String]) {
        unimplemented!()
    }
}

impl MessageEnum {

    pub (crate) fn default_lamp_rgb() -> Self {
        LampRgb(LampRgbMsg::new())
    }

    pub (crate) fn _default_inter_dim() -> Self {
        InterDimmer(InterDimMsg::new())
    }

    pub (crate) fn default_inter_switch() -> Self {
        InterSwitch(InterSwitchMsg::new())
    }


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

        match self {
            LampRgb(msg) => {
                LampRgb(LampRgbMsg {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: msg.state.clone(),
                })

            }
            InterDimmer(msg) => {
                LampRgb(LampRgbMsg {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: msg.brightness,
                    state: msg.state.clone(),
                })
            }
            InterSwitch(msg) => {
                LampRgb(LampRgbMsg {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: msg.state.clone(),
                })
            }
        }
    }

    /// Convert the current type of message to InterSwitch
    fn to_inter_switch(&self, _last_message: &MessageEnum) -> Self {
        match self {
            LampRgb(msg) => {
                InterSwitch(InterSwitchMsg {
                    state: msg.state.clone(),
                })
            }
            InterDimmer(msg) => {
                InterSwitch(InterSwitchMsg {
                    state: msg.state.clone(),
                })
            }
            InterSwitch(msg) => {
                InterSwitch(msg.clone())
            }
        }
    }


    /// Convert the current type of message to InterSwitch
    fn to_inter_dim(&self, last_message: &MessageEnum) -> Self {
        // We know the "last_message" is of type INTER_DIMMER
        let inter = match last_message {
            InterDimmer(inter) => {
                inter
            }
            _ => {
                panic!("last message must be of type LAMP_RGB")
            }
        };

        match self {
            LampRgb(msg) => {
                InterDimmer(InterDimMsg {
                    brightness: msg.brightness,
                    state: msg.state.clone(),
                })
            }
            InterDimmer(msg) => {
                InterDimmer(msg.clone())
            }
            InterSwitch(msg) => {
                InterDimmer(InterDimMsg {
                    brightness: inter.brightness,
                    state: msg.state.clone(),
                })
            }
        }
    }

}