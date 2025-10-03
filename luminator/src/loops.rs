use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;

use crate::device_repo::{KITCHEN_LAMP, MOVE_SENSOR_2, MOVE_SENSOR_3};
use crate::message_enum::MessageEnum;

pub(crate) const SENSOR_LIGHT: &str = "SENSOR_LIGHT";

pub (crate) fn build_init_list(device_repo : &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    vec![
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
    ]
}

pub(crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<HardLoop<MessageEnum>> {
    let sensor_light = HardLoop::new(SENSOR_LIGHT.to_string(),
                                     vec![
                                         device_repo.get(KITCHEN_LAMP).unwrap().clone(),
                                         device_repo.get(MOVE_SENSOR_2).unwrap().clone(),
                                         device_repo.get(MOVE_SENSOR_3).unwrap().clone(),
                                     ]);

    vec![sensor_light]
}

