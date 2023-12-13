use crate::device_message::{InterDim, InterSwitch, LampRGB};
use crate::message_enum::MessageEnum::{INTER_DIMMER, INTER_SWITCH, LAMP_RGB};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    LAMP_RGB(LampRGB),
    INTER_DIMMER(InterDim),
    INTER_SWITCH(InterSwitch),
}



impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            LAMP_RGB(_) => {
                let msg =  r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            INTER_DIMMER(_) => {
                let msg = r#"{"color":{"x":"","y":""}}"#;
                msg.to_string()
            }
            INTER_SWITCH(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            LAMP_RGB(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
                // msg.to_owned()
            }
            INTER_DIMMER(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            INTER_SWITCH(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            LAMP_RGB(_) => {
                Ok(LAMP_RGB(LampRGB::from_json(json_msg)?))
            }
            INTER_DIMMER(_) => {
                Ok(INTER_DIMMER(InterDim::from_json(json_msg)?))
            }
            INTER_SWITCH(_) => {
                Ok(INTER_SWITCH(InterSwitch::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_lamp_rgb() -> Self {
        LAMP_RGB(LampRGB::new())
    }

    pub (crate) fn default_inter_dim() -> Self {
        INTER_DIMMER(InterDim::new())
    }

    pub (crate) fn default_inter_switch() -> Self {
        INTER_SWITCH(InterSwitch::new())
    }

    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            LAMP_RGB(_) => {
                original_message.to_lamp_rgb(&last_message)
            }
            INTER_DIMMER(_) => {
                original_message.to_inter_dim(&last_message)
            }
            INTER_SWITCH(_) => {
                original_message.to_inter_switch(&last_message)
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

        match self {
            LAMP_RGB(msg) => {
                LAMP_RGB(LampRGB {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: rgb.brightness,
                    state: msg.state.clone(),
                })

            }
            INTER_DIMMER(msg) => {
                LAMP_RGB(LampRGB {
                    //color_mode: rgb.color_mode.clone(),
                    color: rgb.color.clone(),
                    brightness: msg.brightness,
                    state: msg.state.clone(),
                })
            }
            INTER_SWITCH(msg) => {
                LAMP_RGB(LampRGB {
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
            LAMP_RGB(msg) => {
                INTER_SWITCH(InterSwitch {
                    state: msg.state.clone(),
                })
            }
            INTER_DIMMER(msg) => {
                INTER_SWITCH(InterSwitch {
                    state: msg.state.clone(),
                })
            }
            INTER_SWITCH(msg) => {
                INTER_SWITCH(msg.clone())
            }
        }
    }


    /// Convert the current type of message to InterSwitch
    fn to_inter_dim(&self, last_message: &MessageEnum) -> Self {
        // We know the "last_message" is of type INTER_DIMMER
        let inter = match last_message {
            INTER_DIMMER(inter) => {
                inter
            }
            _ => {
                panic!("last message must be of type LAMP_RGB")
            }
        };

        match self {
            LAMP_RGB(msg) => {
                INTER_DIMMER(InterDim {
                    brightness: msg.brightness,
                    state: msg.state.clone(),
                })
            }
            INTER_DIMMER(msg) => {
                INTER_DIMMER(msg.clone())
            }
            INTER_SWITCH(msg) => {
                INTER_DIMMER(InterDim {
                    brightness: inter.brightness,
                    state: msg.state.clone(),
                })
            }
        }
    }

}