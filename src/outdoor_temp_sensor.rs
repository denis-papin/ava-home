

use std::net::TcpStream;


use crate::{DynDevice};
use crate::messages::{DeviceMessage, TempSensor};

pub (crate) const TEMP_MEUBLE_TV: &str = "temp_meuble_tv";

#[derive(Debug)]
pub (crate) struct OutdoorTempSensorDevice {
}

impl OutdoorTempSensorDevice {
    pub fn _new() -> Self {
        Self {}
    }

    pub fn get_name() -> &'static str {
        TEMP_MEUBLE_TV
    }
}

impl DynDevice for OutdoorTempSensorDevice {

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        todo!()
    }

    // fn init(&mut self, _topic: &str, _msg: &str, _arc_locks: Arc<RefCell<Locks>>) {
    //     todo!()
    // }

    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        TempSensor::from_json(msg)
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    // fn replace(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
    //     todo!()
    // }

    // fn get_last_object_message_as_string(&self, locks: &mut Locks) -> String {
    //     todo!()
    // }
    //
    // fn lock(&self, locks : &mut Locks) {
    //     //
    // }
    //
    // fn unlock(&self, locks: &mut Locks) {
    //     //
    // }

    fn read_object_message(&self, _msg: &str) -> Box<dyn DeviceMessage> {
        todo!()
    }

    // fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
    //     todo!()
    // }

    // fn forward_messages(&self, pub_stream: &mut TcpStream, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
    //     todo!()
    // }

    fn to_local(&self, _origin_message: &Box<dyn DeviceMessage>, _last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        todo!()
    }

    // No last message for the device
    // fn get_last_object_message(&self, locks : &mut Locks) -> Box<dyn DeviceMessage> {
    //     // Box::new ( locks.hall_lamp_lock.last_object_message.clone() )
    //     todo!()
    // }
}
