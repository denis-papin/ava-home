// use std::cell::RefCell;
// use std::collections::HashMap;
// use std::fs;
// use std::sync::Arc;
//
// use log::info;
// use serde::de::DeserializeOwned;
// use serde_derive::Deserialize;
// use crate::generic_device::{GenericDevice, Locality};
// use crate::hard_loop::HardLoop;
//
// pub (crate) const LAMP_CHAMBRE : &str = "kitchen_lamp";
// pub (crate) const SW_CHAMBRE : &str = "sw_chambre";
//
//
// #[derive(Debug, Deserialize)]
// pub struct DeviceDefinition {
//     pub family: String,
//     pub name: String,
//     pub message_type: String,       // ex: "LampRgb" → loads "LampRgb.json"
//     pub process_same_message: bool,
// }
//
// #[derive(Debug, Deserialize)]
// pub struct LoopDefinition {
//     pub loop_name: String,
//     pub devices: Vec<String>,
// }
//
// /// Root configuration describing a module's setup.
// #[derive(Debug, Deserialize)]
// pub struct ConfigRoot {
//     pub devices: Vec<DeviceDefinition>,
//     pub loops: Vec<LoopDefinition>,
//     #[serde(default)]
//     pub devices_to_init: Vec<String>,
// }
//
//
// pub fn  factory<T>(message_type: &str) -> T where T : Locality + DeserializeOwned {
//     let path_to_json = format!(r"/home/denis/Projects/wks-ava-home/ava-home/ava-toolkit/resources/{}.json", message_type);
//     let  object_json= fs::read_to_string(path_to_json).unwrap();
//     let message : T =  serde_json::from_str(object_json.as_str()).unwrap(); // TODO
//     message
// }
//
// /// In the near future, we want to be able to define a pattern from a config file
// /// Ex : switch, switch, lamp.
// /// For that, we need to unmarshall a json file into a GenericDevice which has a specific MessageEnum
// /// let lamp : GenericDevice<MessageEnum> = DeviceFactory.build(MessageEnum.LAMP, json)
// pub fn build_device_repo_gen<T>() -> HashMap<String, Arc<RefCell<GenericDevice<T>>>> where T : Locality + DeserializeOwned {
//     info!("Inside the Repo Builder");
//
//     let ALL = vec![DeviceDefinition {
//         family: "zigbee2mqtt".to_string(),
//         name: SW_CHAMBRE.to_string(),
//         message_type: "SimpleSwitch".to_string(),
//         process_same_message: false,
//     },DeviceDefinition {
//         family: "zigbee2mqtt".to_string(),
//         name: LAMP_CHAMBRE.to_string(),
//         message_type: "LampRgb".to_string(),
//         process_same_message: false,
//     }];
//
//     let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice<T>>>> = HashMap::new();
//     let dev_list: Vec<GenericDevice<T>> = ALL.iter().map(|def| GenericDevice::new(&def.family, &def.name, factory(&def.message_type), def.process_same_message) ).collect();
//
//     for dev in dev_list {
//         device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
//     }
//     device_repo
// }
//
// pub fn device_to_listen_gen<T>(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<T>>>>) -> Vec<Arc<RefCell<GenericDevice<T>>>> where T : Locality {
//     vec![
//         device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
//         device_repo.get(SW_CHAMBRE).unwrap().clone(),
//     ]
// }
//
// pub(crate) const BEDROOM_LIGHT: &str = "BEDROOM_LIGHT";
//
// pub (crate) fn build_init_list<T>(device_repo : &HashMap<String, Arc<RefCell<GenericDevice<T>>>>) -> Vec<Arc<RefCell<GenericDevice<T>>>>
// where T : Locality{
//     vec![
//         device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
//     ]
// }
//
// pub(crate) fn build_loops<T>(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<T>>>>) -> Vec<HardLoop<T>>
// where T : Locality  + DeserializeOwned {
//     let bedroom_light = HardLoop::new(BEDROOM_LIGHT.to_string(),
//                                       vec![
//                                           device_repo.get(SW_CHAMBRE).unwrap().clone(),
//                                           device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
//                                       ]);
//
//     vec![bedroom_light]
// }
// use std::cell::RefCell;
// use std::collections::HashMap;
// use std::fs;
// use std::sync::Arc;
//
// use log::info;
// use serde::de::DeserializeOwned;
// use serde_derive::Deserialize;
// use crate::generic_device::{GenericDevice, Locality};
// use crate::hard_loop::HardLoop;
//
// pub (crate) const LAMP_CHAMBRE : &str = "kitchen_lamp";
// pub (crate) const SW_CHAMBRE : &str = "sw_chambre";
//
//
// #[derive(Debug, Deserialize)]
// pub struct DeviceDefinition {
//     pub family: String,
//     pub name: String,
//     pub message_type: String,       // ex: "LampRgb" → loads "LampRgb.json"
//     pub process_same_message: bool,
// }
//
// #[derive(Debug, Deserialize)]
// pub struct LoopDefinition {
//     pub loop_name: String,
//     pub devices: Vec<String>,
// }
//
// /// Root configuration describing a module's setup.
// #[derive(Debug, Deserialize)]
// pub struct ConfigRoot {
//     pub devices: Vec<DeviceDefinition>,
//     pub loops: Vec<LoopDefinition>,
//     #[serde(default)]
//     pub devices_to_init: Vec<String>,
// }
//
//

//
// /// In the near future, we want to be able to define a pattern from a config file
// /// Ex : switch, switch, lamp.
// /// For that, we need to unmarshall a json file into a GenericDevice which has a specific MessageEnum
// /// let lamp : GenericDevice<MessageEnum> = DeviceFactory.build(MessageEnum.LAMP, json)
// pub fn build_device_repo_gen<T>() -> HashMap<String, Arc<RefCell<GenericDevice<T>>>> where T : Locality + DeserializeOwned {
//     info!("Inside the Repo Builder");
//
//     let ALL = vec![DeviceDefinition {
//         family: "zigbee2mqtt".to_string(),
//         name: SW_CHAMBRE.to_string(),
//         message_type: "SimpleSwitch".to_string(),
//         process_same_message: false,
//     },DeviceDefinition {
//         family: "zigbee2mqtt".to_string(),
//         name: LAMP_CHAMBRE.to_string(),
//         message_type: "LampRgb".to_string(),
//         process_same_message: false,
//     }];
//
//     let mut device_repo : HashMap<String, Arc<RefCell<GenericDevice<T>>>> = HashMap::new();
//     let dev_list: Vec<GenericDevice<T>> = ALL.iter().map(|def| GenericDevice::new(&def.family, &def.name, factory(&def.message_type), def.process_same_message) ).collect();
//
//     for dev in dev_list {
//         device_repo.insert( dev.name.to_owned(), Arc::new(RefCell::new(dev)));
//     }
//     device_repo
// }
//
// pub fn device_to_listen_gen<T>(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<T>>>>) -> Vec<Arc<RefCell<GenericDevice<T>>>> where T : Locality {
//     vec![
//         device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
//         device_repo.get(SW_CHAMBRE).unwrap().clone(),
//     ]
// }
//
// pub(crate) const BEDROOM_LIGHT: &str = "BEDROOM_LIGHT";
//
// pub (crate) fn build_init_list<T>(device_repo : &HashMap<String, Arc<RefCell<GenericDevice<T>>>>) -> Vec<Arc<RefCell<GenericDevice<T>>>>
// where T : Locality{
//     vec![
//         device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
//     ]
// }
//
// pub(crate) fn build_loops<T>(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<T>>>>) -> Vec<HardLoop<T>>
// where T : Locality  + DeserializeOwned {
//     let bedroom_light = HardLoop::new(BEDROOM_LIGHT.to_string(),
//                                       vec![
//                                           device_repo.get(SW_CHAMBRE).unwrap().clone(),
//                                           device_repo.get(LAMP_CHAMBRE).unwrap().clone(),
//                                       ]);
//
//     vec![bedroom_light]
// }