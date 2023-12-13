use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use log::info;
use rumqttc::v5::AsyncClient;
use crate::device_repo::{HALL_LAMP, KITCHEN_LAMP, KITCHEN_SWITCH};

use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

pub (crate) const KITCHEN_LOOP : &str = "KITCHEN_LOOP";
pub (crate) const KITCHEN_LOOP_2 : &str = "KITCHEN_LOOP_2";

pub (crate) const TOO_HOT_LOOP : &str = "TOO_HOT_LOOP";
pub (crate) const SENSOR_LOOP : &str = "SENSOR_LOOP";

// fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Vec<Arc<RefCell<dyn DynDevice>>> {
//     vec![
//         device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
//         device_repo.get(KITCHEN_LAMP).unwrap().clone(),
//         device_repo.get(HALL_LAMP).unwrap().clone(),
//         // device_repo.get(TEMP_BAIE_VITREE).unwrap().clone(),
//         // device_repo.get(TEMP_MEUBLE_TV).unwrap().clone()
//         device_repo.get(KITCHEN_SWITCH).unwrap().clone(),
//     ]
// }

// pub (crate) fn find_loops(topic: &str, all_loops: &mut Vec<HardLoop>) -> (Vec<HardLoop>, Option<Arc<RefCell<dyn DynDevice>>>)  {
//     let mut eligible_loops : Vec<HardLoop> = vec![];
//     let mut output_dev : Option<Arc<RefCell<dyn DynDevice>>> = None;
//
//     for lp in all_loops {
//         match lp.find_device_by_topic(topic) {
//             None => {}
//             Some(dev) => {
//                 info!("Found topic in [{}] loop, topic=[{}]", & lp.get_name(), topic);
//                 eligible_loops.push(lp.clone());
//                 output_dev = Some(dev.clone());
//             }
//         }
//
//     }
//     (eligible_loops, output_dev)
// }

pub (crate) fn find_loops(topic: &str, all_loops: &mut Vec<HardLoop>) -> (Vec<HardLoop>, Option<Arc<RefCell<GenericDevice>>>)  {
    let mut eligible_loops : Vec<HardLoop> = vec![];
    let mut output_dev : Option<Arc<RefCell<GenericDevice>>> = None;

    for lp in all_loops {
        match lp.find_device_by_topic(topic) {
            None => {}
            Some(dev) => {
                info!("Found topic in [{}] loop, topic=[{}]", & lp.get_name(), topic);
                eligible_loops.push(lp.clone());
                output_dev = Some(dev.clone());
            }
        }

    }
    (eligible_loops, output_dev)
}

pub (crate) fn build_loops(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Vec<HardLoop> {

    // let kitchen_loop = HardLoop::new( KITCHEN_LOOP.to_string(),
    //                                   vec![
    //                                       device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
    //                                       device_repo.get(KITCHEN_LAMP).unwrap().clone(),
    //                                       device_repo.get(HALL_LAMP).unwrap().clone(),
    //                                   ]);

    let kitchen_loop_2 = HardLoop::new( KITCHEN_LOOP_2.to_string(),
                                      vec![
                                          device_repo.get(KITCHEN_SWITCH).unwrap().clone(),
                                          device_repo.get(KITCHEN_LAMP).unwrap().clone(),
                                          device_repo.get(HALL_LAMP).unwrap().clone(),
                                      ]);


    // let lamp_loop = HardLoop::new( KITCHEN_LOOP.to_string(),
    //                                vec![
    //                                    device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
    //                                    device_repo.get(KITCHEN_LAMP).unwrap().clone(),
    //                                    device_repo.get(HALL_LAMP).unwrap().clone(),
    //                                ]);
    //
    // let too_hot_loop = HardLoop::new( TOO_HOT_LOOP.to_string(),
    //                                   vec![
    //                                       device_repo.get(TEMP_BAIE_VITREE).unwrap().clone(),
    //                                   ]);
    //
    // let sensor_loop = HardLoop::new( SENSOR_LOOP.to_string(),
    //                                  vec![
    //                                      device_repo.get(TEMP_MEUBLE_TV).unwrap().clone(),
    //                                  ]);

    vec![/*kitchen_loop, */kitchen_loop_2/*, too_hot_loop, sensor_loop, lamp_loop*/]
}

#[derive(Clone)]
pub (crate) struct HardLoop {
    pub name : String,
    // pub devices : Vec<Arc<RefCell<dyn DynDevice>>>,
    pub devices : Vec<Arc<RefCell<GenericDevice>>>,
}

impl HardLoop {
    fn new(name: String, devices : Vec<Arc<RefCell<GenericDevice>>>) -> Self {
        Self {
            name,
            devices,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_devices(&self) -> Vec<Arc<RefCell<GenericDevice>>> {
        self.devices.clone()
    }

    pub fn find_device_by_topic(&self, topic: &str) -> Option<Arc<RefCell<GenericDevice>>> {
        for dev in self.get_devices() {
            let dd = dev.deref().borrow();
            if dd.get_topic() == topic {
                return Some(dev.clone());
            }
        }
        None
    }

    pub async fn loop_devices(&self, topic: &str, original_message: &MessageEnum, mut client: &mut AsyncClient) {
        for dev in self.get_devices() {
            info!("Loop the devices");
            let dd1 = dev.as_ref().borrow();
            let dd = dd1.deref();
            if &dd.get_topic() != topic {
                info!("🚀 Device Topic of the loop: [{:?}]", &dd.get_topic());
                dd.consume_message(original_message, &mut client).await;
                info!("🚩 End Device Topic of the loop: [{:?}]", &dd.get_topic());
            }
        }
    }

}
