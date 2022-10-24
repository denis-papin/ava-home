use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::kitchen_lamp::LampRGB;
use crate::{DynDevice, Locks, publish, KitchenLampDevice, HallLampDevice, DeviceMessage};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub (crate) struct InterDim {
    pub brightness:u8,
    // linkquality:u8,
    pub state: String,
}

pub (crate) const KITCHEN_INTER_DIM : &str = "kitchen_inter_dim";

#[derive(Debug)]
pub (crate) struct KitchenInterDimDevice {
    // pub locks : RefCell<Locks>,
}

impl KitchenInterDimDevice {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub fn receive(mut pub_stream: &mut TcpStream, inter_dim : InterDim ) {
        let message = serde_json::to_string(&inter_dim).unwrap(); // TODO VI
        info!("➡ Prepare to be sent to the {}, {:?} ", Self::get_name(), &message);
        publish(&mut pub_stream, &format!("zigbee2mqtt/{}/set", Self::get_name()), &message);
    }

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


    // TODO handle the locks
    //      Create a generic BasicDevice to make this routine general
    //      Build closure to process the publish to other devices
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

                        KitchenLampDevice::receive(&mut pub_stream, lamp_rgb);

                        locks.hall_lamp_lock.inc();
                        let lamp_basic = LampRGB {
                            color: locks.hall_lamp_lock.last_object_message.color.clone(),
                            brightness: inter_dim.brightness,
                            state: inter_dim.state.clone(),
                        };

                        HallLampDevice::receive(&mut pub_stream, lamp_basic);

                        // locks.switch_locks += 1;
                        // let message = format!("{{\"state\":\"{}\"}}", &inter_dim.state);
                        // publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
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

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        todo!()
    }

    fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
        todo!()
    }

    fn forward_messages(&self, pub_stream: &mut TcpStream, locks: &mut Locks, object_message: Box<dyn DeviceMessage>) {
        todo!()
    }
}
