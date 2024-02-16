use crate::device_message::RegulationMap;

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    RegulationMsg(RegulationMap),
}

impl MessageEnum {
    pub (crate) fn raw_message(&self) -> String {
        match self {
            MessageEnum::RegulationMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
}

