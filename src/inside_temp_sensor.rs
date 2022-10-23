
///-------------
use std::cell::{Cell, RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{INSIDE_TEMP_SENSOR, LampRGB, Locks, OUTSIDE_TEMP_SENSOR, publish};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub (crate) struct TempSensor {
    pub battery : u32,
    pub humidity :f32,
    pub linkquality : u32,
    pub temperature: f32,
    pub voltage: u32,
}

#[derive(Debug)]
pub (crate) struct InsideTempSensorDevice {
}

impl InsideTempSensorDevice {
    pub fn new() -> Self {
        Self { }
    }

    pub (crate) fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>) {

        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == format!("zigbee2mqtt/{}", INSIDE_TEMP_SENSOR ) || topic == format!("zigbee2mqtt/{}", OUTSIDE_TEMP_SENSOR ) {
                let r_info: Result<TempSensor, _> = serde_json::from_str(msg);
                let message = match r_info {
                    Ok(lamp) => { lamp }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for inside_temp_sensor :  {e}");
                    }
                };

                info!("🍺 inside_temp_sensor message, {:?} ", &message);
                info!("PROCESS inside_temp_sensor ({}): {}", topic, msg);

                let mut lamp_rgb = locks.last_hall_lamp.clone();
                if message.temperature >= 22.0 {
                    //locks.hall_lamp_locks += 1;
                } else {
                    lamp_rgb.state = "OFF".to_string();
                    let message = serde_json::to_string(&lamp_rgb).unwrap();
                    info!("➡ Prepare to be sent to the hall lamp, {:?} ", &message);
                    publish(&mut pub_stream, "zigbee2mqtt/hall_lamp/set", &message);
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }
}
