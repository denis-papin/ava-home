use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;

use crate::{DynDevice, Locks, publish, KitchenLampDevice, HallLampDevice, InterSwitch, KitchenSwitchDevice};
use crate::messages::{DeviceMessage, InterDim, LampRGB};


pub (crate) const KITCHEN_INTER_DIM : &str = "kitchen_inter_dim";

#[derive(Debug)]
pub (crate) struct KitchenInterDimDevice {
    // pub locks : RefCell<Locks>,
}

impl KitchenInterDimDevice {
    pub(crate) fn new() -> Self {
        Self {}
    }

    // pub fn receive(mut pub_stream: &mut TcpStream, inter_dim : InterDim ) {
    //     match serde_json::to_string(&inter_dim) {
    //         Ok(message) => {
    //             info!("➡ Prepare to be sent to the {}, {:?} ", Self::get_name(), &message);
    //             publish(&mut pub_stream, &format!("zigbee2mqtt/{}/set", Self::get_name()), &message);
    //         }
    //         Err(_) => {
    //             error!("💣 Impossible to parse the message :{:?}", &inter_dim);
    //         }
    //     }
    // }

    pub fn get_name() -> &'static str {
        KITCHEN_INTER_DIM
    }
}

impl DynDevice for KitchenInterDimDevice {

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        todo!()
    }

    fn init(&mut self, _topic: &str, _msg: &str, _arc_locks: Arc<RefCell<Locks>>) {
        todo!()
    }


    fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream,  arc_locks : Arc<RefCell<Locks>>) {
        let locks = {
            // let mut locks = rc_locks.get_mut();
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() {
                info!("Execute device {}", self.get_topic());
                let r_info: Result<InterDim, _> = serde_json::from_str(msg);
                let inter_dim = r_info.unwrap();

                if locks.kitchen_inter_dim_lock.count_locks > 0 {
                    info!("⛔ DIMMER MESSAGE Here we are, {:?} ", &inter_dim);
                    info!("DIMMER IS LOCKED BY THE DIMMER ({}): {}", topic, msg);
                    locks.kitchen_inter_dim_lock.dec();
                } else {
                    if inter_dim == locks.kitchen_inter_dim_lock.last_object_message {
                        info!("⛔ DIMMER [same message], {:?} ", &inter_dim);
                    } else {
                        info!("🍺 DIMMER MESSAGE Here we are, {:?} ", &inter_dim);

                        locks.kitchen_lamp_lock.inc();
                        let lamp_rgb = LampRGB {
                            color: locks.kitchen_lamp_lock.last_object_message.color.clone(),
                            brightness: inter_dim.brightness,
                            state: inter_dim.state.clone(),
                        };

                        KitchenLampDevice::new().receive(&mut pub_stream, Box::new(lamp_rgb));

                        locks.hall_lamp_lock.inc();
                        let lamp_basic = LampRGB {
                            color: locks.hall_lamp_lock.last_object_message.color.clone(),
                            brightness: inter_dim.brightness,
                            state: inter_dim.state.clone(),
                        };

                        HallLampDevice::new().receive(&mut pub_stream, Box::new(lamp_basic));

                        locks.kitchen_switch_lock.inc();
                        let inter_switch = InterSwitch {
                            state: inter_dim.state.clone(),
                        };
                        KitchenSwitchDevice::new().receive(&mut pub_stream, Box::new(inter_switch));
                    }
                }
                locks.kitchen_inter_dim_lock.replace(inter_dim);
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    fn replace(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
        todo!()
    }

    fn get_last_object_message(&self, locks: &mut Locks) -> String {
        todo!()
    }

    fn unlock(&self, locks: &mut Locks) {
        todo!()
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        todo!()
    }

    fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
        todo!()
    }

    fn forward_messages(&self, pub_stream: &mut TcpStream, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
        todo!()
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_dim()
    }
}
