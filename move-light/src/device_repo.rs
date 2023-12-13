use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

pub (crate) const MOVE_SENSOR_2 : &str = "move_sensor_2";
pub (crate) const MOVE_SENSOR_3 : &str = "move_sensor_3";
pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";

pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice>>> = HashMap::new();
    let dev_list: Vec<GenericDevice> = vec![
        GenericDevice::new(KITCHEN_LAMP, MessageEnum::default_lamp_rgb()),
        GenericDevice::new(MOVE_SENSOR_2, MessageEnum::default_move_sensor()),
        GenericDevice::new(MOVE_SENSOR_3, MessageEnum::default_move_sensor()),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<Arc<RefCell<GenericDevice>>> {
    vec![
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(MOVE_SENSOR_2).unwrap().clone(),
        device_repo.get(MOVE_SENSOR_3).unwrap().clone(),
    ]
}