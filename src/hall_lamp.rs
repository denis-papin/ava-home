
///-------------
use std::cell::{Cell, RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{LampRGB, Locks, publish};

#[derive(Debug)]
pub (crate) struct HallLampDevice {
}

impl HallLampDevice {
    pub fn new() -> Self {
        Self {

        }
    }

    pub (crate) fn init(&self, topic : &str, msg : &str) -> Option<LampRGB> {
        if topic == "zigbee2mqtt/hall_lamp" {
            info!("✨ Init HALL LAMP");
            let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
            Some(match r_info {
                Ok(lamp) => { lamp }
                Err(e) => {
                    panic!("💀 Cannot parse the message for the HALL LAMP : {e}");
                }
            })
        } else {
            None
        }
    }

    pub (crate) fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {

        // let mut locks = rc_locks.get_mut();
        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            info!(">>>>>>>>>>> rc_locks before LAMP {:?}", &locks);

            // let mut last_lamp_rgb = LampRGB {
            //     color: None/*Some(LampColor {
            //     hue: 0,
            //     saturation: 0,
            //     x: 0.0,
            //     y: 0.0
            // })*/,
            //     brightness: 0,
            //     state: "".to_string()
            // };
            if topic == "zigbee2mqtt/hall_lamp" {
                info!(">>>>>>>>>> execute device HALL LAMP");
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

                let lamp_rgb = match r_info {
                    Ok(lamp) => { lamp }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the HALL LAMP :  {e}");
                    }
                };

                if locks.hall_lamp_locks > 0 {
                    info!("⛔ HALL LAMP Here we are, {:?} ", &lamp_rgb);
                    info!("HALL LAMP IS LOCKED BY THE DIMMER ({}): {}", topic, msg);
                    locks.hall_lamp_locks -= 1;
                } else {
                    if lamp_rgb == locks.last_hall_lamp {
                        info!("⛔ HALL LAMP [same message], {:?} ", &lamp_rgb);
                    } else {
                        info!("🍺 HALL LAMP Here we are, {:?} ", &lamp_rgb);
                        info!("PROCESS HALL LAMP ({}): {}", topic, msg);
                        locks.dim_locks += 1;
                        let message = format!("{{\"brightness\":{},\"state\":\"{}\"}}", lamp_rgb.brightness, &lamp_rgb.state);
                        publish(&mut pub_stream, "zigbee2mqtt/kitchen_inter_dim/set", &message);


                        //
                        locks.lamp_locks += 1;
                        let lamp_rgb = LampRGB {
                            color: locks.last_kitchen_lamp.color.clone(),
                            brightness: lamp_rgb.brightness,
                            state: lamp_rgb.state.clone(),
                        };

                        let message = serde_json::to_string(&lamp_rgb).unwrap();
                        info!("➡ Prepare to be sent to the lamp, {:?} ", &message);
                        publish(&mut pub_stream, "zigbee2mqtt/kitchen_lamp/set", &message);


                        // locks.switch_locks += 1;
                        // let message = format!("{{\"state\":\"{}\"}}", &lamp_rgb.state);
                        // publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                    }
                    locks.last_hall_lamp = lamp_rgb;
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());

    }
}
