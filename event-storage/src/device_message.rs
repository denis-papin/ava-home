use serde_derive::*;

use crate::device_message::RadiatorMode::{CFT, ECO, FRO, STOP};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct TempSensor {
    pub battery : f32,
    pub humidity :f32,
    pub linkquality : u32,
    pub temperature: f32,
    pub voltage: u32,
}

impl TempSensor {
    pub (crate) fn new() -> Self {
        Self {
            battery: 0.0,
            humidity: 0.0,
            linkquality: 0,
            temperature: 0.0,
            voltage: 0,
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}


#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) enum RadiatorMode {
    CFT,
    ECO,
    FRO,
    STOP
}

impl RadiatorMode {
    pub(crate) fn from_value(value : String) -> Self {
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
            mode: FRO
        }
    }

    pub (crate) fn from_mode(mode: RadiatorMode) -> Self {
        Self {
            mode
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}