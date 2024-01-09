
use serde_derive::*;
//
// #[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
// pub (crate) struct LampColor {
//     pub x:f32,
//     pub y:f32,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub (crate) struct LampRGB {
//     // There are 2 different modes : color xy for RGB and color temp for white lamps
//     // pub color_mode: String, // "xy"
//     pub color : LampColor,
//     // pub color_temp:u16,
//     pub brightness:u16,
//     pub state: String,
// }
//
// //
// impl LampRGB {
//     pub (crate) fn new() -> Self {
//         Self {
//             // There are 2 different modes : color xy for RGB and color temp for white lamps
//             // color_mode: "xy".to_string(),
//             color: LampColor {
//                 x: 0.0,
//                 y: 0.0
//             },
//             //color_temp: 270,
//             brightness: 40,
//             state: "OFF".to_string()
//         }
//     }
//
//     pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
//         serde_json::from_str(msg).map_err(|e| e.to_string())
//     }
// }
//
//
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub (crate) struct InterSwitch {
//     pub state: String,
// }
//
// impl InterSwitch {
//     pub fn new() -> Self {
//         Self {
//             state: "OFF".to_string()
//         }
//     }
//
//     pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
//         serde_json::from_str(msg).map_err(|e| e.to_string())
//     }
// }

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

//
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub (crate) struct InterDim {
//     pub brightness:u16,
//     // linkquality:u8,
//     pub state: String,
// }
//
// impl InterDim {
//     pub fn new() -> Self {
//         Self {
//             brightness: 0,
//             state: "OFF".to_string()
//         }
//     }
//
//     pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
//         serde_json::from_str(msg).map_err(|e| e.to_string())
//     }
// }
