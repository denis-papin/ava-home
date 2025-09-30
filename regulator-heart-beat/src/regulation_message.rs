use ava_toolkit::device_message::RegulationMapMsg;

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    RegulationMsg(RegulationMapMsg),
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

