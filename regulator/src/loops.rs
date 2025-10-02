use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use crate::device_repo::{RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON, REGULATE_RADIATOR};
use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;
use crate::message_enum::MessageEnum;

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

