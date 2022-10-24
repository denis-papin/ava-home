use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct LampColor {
    pub hue: Option<u32>,
    pub saturation: Option<u32>,
    pub x:f32,
    pub y:f32,
}

pub (crate) trait DeviceMessage {
    fn to_lamp_rgb(&self) -> &'_ LampRGB;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct LampRGB {
    pub color : LampColor,
    pub brightness:u8,
    pub state: String,
}

//
impl LampRGB {
    pub (crate) fn new() -> Self {
        Self {
            color: LampColor {
                hue: None,
                saturation: None,
                x: 0.0,
                y: 0.0
            },
            brightness: 0,
            state: "".to_string()
        }
    }
}

impl DeviceMessage for LampRGB {
    fn to_lamp_rgb(&self) -> &'_ LampRGB {
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterSwitch {
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct TempSensor {
    pub battery : u32,
    pub humidity :f32,
    pub linkquality : u32,
    pub temperature: f32,
    pub voltage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterDim {
    pub brightness:u8,
    // linkquality:u8,
    pub state: String,
}