use std::collections::HashMap;
use ava_toolkit::device_message::{LampRgbMsg, MoveSensorMsg};
use ava_toolkit::generic_device::Locality;
use crate::message_enum::MessageEnum::{LampRgb, MoveSensor};

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
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    LampRgb(LampRgbMsg),
    MoveSensor(MoveSensorMsg)
}

impl Locality for MessageEnum {

    fn query_for_state(&self) -> String {
        match self {
            LampRgb(_) => {
                let msg =  r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            MoveSensor(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    fn raw_message(&self) -> String {
        match self {
            LampRgb(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            MoveSensor(msg) => {
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
            MoveSensor(_) => {
                original_message.to_move_sensor(&last_message)
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
            MoveSensor(_) => {
                Ok(MoveSensor(MoveSensorMsg::from_json(json_msg)?))
            }
        }
    }

    async fn process(&self, _topic: &str, _args: &[String]) {
        unreachable!()
    }
}


impl MessageEnum {

    pub (crate) fn default_lamp_rgb() -> Self {
        LampRgb(LampRgbMsg::new())
    }

    pub (crate) fn default_move_sensor() -> Self {
        MoveSensor(MoveSensorMsg::new())
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

        let ret = match self {
            LampRgb(msg) => {
                LampRgb(LampRgbMsg {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: msg.state.clone(),
                })
            }
            MoveSensor(msg) => {
               // MOVE_SENSOR(MoveSensor::new())
                let state = if msg.occupancy {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                };

                LampRgb(LampRgbMsg {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state,
                })
            }
        };
        let ret = ensure_specific_enum!(ret, LampRgb);
        ret
    }

    fn to_move_sensor(&self, last_message: &MessageEnum) -> Self {
        let move_sensor = match last_message {
            MoveSensor(move_sensor) => {
                move_sensor
            }
            _ => {
                panic!("last message must be of type MOVE_SENSOR")
            }
        };
        let ret = MoveSensor(move_sensor.clone());
        let ret = ensure_specific_enum!(ret, MoveSensor);
        ret
    }

}