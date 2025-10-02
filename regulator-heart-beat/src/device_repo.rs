use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use ava_toolkit::generic_device::{GenericDevice, SYSTEM_FAMILY};
use crate::message_enum::MessageEnum;

pub (crate) const REGULATE_RADIATOR: &str = "regulate_radiator";

pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>> = HashMap::new();
    let dev_list: Vec<GenericDevice<MessageEnum>> = vec![
        GenericDevice::new(SYSTEM_FAMILY, REGULATE_RADIATOR, MessageEnum::default_regulation(), false),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

// pub (crate) fn device_to_listen(_device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
//     vec![
//     ]
// }