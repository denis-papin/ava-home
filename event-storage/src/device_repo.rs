use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;

use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

pub (crate) const TS_SALON_1 : &str = "ts_salon_1";
pub (crate) const TS_SALON_2 : &str = "ts_salon_2";

pub (crate) const TS_BUREAU : &str = "ts_bureau";

pub (crate) const TS_COULOIR : &str = "ts_couloir";

pub (crate) const TS_CHAMBRE_1 : &str = "ts_chambre_1";


pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice>>> = HashMap::new();
    let dev_list: Vec<GenericDevice> = vec![
        GenericDevice::new(TS_SALON_1, MessageEnum::default_temp_sensor()),
        GenericDevice::new(TS_SALON_2, MessageEnum::default_temp_sensor()),
        GenericDevice::new(TS_BUREAU, MessageEnum::default_temp_sensor()),
        GenericDevice::new(TS_CHAMBRE_1, MessageEnum::default_temp_sensor()),
        GenericDevice::new(TS_COULOIR, MessageEnum::default_temp_sensor()),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<Arc<RefCell<GenericDevice>>> {
    vec![
        device_repo.get(TS_SALON_1).unwrap().clone(),
        device_repo.get(TS_SALON_2).unwrap().clone(),
        device_repo.get(TS_BUREAU).unwrap().clone(),
        device_repo.get(TS_CHAMBRE_1).unwrap().clone(),
        device_repo.get(TS_COULOIR).unwrap().clone(),
    ]
}