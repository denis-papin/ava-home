use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use log::info;
use rumqttc::v5::AsyncClient;
use crate::generic_device::{GenericDevice, Locality};

#[derive(Clone)]
pub struct HardLoop<T : Locality> {
    pub name : String,
    pub devices : Vec<Arc<RefCell<GenericDevice<T>>>>,
}

impl <T> HardLoop<T> where T : Locality {
    pub fn new(name: String, devices : Vec<Arc<RefCell<GenericDevice<T>>>>) -> Self {
        Self {
            name,
            devices,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_devices(&self) -> Vec<Arc<RefCell<GenericDevice<T>>>> {
        self.devices.clone()
    }

    pub fn find_device_by_topic(&self, topic: &str) -> Option<Arc<RefCell<GenericDevice<T>>>> {
        for dev in self.get_devices() {
            let dd = dev.deref().borrow();
            if dd.get_topic() == topic {
                return Some(dev.clone());
            }
        }
        None
    }

    pub async fn loop_devices(&self, topic: &str, original_message: &T, mut client: &mut AsyncClient) {
        for dev in self.get_devices() {
            info!("Loop the devices");
            let dd1 = dev.as_ref().borrow();
            let dd = dd1.deref();
            if &dd.get_topic() != topic {
                info!("🚀 Device Topic of the loop: [{:?}]", &dd.get_topic());
                dd.consume_message(original_message, &mut client).await;
                info!("🚩 End Device Topic of the loop: [{:?}]", &dd.get_topic());
            }
        }
    }

    /// TODO This routine needs to manipulate some external data - it's only use in the regulator...
    pub async fn loop_devices_with_data(&self, topic: &str, original_message: &T, ext_data: &HashMap<String, f64>, mut client: &mut AsyncClient) {
        for dev in self.get_devices() {
            let ref_device = dev.as_ref().borrow();
            let device = ref_device.deref();
            info!("Loop the devices : [{}]", &device.get_topic());
            if &device.get_topic() != topic {
                info!("🚀 Device Topic of the loop: [{:?}]", &device.get_topic());
                device.consume_message(original_message, &ext_data, &mut client).await;
                info!("🚩 End Device Topic of the loop: [{:?}]", &device.get_topic());
            } else {
                 info!("Device ignored : [{}]", &device.get_topic());
            }
        }
    }

}