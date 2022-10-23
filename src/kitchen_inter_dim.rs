use std::cell::{Cell, RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::kitchen_lamp::LampRGB;
use crate::{Locks, publish};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterDim {
    pub brightness:u8,
    // linkquality:u8,
    pub state: String,
}

#[derive(Debug)]
pub (crate) struct KitchenInterDimDevice {
    // pub locks : RefCell<Locks>,
}

impl KitchenInterDimDevice {
    pub (crate)  fn new() -> Self {
        Self {
            // locks,
        }
    }

    // TODO handle the locks
    //      Create a generic BasicDevice to make this routine general
    //      Build closure to process the publish to other devices
    pub (crate)  fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream,  arc_locks : Arc<RefCell<Locks>>) {
        let locks = {
            // let mut locks = rc_locks.get_mut();
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == "zigbee2mqtt/kitchen_inter_dim" {
                info!(">>>>>>>>>> execute device DIMMER");
                let r_info: Result<InterDim, _> = serde_json::from_str(msg);
                let inter_dim = r_info.unwrap();

                if locks.dim_locks > 0 {
                    info!("⛔ DIMMER MESSAGE Here we are, {:?} ", &inter_dim);
                    info!("DIMMER IS LOCKED BY THE DIMMER ({}): {}", topic, msg);
                    locks.dim_locks -= 1;
                } else {
                    if inter_dim == locks.last_inter_dim {
                        info!("⛔ DIMMER [same message], {:?} ", &inter_dim);
                    } else {
                        info!("🍺 DIMMER MESSAGE Here we are, {:?} ", &inter_dim);

                        locks.lamp_locks += 1;
                        let lamp_basic = LampRGB {
                            color: locks.last_kitchen_lamp.color.clone(),
                            brightness: inter_dim.brightness,
                            state: inter_dim.state.clone(),
                        };

                        let message = serde_json::to_string(&lamp_basic).unwrap();
                        info!("➡ Prepare to be sent to the lamp, {:?} ", &message);
                        publish(&mut pub_stream, "zigbee2mqtt/kitchen_lamp/set", &message);


                        locks.hall_lamp_locks += 1;
                        let lamp_basic = LampRGB {
                            color: locks.last_hall_lamp.color.clone(),
                            brightness: inter_dim.brightness,
                            state: inter_dim.state.clone(),
                        };

                        let message = serde_json::to_string(&lamp_basic).unwrap();
                        info!("➡ Prepare to be sent to the hall lamp, {:?} ", &message);
                        publish(&mut pub_stream, "zigbee2mqtt/hall_lamp/set", &message);

                        // locks.switch_locks += 1;
                        // let message = format!("{{\"state\":\"{}\"}}", &inter_dim.state);
                        // publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                    }
                }
                locks.last_inter_dim = inter_dim;
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }
}
