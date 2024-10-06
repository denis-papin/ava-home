
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct LampColor {
    pub x:f32,
    pub y:f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct LampRGB {
    // There are 2 different modes : color xy for RGB and color temp for white lamps
    // pub color_mode: String, // "xy"
    pub color : LampColor,
    // pub color_temp:u16,
    pub brightness:u16,
    pub state: String,
}

//
impl LampRGB {
    pub (crate) fn new() -> Self {
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

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct MoveSensor {
    pub occupancy: bool,
}

impl MoveSensor {
    pub fn new() -> Self {
        Self {
            occupancy: false
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}
