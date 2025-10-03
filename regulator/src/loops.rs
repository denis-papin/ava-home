use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::device_repo::{RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON, REGULATE_RADIATOR};
use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;
use crate::message_enum::MessageEnum;

pub (crate) fn build_init_list(_device_repo : &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    // Nothing to init
    vec![]
}

pub(crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<HardLoop<MessageEnum>> {
    let loop_1 = HardLoop::new("loop_1".to_string(),
                               vec![device_repo.get(REGULATE_RADIATOR).unwrap().clone(),
                                    device_repo.get(RAD_SALON).unwrap().clone(),
                               ]);
    let loop_2 = HardLoop::new("loop_2".to_string(),
                               vec![device_repo.get(REGULATE_RADIATOR).unwrap().clone(),
                                    device_repo.get(RAD_BUREAU).unwrap().clone(),

                               ]);
    let loop_3 = HardLoop::new("loop_3".to_string(),
                               vec![device_repo.get(REGULATE_RADIATOR).unwrap().clone(),
                                    device_repo.get(RAD_COULOIR).unwrap().clone(),
                               ]);
    let loop_4 = HardLoop::new("loop_4".to_string(),
                               vec![device_repo.get(REGULATE_RADIATOR).unwrap().clone(),
                                    device_repo.get(RAD_CHAMBRE).unwrap().clone(),
                               ]);
    vec![loop_1, loop_2, loop_3, loop_4]
}

