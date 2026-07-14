use std::cell::RefCell;
use std::collections::HashMap;

use std::sync::Arc;

use crate::device_repo::{HALL_LAMP, KITCHEN_LAMP, KITCHEN_SWITCH};

use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;
use crate::message_enum::MessageEnum;

pub (crate) const KITCHEN_LOOP_2 : &str = "KITCHEN_LOOP_2";

pub (crate) fn build_init_list(device_repo : &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    let values = device_repo.values();
    let mut v = vec![];
    for a in values {
        v.push(a.clone());
    }
    v
}

pub (crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<HardLoop<MessageEnum>> {
    
    let kitchen_loop_2 = HardLoop::new( KITCHEN_LOOP_2.to_string(),
                                      vec![
                                          device_repo.get(KITCHEN_SWITCH).unwrap().clone(),
                                          device_repo.get(KITCHEN_LAMP).unwrap().clone(),
                                          device_repo.get(HALL_LAMP).unwrap().clone(),
                                      ]);

    vec![/*kitchen_loop, */kitchen_loop_2/*, too_hot_loop, sensor_loop, lamp_loop*/]
}
