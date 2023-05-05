
use std::net::TcpStream;

use crate::{DynDevice, publish, InterSwitch};
use crate::messages::{DeviceMessage};

pub (crate) const KITCHEN_SWITCH : &str = "kitchen_switch";

#[derive(Debug)]
pub (crate) struct KitchenSwitchDevice {
    pub setup : bool,
}

impl KitchenSwitchDevice {
    pub(crate) fn _new() -> Self {
        Self {setup: false}
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


    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        InterSwitch::from_json(msg)
    }

    // fn init(&mut self, topic: &str, msg: &str, arc_locks: Arc<RefCell<Locks>>) {
    //     let locks = {
    //         let borr = arc_locks.as_ref().borrow();
    //         let mut locks = borr.deref().clone();
    //
    //         if topic == &self.get_topic() {
    //             info!("✨ Init device {} :",  &self.get_topic().to_uppercase());
    //             let r_info: Result<InterSwitch, _> = serde_json::from_str(msg);
    //             match r_info {
    //                 Ok(message) => {
    //                     self.setup = true;
    //                     locks.kitchen_switch_lock.replace(message);
    //                 }
    //                 Err(e) => {
    //                     panic!("💀 Cannot parse the message for the device {}, {} :",  &self.get_topic().to_uppercase(), e);
    //                 }
    //             }
    //         }
    //         locks
    //     };
    //     arc_locks.replace(locks.clone());
    // }


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
    // fn allowed_to_process(&self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) -> (bool, bool) {
    //     let message = object_message.as_inter_switch();
    //     let is_locked = locks.kitchen_switch_lock.count_locks > 0;
    //     let is_same = *message == locks.kitchen_switch_lock.last_object_message;
    //     (is_locked, is_same)
    // }

    // fn replace( &self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage> ) {
    //     let message = object_message.as_inter_switch().clone();
    //     locks.kitchen_switch_lock.replace(message);
    // }

    // fn get_last_object_message_as_string(&self, locks : &mut Locks) -> String {
    //     format!( "{:?}", locks.kitchen_switch_lock.last_object_message )
    // }
    //
    // fn lock(&self, locks : &mut Locks) {
    //     locks.kitchen_switch_lock.inc();;
    // }
    //
    // fn unlock(&self, locks : &mut Locks) {
    //     locks.kitchen_switch_lock.dec();;
    // }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"state":""}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, _last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_switch()
    }

    // fn get_last_object_message(&self, locks : &mut Locks) -> Box<dyn DeviceMessage> {
    //     Box::new ( locks.kitchen_switch_lock.last_object_message.clone() )
    // }
}
