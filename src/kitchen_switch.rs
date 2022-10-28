use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;

use crate::{DynDevice, Locks, publish, KitchenLampDevice, HallLampDevice, InterSwitch, KitchenInterDimDevice};
use crate::messages::{DeviceMessage, InterDim, LampRGB};

pub (crate) const KITCHEN_SWITCH : &str = "kitchen_switch";

#[derive(Debug)]
pub (crate) struct KitchenSwitchDevice {
    pub setup : bool,
}

impl KitchenSwitchDevice {
    pub(crate) fn new() -> Self {
        Self {setup: false}
    }

    pub fn receive(mut pub_stream: &mut TcpStream, object_message: InterSwitch ) {
        match serde_json::to_string(&object_message) {
            Ok(message) => {
                info!("➡ Prepare to be sent to the {}, {:?} ", Self::get_name(), &message);
                publish(&mut pub_stream, &format!("zigbee2mqtt/{}/set", Self::get_name()), &message);
            }
            Err(_) => {
                error!("💣 Impossible to parse the message :{:?}", &object_message);
            }
        }
    }

    pub fn get_name() -> &'static str {
        KITCHEN_SWITCH
    }
}

impl DynDevice for KitchenSwitchDevice {

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn init(&mut self, topic: &str, msg: &str, arc_locks: Arc<RefCell<Locks>>) {
        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() {
                info!("✨ Init device {} :",  &self.get_topic().to_uppercase());
                let r_info: Result<InterSwitch, _> = serde_json::from_str(msg);
                match r_info {
                    Ok(message) => {
                        self.setup = true;
                        locks.kitchen_switch_lock.replace(message);
                    }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the device {}, {} :",  &self.get_topic().to_uppercase(), e);
                    }
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }


    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<InterSwitch, _> = serde_json::from_str(msg);

        match r_info {
            Ok(lamp) => { Box::new(lamp) }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(InterSwitch::new())
            }
        }
    }

    //
    fn allowed_to_process(&self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) -> (bool, bool) {
        let message = object_message.to_inter_switch();
        let is_locked = locks.kitchen_switch_lock.count_locks > 0;
        let is_same = *message == locks.kitchen_switch_lock.last_object_message;
        (is_locked, is_same)
    }

    fn forward_messages(&self, mut pub_stream: &mut TcpStream, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) {
        let message = object_message.to_inter_switch();

        locks.kitchen_inter_dim_lock.inc();
        let inter_dim = InterDim {
            brightness: locks.hall_lamp_lock.last_object_message.brightness, // TODO keep the full state of the dimmer : locks.kitchen_inter_dim_lock.last_object_message.brightness,
            state: message.state.clone(),
        };
        KitchenInterDimDevice::receive(&mut pub_stream, inter_dim);

        //
        locks.hall_lamp_lock.inc();
        let lamp_rgb_hall = LampRGB {
            color: locks.hall_lamp_lock.last_object_message.color.clone(),
            brightness: locks.hall_lamp_lock.last_object_message.brightness,
            state: message.state.clone(),
        };

        HallLampDevice::receive(&mut pub_stream,  lamp_rgb_hall);

        //
        locks.kitchen_lamp_lock.inc();
        let lamp_rgb_kitchen = LampRGB {
            color: locks.kitchen_lamp_lock.last_object_message.color.clone(),
            brightness: locks.kitchen_lamp_lock.last_object_message.brightness,
            state: message.state.clone(),
        };

        KitchenLampDevice::receive(&mut pub_stream,  lamp_rgb_kitchen);

    }

    fn replace( &self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage> ) {
        let message = object_message.to_inter_switch().clone();
        locks.kitchen_switch_lock.replace(message);
    }

    fn get_last_object_message(&self, locks : &mut Locks) -> String {
        format!( "{:?}", locks.kitchen_switch_lock.last_object_message )
    }

    fn unlock(&self, locks : &mut Locks) {
        locks.kitchen_switch_lock.dec();;
    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"state":""}"#);
    }
}
