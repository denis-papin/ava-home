use std::cell::{RefCell};
use std::net::TcpStream;

use std::sync::Arc;
use crate::{DynDevice, DeviceLock};
use crate::messages::{DeviceMessage, InterDim};

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

    fn setup(&mut self, _setup: bool) {
        // Nothing to do
    }
    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        todo!()
    }

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( InterDim::from_json(msg)? ))
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, _last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_inter_dim()
    }
}
