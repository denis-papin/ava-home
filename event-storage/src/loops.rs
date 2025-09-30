use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;

use crate::device_repo::{MOVE_SENSOR_BUREAU, MOVE_SENSOR_SALON, RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON, SW_CHAMBRE, TS_BUREAU, TS_CHAMBRE_1, TS_COULOIR, TS_SALON_1, TS_SALON_2};
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
    vec![HardLoop::new("loop_1".to_string(), vec![device_repo.get(TS_SALON_1).unwrap().clone()]),
         HardLoop::new("loop_2".to_string(), vec![device_repo.get(TS_SALON_2).unwrap().clone()]),
         HardLoop::new("loop_3".to_string(), vec![device_repo.get(TS_BUREAU).unwrap().clone()]),
         HardLoop::new("loop_4".to_string(), vec![device_repo.get(TS_CHAMBRE_1).unwrap().clone()]),
         HardLoop::new("loop_5".to_string(), vec![device_repo.get(TS_COULOIR).unwrap().clone()]),

         HardLoop::new("loop_sw_chambre".to_string(), vec![device_repo.get(SW_CHAMBRE).unwrap().clone()]),

         HardLoop::new("loop_sensor_bureau".to_string(), vec![device_repo.get(MOVE_SENSOR_BUREAU).unwrap().clone()]),
         HardLoop::new("loop_sensor_salon".to_string(), vec![device_repo.get(MOVE_SENSOR_SALON).unwrap().clone()]),

         HardLoop::new("loop_6".to_string(), vec![device_repo.get(RAD_SALON).unwrap().clone()]),
         HardLoop::new("loop_7".to_string(), vec![device_repo.get(RAD_BUREAU).unwrap().clone()]),
         HardLoop::new("loop_8".to_string(), vec![device_repo.get(RAD_COULOIR).unwrap().clone()]),
         HardLoop::new("loop_9".to_string(), vec![device_repo.get(RAD_CHAMBRE).unwrap().clone()]),
    ]
}

