use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;

use crate::{DynDevice, Locks, publish, KitchenLampDevice, HallLampDevice, InterSwitch, KitchenSwitchDevice, DeviceLock};
use crate::messages::{DeviceMessage, InterDim, LampRGB};


pub (crate) const KITCHEN_INTER_DIM : &str = "kitchen_inter_dim";

#[derive(Debug)]
pub (crate) struct KitchenInterDimDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>
}

impl KitchenInterDimDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW KitchenInterDimDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl ))
        }
    }
    pub fn get_name() -> &'static str {
        KITCHEN_INTER_DIM
    }
}

impl DynDevice for KitchenInterDimDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        // Nothing to do
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        todo!()
    }

    fn init(&mut self, _topic: &str, _msg: &str, _arc_locks: Arc<RefCell<Locks>>) {
        todo!()
    }

    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        InterDim::from_json(msg)
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    fn replace(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
        let inter_dim = object_message.as_inter_dim().clone();
        locks.kitchen_inter_dim_lock.replace(inter_dim);
    }

    fn get_last_object_message_as_string(&self, locks: &mut Locks) -> String {
        format!( "{:?}", locks.kitchen_inter_dim_lock.last_object_message )
    }

    fn lock(&self, locks: &mut Locks) {
        locks.kitchen_inter_dim_lock.inc();
    }

    fn unlock(&self, locks: &mut Locks) {
        locks.kitchen_inter_dim_lock.dec();
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<InterDim, _> = serde_json::from_str(msg);

        match r_info {
            Ok(obj) => { Box::new(obj) }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(InterDim::new())
            }
        }
    }

    fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
        let local_message = object_message.as_inter_dim();
        let is_locked = locks.kitchen_inter_dim_lock.count_locks > 0;

        let old = &locks.kitchen_inter_dim_lock.last_object_message;
        dbg!(old);
        dbg!(&local_message);
        let is_same = *local_message == locks.kitchen_inter_dim_lock.last_object_message;
        dbg!(is_same);
        (is_locked, is_same)
    }

    // fn forward_messages(&self, pub_stream: &mut TcpStream, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
    //     todo!()
    // }

    fn get_last_object_message(&self, locks : &mut Locks) -> Box<dyn DeviceMessage> {
        Box::new ( locks.kitchen_inter_dim_lock.last_object_message.clone() )
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_dim()
    }
}
