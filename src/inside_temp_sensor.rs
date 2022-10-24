
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{DeviceMessage, DynDevice, HallLampDevice, LampRGB, Locks};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct TempSensor {
    pub battery : u32,
    pub humidity :f32,
    pub linkquality : u32,
    pub temperature: f32,
    pub voltage: u32,
}

pub (crate) const INSIDE_TEMP_SENSOR : &str = "inside_temp_sensor";
pub (crate) const OUTSIDE_TEMP_SENSOR : &str = "0x00124b002503888f";

#[derive(Debug)]
pub (crate) struct InsideTempSensorDevice {
}

impl InsideTempSensorDevice {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_name() -> &'static str {
        INSIDE_TEMP_SENSOR
    }
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

    fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {

        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() || topic == format!("zigbee2mqtt/{}", OUTSIDE_TEMP_SENSOR ) {
                let r_info: Result<TempSensor, _> = serde_json::from_str(msg);
                let message = match r_info {
                    Ok(lamp) => { lamp }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for inside_temp_sensor :  {e}");
                    }
                };

                info!("🍺 inside_temp_sensor message, {:?} ", &message);
                info!("PROCESS inside_temp_sensor ({}): {}", topic, msg);

                let mut lamp_rgb = locks.hall_lamp_lock.last_object_message.clone();
                locks.hall_lamp_lock.inc();
                info!("🔥 Temperature: {}", message.temperature);
                if message.temperature >= 22.0 {
                    lamp_rgb.state = "ON".to_string();
                } else {
                    lamp_rgb.state = "OFF".to_string();
                }
                HallLampDevice::receive(&mut pub_stream, lamp_rgb);
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    fn trigger_info(&self, _pub_stream: &mut TcpStream) {
        todo!()
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        todo!()
    }

    fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
        todo!()
    }

    fn forward_messages(&self, pub_stream: &mut TcpStream, locks: &mut Locks, object_message: Box<dyn DeviceMessage>) {
        todo!()
    }
}
