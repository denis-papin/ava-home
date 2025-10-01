use serde_derive::*;

use crate::device_message::RadiatorMode::FRO;

pub const ZIGBEE_FAMILY : &str = "zigbee2mqtt";

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct RegulationMap {
    pub tc_bureau: f32,
    pub tc_salon_1: f32,
    pub tc_salon_2: f32,
    pub tc_chambre_1: f32,
    pub tc_couloir: f32,
}

impl RegulationMap {
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
pub struct TempSensor {
    pub battery : f32,
    pub humidity :f32,
    pub linkquality : u32,
    pub temperature: f32,
}

impl TempSensor {
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
pub struct MoveSensor {
    pub battery : Option<f32>,
    pub linkquality : u32,
    pub occupancy: bool,
}

impl MoveSensor {
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
pub struct BasicSwitch {
    pub battery: f32,
    pub linkquality: u32,
    pub action: String,
}

impl BasicSwitch {
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
pub struct Radiator {
    pub mode: RadiatorMode
}

impl Radiator {
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
pub struct LampColor {
    pub x:f32,
    pub y:f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LampRGB {
    // There are 2 different modes : color xy for RGB and color temp for white lamps
    // pub color_mode: String, // "xy"
    pub color : LampColor,
    // pub color_temp:u16,
    pub brightness:u16,
    pub state: String,
}

impl LampRGB {
    pub fn new() -> Self {
        Self {
            // There are 2 different modes : color xy for RGB and color temp for white lamps
            // color_mode: "xy".to_string(),
            color: LampColor {
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
pub struct SimpleSwitch {
    pub action: String,
}

impl SimpleSwitch {
    pub fn new() -> Self {
        Self {
            action: "single".to_string()
        }
    }

    pub fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

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