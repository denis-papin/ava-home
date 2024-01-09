use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use log::info;
use rumqttc::v5::AsyncClient;

use crate::device_repo::{TS_BUREAU, TS_CHAMBRE_1, TS_COULOIR, TS_SALON_1, TS_SALON_2};
use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

// pub (crate) const TS_SALON_1_LOOP : &str = "TS_SALON_1_LOOP";
// pub (crate) const TS_SALON_2_LOOP : &str = "TS_SALON_2_LOOP";

pub (crate) fn find_loops(topic: &str, all_loops: &mut Vec<HardLoop>) -> (Vec<HardLoop>, Option<Arc<RefCell<GenericDevice>>>)  {
    let mut eligible_loops : Vec<HardLoop> = vec![];
    let mut output_dev : Option<Arc<RefCell<GenericDevice>>> = None;

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

pub(crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<HardLoop> {
    let loop_1 = HardLoop::new("loop_1".to_string(), vec![device_repo.get(TS_SALON_1).unwrap().clone()]);
    let loop_2 = HardLoop::new("loop_2".to_string(),vec![device_repo.get(TS_SALON_2).unwrap().clone()]);
    let loop_3 = HardLoop::new("loop_3".to_string(), vec![device_repo.get(TS_BUREAU).unwrap().clone()]);
    let loop_4 = HardLoop::new("loop_4".to_string(),vec![device_repo.get(TS_CHAMBRE_1).unwrap().clone()]);
    let loop_5 = HardLoop::new("loop_5".to_string(), vec![device_repo.get(TS_COULOIR).unwrap().clone()]);

    vec![loop_1, loop_2, loop_3, loop_4, loop_5]
}

#[derive(Clone)]
pub (crate) struct HardLoop {
    pub name : String,
    // pub devices : Vec<Arc<RefCell<dyn DynDevice>>>,
    pub devices : Vec<Arc<RefCell<GenericDevice>>>,
}

impl HardLoop {
    fn new(name: String, devices : Vec<Arc<RefCell<GenericDevice>>>) -> Self {
        Self {
            name,
            devices,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_devices(&self) -> Vec<Arc<RefCell<GenericDevice>>> {
        self.devices.clone()
    }

    pub fn find_device_by_topic(&self, topic: &str) -> Option<Arc<RefCell<GenericDevice>>> {
        for dev in self.get_devices() {
            let dd = dev.deref().borrow();
            if dd.get_topic() == topic {
                return Some(dev.clone());
            }
        }
        None
    }

    pub async fn loop_devices(&self, topic: &str, original_message: &MessageEnum, mut client: &mut AsyncClient) {
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

}
