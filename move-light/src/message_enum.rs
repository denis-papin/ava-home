use log::info;
use crate::device_message::{LampRGB, MoveSensor};
use crate::message_enum::MessageEnum::{LAMP_RGB, MOVE_SENSOR};

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

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    LAMP_RGB(LampRGB),
    MOVE_SENSOR(MoveSensor)
}



impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            LAMP_RGB(_) => {
                let msg =  r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            MOVE_SENSOR(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            LAMP_RGB(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            MOVE_SENSOR(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            LAMP_RGB(_) => {
                Ok(LAMP_RGB(LampRGB::from_json(json_msg)?))
            }
            MOVE_SENSOR(_) => {
                Ok(MOVE_SENSOR(MoveSensor::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_lamp_rgb() -> Self {
        LAMP_RGB(LampRGB::new())
    }

    pub (crate) fn default_move_sensor() -> Self {
        MOVE_SENSOR(MoveSensor::new())
    }

    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            LAMP_RGB(_) => {
                original_message.to_lamp_rgb(&last_message)
            }
            MOVE_SENSOR(_) => {
                original_message.to_move_sensor(&last_message)
            }
        }
    }

    /// Convert the current type of message to LampRGB
    fn to_lamp_rgb(&self, last_message: &MessageEnum) -> Self {
        // We know the "last_message" is of type LAMP_RGB
        let rgb = match last_message {
            LAMP_RGB(rgb) => {
                rgb
            }
            _ => {
                panic!("last message must be of type LAMP_RGB")
            }
        };

        let ret = match self {
            LAMP_RGB(msg) => {
                LAMP_RGB(LampRGB {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: msg.state.clone(),
                })
            }
            MOVE_SENSOR(msg) => {
               // MOVE_SENSOR(MoveSensor::new())
                let state = if msg.occupancy {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                };

                LAMP_RGB(LampRGB {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state,
                })
            }
        };
        let ret = ensure_specific_enum!(ret, LAMP_RGB);
        ret
    }

    fn to_move_sensor(&self, last_message: &MessageEnum) -> Self {
        let move_sensor = match last_message {
            MOVE_SENSOR(move_sensor) => {
                move_sensor
            }
            _ => {
                panic!("last message must be of type MOVE_SENSOR")
            }
        };
        let ret = MOVE_SENSOR(move_sensor.clone());
        let ret = ensure_specific_enum!(ret, MOVE_SENSOR);
        ret
    }


}