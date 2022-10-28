
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use serde_derive::*;
use outdoor_temp_sensor::OutdoorTempSensorDevice;
use crate::{DynDevice, HallLampDevice, Locks, outdoor_temp_sensor};
use crate::messages::{DeviceMessage, TempSensor};

pub (crate) const INSIDE_TEMP_SENSOR : &str = "inside_temp_sensor";

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

            if topic == &self.get_topic() || topic == OutdoorTempSensorDevice::new().get_topic() /*format!("zigbee2mqtt/{}", OUTSIDE_TEMP_SENSOR )*/ {
                let r_info: Result<TempSensor, _> = serde_json::from_str(msg);
                let message = match r_info {
                    Ok(lamp) => { lamp }
                    Err(e) => {
                        error!("💀 Cannot parse the message for inside_temp_sensor :  {e}");
                        return;
                    }
                };

                info!("🍺 inside_temp_sensor message, {:?} ", &message);
                info!("PROCESS inside_temp_sensor ({}): {}", topic, msg);

                info!("🔥 Temperature: {}", message.temperature);
                let lamp_rgb = if message.temperature >= 22.0 {
                    let mut lamp_red = locks.hall_lamp_lock.last_object_message.clone();
                    lamp_red.state = "ON".to_string();
                    lamp_red.color.x = 0.640625;
                    lamp_red.color.y = 0.328125;
                    lamp_red
                } else {
                    let mut lamp_green = locks.hall_lamp_lock.last_object_message.clone();
                    lamp_green.state = "OFF".to_string();
                    lamp_green.color.x = 0.3;
                    lamp_green.color.y = 0.6;
                    lamp_green
                };

                let mut lamp_off =  locks.hall_lamp_lock.last_object_message.clone();
                lamp_off.state = "OFF".to_string();

                for _ in 1..=3 {
                    locks.hall_lamp_lock.inc();
                    HallLampDevice::receive(&mut pub_stream, lamp_off.clone());
                    std::thread::sleep(Duration::from_millis(200));

                    locks.hall_lamp_lock.inc();
                    HallLampDevice::receive(&mut pub_stream, lamp_rgb.clone());
                    std::thread::sleep(Duration::from_millis(500));
                }

                locks.hall_lamp_lock.inc();
                HallLampDevice::receive(&mut pub_stream, lamp_off.clone());
                std::thread::sleep(Duration::from_millis(200));

                // Back to origin
                locks.hall_lamp_lock.inc();
                HallLampDevice::receive(&mut pub_stream,  locks.hall_lamp_lock.last_object_message.clone());
            }
            locks
        };
        arc_locks.replace(locks.clone());
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
}
