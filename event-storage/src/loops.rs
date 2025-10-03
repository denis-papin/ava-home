use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;

use crate::device_repo::{MOVE_SENSOR_BUREAU, MOVE_SENSOR_SALON, RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON, SW_CHAMBRE, TS_BUREAU, TS_CHAMBRE_1, TS_COULOIR, TS_SALON_1, TS_SALON_2};
use crate::message_enum::MessageEnum;

pub (crate) fn build_init_list(_device_repo : &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
     // Nothing to init
     vec![]
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

