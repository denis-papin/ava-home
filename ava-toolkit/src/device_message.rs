use serde_derive::*;

use crate::device_message::RadiatorMode::FRO;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct RegulationMapMsg {
    pub tc_bureau: f32,
    pub tc_salon_1: f32,
    pub tc_salon_2: f32,
    pub tc_chambre_1: f32,
    pub tc_couloir: f32,
}

impl RegulationMapMsg {
    pub fn new() -> Self {
        Self {
            tc_bureau: 0.0,
            tc_salon_1: 0.0,
            tc_salon_2: 0.0,
            tc_chambre_1: 0.0,
            tc_couloir: 0.0,
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct TempSensorMsg {
    pub battery : f32,
    pub humidity :f32,
    pub linkquality : u32,
    pub temperature: f32,
}

impl TempSensorMsg {
    pub fn new() -> Self {
        Self {
            battery: 0.0,
            humidity: 0.0,
            linkquality: 0,
            temperature: 0.0,
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct MoveSensorMsg {
    pub battery : Option<f32>,
    pub linkquality : u32,
    pub occupancy: bool,
}

impl MoveSensorMsg {
    pub fn new() -> Self {
        Self {
            battery: Some(0.0),
            linkquality: 0,
            occupancy: false,
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BasicSwitchMsg {
    pub battery: f32,
    pub linkquality: u32,
    pub action: String,
}

impl BasicSwitchMsg {
    pub fn new() -> Self {
        Self {
            battery: 0.0,
            linkquality: 0,
            action: "".to_string(),
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum RadiatorMode {
    CFT,
    ECO,
    FRO,
    STOP
}

// impl RadiatorMode {
//     pub(crate) fn from_value(value : String) -> Self {
//         match value.as_str() {
//             "cft" => CFT,
//             "eco" => ECO,
//             "fro" => FRO,
//             _ => STOP,
//         }
//     }
// }


#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct RadiatorMsg {
    pub mode: RadiatorMode
}

impl RadiatorMsg {
    pub fn new() -> Self {
        Self {
            mode: FRO
        }
    }

    pub fn from_mode(mode: RadiatorMode) -> Self {
        Self {
            mode
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}


#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct LampColorMsg {
    pub x:f32,
    pub y:f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LampRgbMsg {
    // There are 2 different modes : color xy for RGB and color temp for white lamps
    // pub color_mode: String, // "xy"
    pub color : LampColorMsg,
    // pub color_temp:u16,
    pub brightness:u16,
    pub state: String,
}

impl LampRgbMsg {
    pub fn new() -> Self {
        Self {
            // There are 2 different modes : color xy for RGB and color temp for white lamps
            // color_mode: "xy".to_string(),
            color: LampColorMsg {
                x: 0.0,
                y: 0.0
            },
            //color_temp: 270,
            brightness: 40,
            state: "OFF".to_string()
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

///


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InterSwitch {
    pub state: String,
}

impl InterSwitch {
    pub fn new() -> Self {
        Self {
            state: "OFF".to_string()
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InterDim {
    pub brightness:u16,
    pub state: String,
}

impl InterDim {
    pub fn new() -> Self {
        Self {
            brightness: 0,
            state: "OFF".to_string()
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}