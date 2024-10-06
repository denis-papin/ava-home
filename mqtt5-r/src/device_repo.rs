use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

pub (crate) const KITCHEN_SWITCH : &str = "kitchen_switch";
pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";
pub(crate) const HALL_LAMP : &str = "hall_lamp";

pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice>>> = HashMap::new();
    let dev_list: Vec<GenericDevice> = vec![
        GenericDevice::new(KITCHEN_SWITCH, MessageEnum::default_inter_switch()),
        // GenericDevice::new(KITCHEN_INTER_DIM, MessageEnum::default_inter_dim()),
        GenericDevice::new(KITCHEN_LAMP, MessageEnum::default_lamp_rgb()),
        GenericDevice::new(HALL_LAMP, MessageEnum::default_lamp_rgb()),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    // device_repo.insert(TEMP_BAIE_VITREE.to_owned(), Arc::new(RefCell::new(InsideTempSensorDevice::new())));
    // device_repo.insert(TEMP_MEUBLE_TV.to_owned(), Arc::new(RefCell::new(OutdoorTempSensorDevice::new())));
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<Arc<RefCell<GenericDevice>>> {
    vec![
        // device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(HALL_LAMP).unwrap().clone(),
        // device_repo.get(TEMP_BAIE_VITREE).unwrap().clone(),
        // device_repo.get(TEMP_MEUBLE_TV).unwrap().clone()
        device_repo.get(KITCHEN_SWITCH).unwrap().clone(),
    ]
}