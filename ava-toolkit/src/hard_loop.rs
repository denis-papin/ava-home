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

    // static
    pub fn find_loops(topic: &str, all_loops: &Vec<HardLoop<T>>) -> (Vec<HardLoop<T>>, Option<Arc<RefCell<GenericDevice<T>>>>)  {
        let mut eligible_loops : Vec<HardLoop<T>> = vec![];
        let mut output_dev : Option<Arc<RefCell<GenericDevice<T>>>> = None;

        for lp in all_loops {
            match lp.find_device_by_topic(topic) {
                None => {}
                Some(dev) => {
                    info!("Found topic in [{}] loop, topic=[{}]", & lp.get_name(), topic);
                    eligible_loops.push(lp.clone());
                    output_dev = Some(dev.clone());
                }
            }

        }
        (eligible_loops, output_dev)
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

    /// This routine may manipulate some external data, like in the regulator project.
    pub async fn loop_devices(&self, topic: &str, original_message: &T, o_ext_data: Option<&HashMap<String, f64>>, mut client: &mut AsyncClient) {
        let devices = self.get_devices();
        for dev in devices.iter() {
            let ref_device = dev.as_ref().borrow();
            let device = ref_device.deref();
            info!("Loop the devices : [{}], for the current topic [{}]", &device.get_topic(), topic);
            if &device.get_topic() != topic {
                info!("🚀 Device Topic of the loop: [{:?}]", &device.get_topic());
                device.consume_message(original_message, o_ext_data, &mut client).await;
                info!("🚩 End Device Topic of the loop: [{:?}]", &device.get_topic());
            } else {
                 info!("Device ignored : [{}]", &device.get_topic());
            }
        }
    }

}