use std::slice::RSplit;
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct LampColor {
    pub hue: Option<u32>,
    pub saturation: Option<u32>,
    pub x:f32,
    pub y:f32,
}

pub (crate) trait DeviceMessage {
    fn as_lamp_rgb(&self) -> &'_ LampRGB;
    fn as_inter_switch(&self) -> &'_ InterSwitch;

    fn to_json(&self) -> serde_json::error::Result<String>;

    fn to_inter_switch(&self) -> /*InterSwitch*/ Box<dyn DeviceMessage> {
        todo!()
    }

    fn to_inter_dim(&self) -> /*InterDim*/ Box<dyn DeviceMessage>  {
        todo!()
    }

    fn to_lamp_rgb(&self, last_message : &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        todo!()
    }
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
            brightness: 40,
            state: "".to_string()
        }
    }
}

impl DeviceMessage for LampRGB {
    fn as_lamp_rgb(&self) -> &'_ LampRGB {
        self
    }

    fn as_inter_switch(&self) -> &'_ InterSwitch {
        todo!()
    }

    fn to_json(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }

    fn to_inter_switch(&self) -> Box<dyn DeviceMessage>  {
        Box::new(InterSwitch {
            state: self.state.clone(),
        })
    }

    fn to_inter_dim(&self) -> Box<dyn DeviceMessage>  {
        Box::new(InterDim {
            brightness: self.brightness,
            state: self.state.clone(),
        })
    }

    fn to_lamp_rgb(&self, last_message : &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        let rgb = last_message.as_lamp_rgb();
        Box::new(LampRGB {
            color: rgb.color.clone(),
            brightness: self.brightness,
            state: self.state.clone(),
        })
    }

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterSwitch {
    pub state: String,
}

impl InterSwitch {
    pub fn new() -> Self {
        Self {
            state: "OFF".to_string()
        }
    }
}

impl DeviceMessage for InterSwitch {
    fn as_lamp_rgb(&self) -> &'_ LampRGB {
        todo!()
    }

    fn as_inter_switch(&self) -> &'_ InterSwitch {
       self
    }

    fn to_json(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }
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

impl DeviceMessage for InterDim {
    fn as_lamp_rgb(&self) -> &'_ LampRGB {
        todo!()
    }

    fn as_inter_switch(&self) -> &'_ InterSwitch {
        todo!()
    }

    fn to_json(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }
}