use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;

use crate::device_repo::{LAMP_CHAMBRE, SW_CHAMBRE};
use crate::message_enum::MessageEnum;

pub(crate) const BEDROOM_LIGHT: &str = "BEDROOM_LIGHT";

pub (crate) fn build_init_list(device_repo : &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    vec![
        device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
    ]
}

pub(crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<HardLoop<MessageEnum>> {
    let bedroom_light = HardLoop::new(BEDROOM_LIGHT.to_string(),
                                      vec![
                                         device_repo.get(SW_CHAMBRE).unwrap().clone(),
                                         device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
                                     ]);

    vec![bedroom_light]
}

