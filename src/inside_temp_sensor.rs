
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use serde_derive::*;
use outdoor_temp_sensor::OutdoorTempSensorDevice;
use crate::{DynDevice, HallLampDevice, Locks, outdoor_temp_sensor};
use crate::messages::{DeviceMessage, TempSensor};

pub const TEMP_BAIE_VITREE : &str = "temp_baie_vitrée";

//// TEST

pub struct Dev1 {}
pub struct Dev2 {}

trait Device {
    fn process(&self) -> u16 {
        0
    }
}

trait Sensor: Device {
    fn process(&self) -> u16 {
        10
    }
}


impl Device for Dev1 {

}

impl Sensor for Dev1 {
}

impl Device for Dev2 {
    fn process(&self) -> u16 {
        0
    }
}

// TODO Maintenant il faut regarder comment et ou
//      les objets Devices sont utilisés pour avoir de vrais
//      singletons avec un vrai état unique et un lock.

// Boucle <- Liste de Device => "send message"

#[cfg(test)]
mod tests {
    use crate::inside_temp_sensor::{Dev1, Dev2, Device};

    #[test]
    fn test_traits() {
        let grid: Vec<Box<Device>> = vec![
            Box::new(Dev1 {}),
            Box::new(Dev2 {}),
        ];

        for d in grid {
            println!("{}", d.process());
        }
    }
}

///

#[derive(Debug)]
pub struct InsideTempSensorDevice {
}

impl InsideTempSensorDevice {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_name() -> &'static str {
        TEMP_BAIE_VITREE
    }
}

trait TempSensorDevice : DynDevice {
    fn process(&self, original_message : &Box<dyn DeviceMessage>) {
        // let last_message: Box<dyn DeviceMessage> = Box::new(TempSensor::new());
        // let object_message = self.to_local(&original_message, &last_message);
        info!("Process the message for the device: [{}]", self.get_topic());
        let temp_sensor_message = original_message.as_temp_sensor();
        dbg!(temp_sensor_message);
    }
}

impl TempSensorDevice for InsideTempSensorDevice {
}

impl DynDevice for InsideTempSensorDevice {

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
        TempSensor::from_json(msg)
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    fn replace(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
        // Nothing to do, no lock for sensors
    }

    fn get_last_object_message_as_string(&self, locks: &mut Locks) -> String {
        // Nothing to do, no last message
        "".to_string()
    }

    fn lock(&self, locks: &mut Locks) {
       // Nothing to do
    }

    fn unlock(&self, locks: &mut Locks) {
        // Nothing to do
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<TempSensor, _> = serde_json::from_str(msg);

        match r_info {
            Ok(temp_sensor_message) => {
                dbg!(&temp_sensor_message);
                Box::new(temp_sensor_message)
            }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(TempSensor::new())
            }
        }
    }

    fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
        (false, false)
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_temp_sensor(last_message)
    }

    // No last message for the device
    fn get_last_object_message(&self, locks : &mut Locks) -> Box<dyn DeviceMessage> {
        // Box::new ( locks.hall_lamp_lock.last_object_message.clone() )
        todo!()
    }

}
