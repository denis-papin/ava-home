
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;

use crate::{DeviceLock, DynDevice, InterSwitch, KitchenInterDimDevice, KitchenLampDevice, KitchenSwitchDevice, Locks, publish};
use crate::messages::{DeviceMessage, InterDim, LampRGB};

pub(crate) const HALL_LAMP : &str = "hall_lamp";

#[derive(Debug)]
pub(crate) struct HallLampDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
    pub setup : bool,
}

// TODO generalise the struct to handle all the "Lamp" family, pass the name in the constructor.
impl HallLampDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW HallLampDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl )),
            setup: false,
        }
    }
    pub fn get_name() -> &'static str {
        HALL_LAMP
    }
}

impl DynDevice for HallLampDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        self.setup = setup;
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }


    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        LampRGB::from_json(msg)
    }

    // TODO Generalize it
    fn init(&mut self, topic : &str, msg : &str, arc_locks: Arc<RefCell<Locks>>) {
        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() {
                info!("✨ Init device {} :",  &self.get_topic().to_uppercase());
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
                match r_info {
                    Ok(lamp) => {
                        self.setup = true;
                        locks.hall_lamp_lock.replace(lamp);
                    }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the device {}, {} :",  &self.get_topic().to_uppercase(), e);
                    }
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    // TODO same as from_json_to_local ?
    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

        match r_info {
            Ok(lamp) => { Box::new(lamp) }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(LampRGB::new())
            }
        }
    }

    fn allowed_to_process(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) -> (bool,bool) {
        let lamp_rgb = object_message.as_lamp_rgb();
        let is_locked = locks.hall_lamp_lock.count_locks > 0;
        let is_same = *lamp_rgb == locks.hall_lamp_lock.last_object_message;
        (is_locked, is_same)
    }


    // TODO check if still used ???
    fn replace(&self, locks: &mut Locks, object_message: &Box<dyn DeviceMessage>) {
        let rgb = object_message.as_lamp_rgb().clone();
        locks.hall_lamp_lock.replace(rgb );
    }

    // TODO check if still used ???
    fn get_last_object_message_as_string(&self, locks: &mut Locks) -> String {
        format!( "{:?}", locks.hall_lamp_lock.last_object_message )
    }

    // TODO check if still used ???
    fn get_last_object_message(&self, locks : &mut Locks) -> Box<dyn DeviceMessage> {
        Box::new ( locks.hall_lamp_lock.last_object_message.clone() )
    }

    // TODO check if still used ???
    fn lock(&self, locks: &mut Locks) {
        locks.hall_lamp_lock.inc();
    }

    // TODO check if still used ???
    fn unlock(&self, locks: &mut Locks) {
        locks.hall_lamp_lock.dec();
    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("HallLamp tries to build its LambRGB message");
        origin_message.to_lamp_rgb(last_message)
    }

}
