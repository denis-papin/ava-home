use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;

use crate::device_repo::{KITCHEN_LAMP, MOVE_SENSOR_2, MOVE_SENSOR_3};
use crate::message_enum::MessageEnum;

pub (crate) const SENSOR_LIGHT: &str = "SENSOR_LIGHT";


pub (crate) fn find_loops(topic: &str, all_loops: &mut Vec<HardLoop<MessageEnum>>) -> (Vec<HardLoop<MessageEnum>>, Option<Arc<RefCell<GenericDevice<MessageEnum>>>>)  {
    let mut eligible_loops : Vec<HardLoop<MessageEnum>> = vec![];
    let mut output_dev : Option<Arc<RefCell<GenericDevice<MessageEnum>>>> = None;

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

pub (crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<HardLoop<MessageEnum>> {

    let sensor_light = HardLoop::new(SENSOR_LIGHT.to_string(),
                                       vec![
                                          device_repo.get(KITCHEN_LAMP).unwrap().clone(),
                                          device_repo.get(MOVE_SENSOR_2).unwrap().clone(),
                                          device_repo.get(MOVE_SENSOR_3).unwrap().clone(),
                                      ]);

    vec![sensor_light]
}

