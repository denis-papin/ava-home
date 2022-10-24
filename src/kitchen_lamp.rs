use std::any::Any;
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{DynDevice, HallLampDevice, InterDim, KitchenInterDimDevice, Locks, publish};

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

impl DeviceMessage for LampRGB {
    fn to_lamp_rgb(&self) -> &'_ LampRGB {
        self
    }
}

pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";

#[derive(Debug)]
pub (crate) struct KitchenLampDevice {
    pub setup : bool,
}

impl KitchenLampDevice {
    pub fn new() -> Self {
        Self { setup : false }
    }

    pub fn receive(mut pub_stream: &mut TcpStream, lamp_rgb : LampRGB ) {
        let message = serde_json::to_string(&lamp_rgb).unwrap(); // TODO VI
        info!("➡ Prepare to be sent to the {}, {:?} ", Self::get_name(), &message);
        publish(&mut pub_stream, &format!("zigbee2mqtt/{}/set", Self::get_name()), &message);
    }

    pub fn get_name() -> &'static str {
        KITCHEN_LAMP
    }


}

impl DynDevice for KitchenLampDevice {

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
                info!("✨ Init LAMP");
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
                match r_info {
                    Ok(lamp) => {
                        self.setup = true;
                        locks.kitchen_lamp_lock.replace(lamp);
                    }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the LAMP :  {e}");
                    }
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

        match r_info {
            Ok(lamp) => { Box::new(lamp) }
            Err(e) => {
                // TODO Don't break
                panic!("💀 Cannot parse the message for the LAMP :  {e}");
            }
        }
    }

    // TODO change it to return (bool bool)
    fn allowed_to_process(&self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) -> (bool, bool) {
        let lamp_rgb = object_message.to_lamp_rgb();
        let is_locked = if locks.kitchen_lamp_lock.count_locks > 0 {
            info!("⛔ LAMP Here we are, {:?} ", &lamp_rgb);
            info!("LAMP IS LOCKED BY THE DIMMER");
            locks.kitchen_lamp_lock.dec();
            true
        } else {
            false
        };

        info!("Last message  : {:?}", &locks.kitchen_lamp_lock.last_object_message);
        let ok = *lamp_rgb == locks.kitchen_lamp_lock.last_object_message;
        let is_same = if ok {
            info!("⛔ LAMP [same message], {:?} ", &lamp_rgb);
            true
        } else {
            false
        };

        (is_locked, is_same)
    }

    fn forward_messages(&self, mut pub_stream: &mut TcpStream, locks : &mut Locks, object_message : Box<dyn DeviceMessage>) {

        let lamp_rgb = object_message.to_lamp_rgb();

        locks.kitchen_inter_dim_lock.inc();
        let inter_dim = InterDim {
            brightness: lamp_rgb.brightness,
            state: lamp_rgb.state.clone(),
        };
        KitchenInterDimDevice::receive(&mut pub_stream, inter_dim);

        //
        locks.hall_lamp_lock.inc();
        let lamp_rgb_hall = LampRGB {
            color: locks.hall_lamp_lock.last_object_message.color.clone(),
            brightness: lamp_rgb.brightness,
            state: lamp_rgb.state.clone(),
        };

        HallLampDevice::receive(&mut pub_stream,  lamp_rgb_hall);

        // locks.switch_locks += 1;
        // let message = format!("{{\"state\":\"{}\"}}", &lamp_rgb.state);
        // publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);

        // TODO : This is not at the correct place
        locks.kitchen_lamp_lock.replace(lamp_rgb.clone());

    }


    fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {
        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() {
                info!("Execute device LAMP");

                let object_message = self.read_object_message(msg);
                let rgb = object_message.to_lamp_rgb().clone(); // TEST ONLY
                match self.allowed_to_process(&mut locks, &object_message) {
                    (true, _) => {

                    }
                    (false, true) => {
                        info!("(same) REPLACE LAMP : {:?}", &rgb);
                        locks.kitchen_lamp_lock.replace(rgb.clone()); // TEST ONLY
                    }
                    (false, false) => {
                        info!("🍺 LAMP Here we are, {:?} ", &rgb);
                        info!("PROCESS LAMP ({}): {}", topic, msg);
                        self.forward_messages(&mut pub_stream, &mut locks, object_message);
                        info!("(no lock , no same) REPLACE LAMP : {:?}", &rgb);
                        locks.kitchen_lamp_lock.replace(rgb.clone()); // TEST ONLY
                    }
                }

            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
    }

}

