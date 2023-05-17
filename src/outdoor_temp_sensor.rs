use std::cell::RefCell;
use std::net::TcpStream;
use std::sync::Arc;

use crate::{DeviceLock, DynDevice};
use crate::messages::{DeviceMessage, TempSensor};

pub (crate) const TEMP_MEUBLE_TV: &str = "temp_meuble_tv";

#[derive(Debug)]
pub (crate) struct OutdoorTempSensorDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
}

/// Temperature sensor
impl OutdoorTempSensorDevice {
    pub fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW OutdoorTempSensorDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl ))
        }
    }

    pub fn get_name() -> &'static str {
        TEMP_MEUBLE_TV
    }
}

impl DynDevice for OutdoorTempSensorDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        todo!()
    }

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String> {
        Ok(Box::new( TempSensor::from_json(msg)? ))
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    fn to_local(&self, origin_message: &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_temp_sensor(last_message)
    }

}
