use serde_derive::*;

use crate::device_message::RadiatorMode::{CFT, ECO, FRO, STOP};

// TODO Put that in a common lib
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) enum RadiatorMode {
    CFT,
    ECO,
    FRO,
    STOP
}

impl RadiatorMode {
    fn from_value(value : String) -> Self {
        match value.as_str() {
            "cft" => CFT,
            "eco" => ECO,
            "fro" => FRO,
            _ => STOP,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct Radiator {
    pub mode: RadiatorMode
}

impl Radiator {
    pub (crate) fn new() -> Self {
        Self {
            mode: STOP
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

