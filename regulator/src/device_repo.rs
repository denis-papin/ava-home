use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use ava_toolkit::device_message::ZIGBEE_FAMILY;
use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

pub (crate) const REGULATE_RADIATOR: &str = "regulate_radiator";
pub (crate) const RAD_SALON: &str = "rad_salon";
pub (crate) const RAD_BUREAU: &str = "rad_bureau";
pub (crate) const RAD_COULOIR: &str = "rad_couloir";
pub (crate) const RAD_CHAMBRE: &str = "rad_chambre";

pub (crate) const LAMP_CHAMBRE : &str = "kitchen_lamp";
pub (crate) const SW_CHAMBRE : &str = "sw_chambre";

pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice>>> = HashMap::new();
    let dev_list: Vec<GenericDevice> = vec![
        GenericDevice::new("regulator", REGULATE_RADIATOR, MessageEnum::default_regulation_map()),
        GenericDevice::new("external", RAD_SALON, MessageEnum::default_radiator()),
        GenericDevice::new("external", RAD_BUREAU, MessageEnum::default_radiator()),
        GenericDevice::new("external", RAD_COULOIR, MessageEnum::default_radiator()),
        GenericDevice::new("external", RAD_CHAMBRE, MessageEnum::default_radiator()),
        GenericDevice::new(ZIGBEE_FAMILY, SW_CHAMBRE, MessageEnum::default_simple_switch()),
        GenericDevice::new(ZIGBEE_FAMILY, LAMP_CHAMBRE, MessageEnum::default_lamp_rgb()),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<Arc<RefCell<GenericDevice>>> {
    vec![
        device_repo.get(REGULATE_RADIATOR).unwrap().clone(),
        device_repo.get(RAD_SALON).unwrap().clone(),
        device_repo.get(RAD_BUREAU).unwrap().clone(),
        device_repo.get(RAD_COULOIR).unwrap().clone(),
        device_repo.get(RAD_CHAMBRE).unwrap().clone(),
        device_repo.get(SW_CHAMBRE).unwrap().clone(),
        device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
    ]
}