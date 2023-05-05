use std::cell::RefCell;
use std::net::TcpStream;
use std::sync::Arc;
use crate::{DeviceLock, DynDevice};
use crate::messages::{DeviceMessage, TempSensor};

pub const TEMP_BAIE_VITREE : &str = "temp_baie_vitrée";

// #[derive(Debug)]
// pub struct TSensorDevice {
//     pub name: String
// }
//
// impl TSensorDevice {
//     pub fn new(name: &str) -> Self {
//         Self {name: name.to_string()}
//     }
//
//     pub fn get_name() -> String {
//         self.name
//     }
// }

#[derive(Debug)]
pub(crate) struct InsideTempSensorDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
}

impl InsideTempSensorDevice {
    pub fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW InsideTempSensorDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl ))
        }
    }

    pub fn get_name() -> &'static str {
        TEMP_BAIE_VITREE
    }
}

trait TempSensorDevice : DynDevice {
    fn process(&self, original_message : &Box<dyn DeviceMessage>) {
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

    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        TempSensor::from_json(msg)
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
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


    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        origin_message.to_temp_sensor(last_message)
    }

}
