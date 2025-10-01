use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use ava_toolkit::generic_device::{GenericDevice, ZIGBEE_FAMILY};
use crate::message_enum::MessageEnum;

pub (crate) const MOVE_SENSOR_2 : &str = "move_sensor_2";
pub (crate) const MOVE_SENSOR_3 : &str = "move_sensor_3";
pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";

/// In the near future, we want to be able to define a pattern from a config file
/// Ex : switch, switch, lamp.
/// For that, we need to unmarshall a json file into a GenericDevis which has a specific MessageEnum
/// let lamp : GenericDevice<MessageEnum> = DeviceFactory.build(MessageEnum.LAMP, json)
pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>> = HashMap::new();
    let dev_list: Vec<GenericDevice<MessageEnum>> = vec![
        GenericDevice::new(ZIGBEE_FAMILY, KITCHEN_LAMP, MessageEnum::default_lamp_rgb(), false),
        GenericDevice::new(ZIGBEE_FAMILY, MOVE_SENSOR_2, MessageEnum::default_move_sensor(), false),
        GenericDevice::new(ZIGBEE_FAMILY, MOVE_SENSOR_3, MessageEnum::default_move_sensor(), false),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    vec![
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(MOVE_SENSOR_2).unwrap().clone(),
        device_repo.get(MOVE_SENSOR_3).unwrap().clone(),
    ]
}