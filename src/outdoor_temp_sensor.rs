
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use serde_derive::*;
use crate::{DynDevice, HallLampDevice, Locks};
use crate::messages::{DeviceMessage, TempSensor};

pub (crate) const OUTSIDE_TEMP_SENSOR : &str = "outdoor_temp_sensor";

#[derive(Debug)]
pub (crate) struct OutdoorTempSensorDevice {
}

impl OutdoorTempSensorDevice {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_name() -> &'static str {
        OUTSIDE_TEMP_SENSOR
    }
}

impl DynDevice for OutdoorTempSensorDevice {

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        todo!()
    }

    fn init(&mut self, _topic: &str, _msg: &str, _arc_locks: Arc<RefCell<Locks>>) {
        todo!()
    }

    fn execute(&self, topic : &str, msg : &str, pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {
        // nothing to do
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

    fn to_local(&self, origin_message: &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        todo!()
    }
}
