use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

pub (crate) const REGULATE_RADIATOR: &str = "regulate_radiator";

pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice>>> = HashMap::new();
    let dev_list: Vec<GenericDevice> = vec![
        GenericDevice::new(REGULATE_RADIATOR, MessageEnum::default_regulation_map()),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<Arc<RefCell<GenericDevice>>> {
    vec![
        device_repo.get(REGULATE_RADIATOR).unwrap().clone(),
    ]
}