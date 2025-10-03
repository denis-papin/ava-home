use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;
use crate::device_repo::{RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON};
use crate::message_enum::MessageEnum;

pub (crate) fn build_init_list(_device_repo : &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    // Nothing to init
    vec![]
}

pub(crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<HardLoop<MessageEnum>> {
    vec![   HardLoop::new("loop_1".to_string(), vec![device_repo.get(RAD_SALON).unwrap().clone()]),
            HardLoop::new("loop_3".to_string(), vec![device_repo.get(RAD_BUREAU).unwrap().clone()]),
            HardLoop::new("loop_4".to_string(),vec![device_repo.get(RAD_CHAMBRE).unwrap().clone()]),
            HardLoop::new("loop_5".to_string(), vec![device_repo.get(RAD_COULOIR).unwrap().clone()]),
        ]
}
