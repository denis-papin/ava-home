use std::collections::HashMap;
use ava_toolkit::device_message::{RegulationMapMsg};
use ava_toolkit::generic_device::Locality;
use crate::message_enum::MessageEnum::RegulationMsg;

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    RegulationMsg(RegulationMapMsg),
}

impl MessageEnum {
    /// Convert the current type of message to Temperature Sensor
    fn to_regulation(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    pub(crate) fn default_regulation() -> Self {
        RegulationMsg(RegulationMapMsg::new())
    }
}

impl Locality for MessageEnum {
    fn query_for_state(&self) -> String {
        match self {
            RegulationMsg(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    fn raw_message(&self) -> String {
        match self {
            RegulationMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    
    /// Convert the original message to the type of the current Self
    fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            RegulationMsg(_) => {
                original_message.to_regulation(&last_message)
            }
        }
    }

    fn to_local_with_data(&self, original_message: &Self, last_message: &Self, _o_ext_data: Option<&HashMap<String, f64>>, _o_topic: Option<&str>) -> Self {
        self.to_local(original_message, last_message)
    }
    
    fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            RegulationMsg(_) => {
                Ok(RegulationMsg(RegulationMapMsg::from_json(json_msg)?))
            }
        }
    }


    /// Non disponible
    async fn process(&self, _topic: &str, _args: &[String]) {
        unimplemented!()
    }

    async fn compute(&self) -> Option<HashMap<String, f64>> {
        None
    }
}
