
use serde_derive::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct LampColor {
    // pub hue: Option<u32>,
    // pub saturation: Option<u32>,
    pub x:f32,
    pub y:f32,
}

pub (crate) trait DeviceMessage {
    fn as_lamp_rgb(&self) -> &'_ LampRGB {
        todo!()
    }
    fn as_inter_switch(&self) -> &'_ InterSwitch{
        todo!()
    }
    fn as_inter_dim(&self) -> &'_ InterDim{
        todo!()
    }
    fn as_temp_sensor(&self) -> &'_ TempSensor{
        todo!()
    }

    fn to_json(&self) -> serde_json::error::Result<String>;

    fn to_inter_switch(&self) -> /*InterSwitch*/ Box<dyn DeviceMessage> {
        todo!()
    }

    fn to_inter_dim(&self) -> /*InterDim*/ Box<dyn DeviceMessage>  {
        todo!()
    }

    fn to_lamp_rgb(&self, _last_message : &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        todo!()
    }

    fn to_temp_sensor(&self, _last_message : &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct LampRGB {
    // There are 2 different modes : color xy for RGB and color temp for white lamps
    // pub color : LampColor,
    pub color_temp:u16,
    pub brightness:u16,
    pub state: String,
}

//
impl LampRGB {
    pub (crate) fn new() -> Self {
        Self {
            // There are 2 different modes : color xy for RGB and color temp for white lamps
            // color: LampColor {
            //     // hue: None,
            //     // saturation: None,
            //     x: 0.0,
            //     y: 0.0
            // },
            color_temp: 270,
            brightness: 40,
            state: "".to_string()
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
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
            //color: rgb.color.clone(),
            color_temp: self.color_temp,
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

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

impl DeviceMessage for InterSwitch {
    fn as_lamp_rgb(&self) -> &'_ LampRGB {
        todo!()
    }

    fn as_inter_switch(&self) -> &'_ InterSwitch {
       self
    }

    fn as_inter_dim(&self) -> &'_ InterDim {
        todo!()
    }

    fn to_json(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }

}

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

impl DeviceMessage for TempSensor {

    fn as_temp_sensor(&self) -> &'_ TempSensor{
        self
    }

    fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterDim {
    pub brightness:u16,
    // linkquality:u8,
    pub state: String,
}

impl InterDim {
    pub fn new() -> Self {
        Self {
            brightness: 0,
            state: "OFF".to_string()
        }
    }

    pub (crate) fn from_json(msg: &str) -> Result<Self, String> {
        serde_json::from_str(msg).map_err(|e| e.to_string())
    }
}

impl DeviceMessage for InterDim {

    fn as_lamp_rgb(&self) -> &'_ LampRGB {
        todo!()
    }

    fn as_inter_switch(&self) -> &'_ InterSwitch {
        todo!()
    }

    fn as_inter_dim(&self) -> &'_ InterDim {
        self
    }

    fn to_lamp_rgb(&self, last_message : &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("InterDim message conversion to Rgb : {}", &last_message.to_json().unwrap());
        let rgb = last_message.as_lamp_rgb();
        dbg!(rgb);
        Box::new(LampRGB {
            //color: rgb.color.clone(),
            color_temp: rgb.color_temp,
            brightness: self.brightness,
            state: self.state.clone(),
        })
    }

    fn to_json(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }

}