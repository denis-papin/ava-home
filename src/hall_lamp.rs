
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;

use crate::{DeviceMessage, DynDevice, InterDim, KitchenInterDimDevice, KitchenLampDevice, LampRGB, Locks, publish};

pub (crate) const HALL_LAMP : &str = "hall_lamp";

#[derive(Debug)]
pub (crate) struct HallLampDevice {
    pub setup : bool,
}

impl HallLampDevice {
    pub fn new() -> Self {
        Self {setup : false}
    }

    pub fn receive(mut pub_stream: &mut TcpStream, lamp_rgb : LampRGB ) {
        match serde_json::to_string(&lamp_rgb) {
            Ok(message) => {
                info!("➡ Prepare to be sent to the {}, {:?} ", Self::get_name(), &message);
                publish(&mut pub_stream, &format!("zigbee2mqtt/{}/set", Self::get_name()), &message);
            }
            Err(_) => {
                error!("💣 Impossible to parse the message :{:?}", &lamp_rgb);
            }
        }
    }

    pub fn get_name() -> &'static str {
        HALL_LAMP
    }
}

impl DynDevice for HallLampDevice {
    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn init(&mut self, topic : &str, msg : &str, arc_locks: Arc<RefCell<Locks>>) {
        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() {
                info!("✨ Init hall_lamp");
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
                match r_info {
                    Ok(lamp) => {
                        self.setup = true;
                        locks.hall_lamp_lock.replace(lamp);
                    }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the HALL LAMP :  {e}");
                    }
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {

        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            // info!(">>>>>>>>>>> rc_locks before LAMP {:?}", &locks);
            if topic == &self.get_topic() {
                info!("Execute device {}", Self::get_name());
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

                let lamp_rgb = match r_info {
                    Ok(lamp) => { lamp }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the HALL LAMP :  {e}");
                    }
                };

                if locks.hall_lamp_lock.count_locks > 0 {
                    info!("⛔ HALL LAMP Here we are, {:?} ", &lamp_rgb);
                    info!("HALL LAMP IS LOCKED BY THE DIMMER ({}): {}", topic, msg);
                    locks.hall_lamp_lock.dec();
                } else {
                    if lamp_rgb == locks.hall_lamp_lock.last_object_message {
                        info!("⛔ HALL LAMP [same message], {:?} ", &lamp_rgb);
                    } else {
                        info!("🍺 HALL LAMP Here we are, {:?} ", &lamp_rgb);
                        info!("PROCESS HALL LAMP ({}): {}", topic, msg);

                        locks.kitchen_inter_dim_lock.inc();
                        let inter_dim = InterDim {
                            brightness: lamp_rgb.brightness,
                            state: lamp_rgb.state.clone(),
                        };
                        KitchenInterDimDevice::receive(&mut pub_stream, inter_dim);

                        //
                        locks.kitchen_lamp_lock.inc();
                        let lamp_rgb = LampRGB {
                            color: locks.kitchen_lamp_lock.last_object_message.color.clone(),
                            brightness: lamp_rgb.brightness,
                            state: lamp_rgb.state.clone(),
                        };

                        KitchenLampDevice::receive(&mut pub_stream, lamp_rgb);

                        // locks.switch_locks += 1;
                        // let message = format!("{{\"state\":\"{}\"}}", &lamp_rgb.state);
                        // publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                    }
                    locks.hall_lamp_lock.replace(lamp_rgb);
                }

            }
            locks
        };
        arc_locks.replace(locks.clone());

    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
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
