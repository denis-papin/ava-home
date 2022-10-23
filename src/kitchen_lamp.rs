use std::cell::{Cell, RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{Locks, publish};


// "color":{"hue":4,
//          "saturation":97,
//          "x":0.640625,
//          "y":0.328125},
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct LampColor {
    pub hue: Option<u32>,
    pub saturation: Option<u32>,
    pub x:f32,
    pub y:f32,
}


// {"brightness":20,
// "color":{"hue":4,
//          "saturation":97,
//          "x":0.640625,
//          "y":0.328125},
// "color_mode":"xy",
// "color_temp":321,
// "color_temp_startup":300,
// "state":"OFF",
// "update":{"state":"idle"}}'
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct LampRGB {
    pub color : LampColor,
    pub brightness:u8,
    pub state: String,
}

#[derive(Debug)]
pub (crate) struct KitchenLampDevice {
}

impl KitchenLampDevice {
    pub fn new() -> Self {
        Self {

        }
    }

    pub (crate) fn init(&self, topic : &str, msg : &str) -> Option<LampRGB> {
        if topic == "zigbee2mqtt/kitchen_lamp" {
            info!("✨ Init LAMP");
            let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
            Some(match r_info {
                Ok(lamp) => { lamp }
                Err(e) => {
                    panic!("💀 Cannot parse the message for the LAMP :  {e}");
                }
            })
        } else {
            None
        }
    }

    pub (crate) fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {

        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            info!(">>>>>>>>>>> rc_locks before LAMP {:?}", &locks);

            if topic == "zigbee2mqtt/kitchen_lamp" {
                info!(">>>>>>>>>> execute device LAMP");
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

                let lamp_rgb = match r_info {
                    Ok(lamp) => { lamp }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the LAMP :  {e}");
                    }
                };

                if locks.lamp_locks > 0 {
                    info!("⛔ LAMP Here we are, {:?} ", &lamp_rgb);
                    info!("LAMP IS LOCKED BY THE DIMMER ({}): {}", topic, msg);
                    locks.lamp_locks -= 1;
                } else {
                    if lamp_rgb == locks.last_kitchen_lamp {
                        info!("⛔ LAMP [same message], {:?} ", &lamp_rgb);
                    } else {
                        info!("🍺 LAMP Here we are, {:?} ", &lamp_rgb);
                        info!("PROCESS LAMP ({}): {}", topic, msg);
                        locks.dim_locks += 1;
                        let message = format!("{{\"brightness\":{},\"state\":\"{}\"}}", lamp_rgb.brightness, &lamp_rgb.state);
                        publish(&mut pub_stream, "zigbee2mqtt/kitchen_inter_dim/set", &message);

                        //
                        locks.hall_lamp_locks += 1;
                        let lamp_basic = LampRGB {
                            color: locks.last_hall_lamp.color.clone(),
                            brightness: lamp_rgb.brightness,
                            state: lamp_rgb.state.clone(),
                        };

                        let message = serde_json::to_string(&lamp_basic).unwrap();
                        info!("➡ Prepare to be sent to the hall lamp, {:?} ", &message);
                        publish(&mut pub_stream, "zigbee2mqtt/hall_lamp/set", &message);

                        // locks.switch_locks += 1;
                        // let message = format!("{{\"state\":\"{}\"}}", &lamp_rgb.state);
                        // publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                    }
                    locks.last_kitchen_lamp = lamp_rgb;
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());

    }
}

