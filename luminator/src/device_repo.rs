use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use ava_toolkit::device_message::{LampRgbMsg, SimpleSwitchMsg};
use ava_toolkit::generic_device::{GenericDevice, ZIGBEE_FAMILY};
use crate::message_enum::MessageEnum;
use crate::message_enum::MessageEnum::{LampRgb, SimpleSwitch};

pub (crate) const LAMP_CHAMBRE : &str = "kitchen_lamp";
pub (crate) const SW_CHAMBRE : &str = "sw_chambre";

/// In the near future, we want to be able to define a pattern from a config file
/// Ex : switch, switch, lamp.
/// For that, we need to unmarshall a json file into a GenericDevis which has a specific MessageEnum
/// let lamp : GenericDevice<MessageEnum> = DeviceFactory.build(MessageEnum.LAMP, json)
pub (crate) fn build_device_repo() -> HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>> = HashMap::new();

    // 1)  Récupérer le Json de SimpleSwitch
    // let sw = r#"{"action":"single","battery":100,"linkquality":43}"#;
    let sw = r#"{"SimpleSwitch":{"action":"single","battery":100,"linkquality":43}}"#;
    let initial_object : MessageEnum =  serde_json::from_str(sw).unwrap();
    let  sw2= r#"{ "LampRgb" : {"brightness":147,"color":{"x":0.4185,"y":0.5054},"color_mode":"xy","color_temp":255,"linkquality":51,"state":"ON","update":{"state":"idle"}}}"#;
    let initial_object2 : MessageEnum =  serde_json::from_str(sw2).unwrap();
    
    let dev_list: Vec<GenericDevice<MessageEnum>> = vec![
        GenericDevice::new(ZIGBEE_FAMILY, SW_CHAMBRE, initial_object, true),
        GenericDevice::new(ZIGBEE_FAMILY, LAMP_CHAMBRE, initial_object2 , false),
    ];

    for dev in dev_list {
        device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
    }
    device_repo
}

pub (crate) fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Vec<Arc<RefCell<GenericDevice<MessageEnum>>>> {
    vec![
        device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
        device_repo.get(SW_CHAMBRE).unwrap().clone(),
    ]
}