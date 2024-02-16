use std::collections::HashMap;

use log::info;

use crate::db_last_message::db_get_device_state;
use crate::device_message::{Radiator, RadiatorMode, RegulationMap};
use crate::external_computing::{compute, determine_action, RadiatorAction};
use crate::message_enum::MessageEnum::{RadiatorMsg, RegulationMsg};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    RegulationMsg(RegulationMap),
    RadiatorMsg(Radiator)
}

impl MessageEnum {

    pub (crate) async fn fetch_device_state(&self, topic: &str) -> Result<MessageEnum, String> {
        let json_msg = db_get_device_state(&topic).await;
        self.json_to_local(&json_msg)
    }

    pub (crate) fn find_set_topic(&self, topic: &str) -> String {
        match self {
            RegulationMsg(_) => {
                String::from("")
            }
            RadiatorMsg(_msg) => {
                // for some device, the set topic is "<topic>/set"
                String::from(topic)
            }
        }
    }


    pub (crate) fn query_for_state(&self) -> String {
        match self {
            RegulationMsg(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
            RadiatorMsg(_) => {
                // TODO : ???
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            RegulationMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            RadiatorMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            RegulationMsg(_) => {
                Ok(RegulationMsg(RegulationMap::from_json(json_msg)?))
            }
            RadiatorMsg(_) => {
                Ok(RadiatorMsg(Radiator::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_regulation_map() -> Self {
        RegulationMsg(RegulationMap::new())
    }

    pub (crate) fn default_radiator() -> Self {
        RadiatorMsg(Radiator::new())
    }

    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, ext_data: &HashMap<String, f64>, last_message: &MessageEnum, topic: &str) -> Self {
        match self {
            RegulationMsg(_) => {
                original_message.to_regulation_map(&last_message, &topic)
            }
            RadiatorMsg(_) => {
                original_message.to_radiator(&last_message, &ext_data, &topic)
            }
        }
    }

    /// Convert the current type of message to Temperature Sensor
    fn to_regulation_map(&self, last_message: &MessageEnum, topic: &str) -> Self {
        match self {
            RegulationMsg(_msg) => {
                self.clone()
            }
            RadiatorMsg(_msg) => {
                info!("Prepare the message to send for the device: [{}]", topic);
                dbg!(&last_message);
                dbg!(&self);
                last_message.clone()
            }
        }
        
    }

    /// Convert the current type of message to Radiator
    fn to_radiator(&self, last_message: &MessageEnum, ext_data: &HashMap<String, f64>, topic: &str) -> Self {
        match (self, last_message) {
            (RegulationMsg(msg), RadiatorMsg(last_rad)) => {

                info!("Prepare the message to send for the device: [{}]", topic);

                let action = match (topic, last_rad.mode) {
                    ("external/rad_bureau", _) if last_rad.mode != RadiatorMode::ECO => {
                        let t_current = *ext_data.get("bureau").unwrap();
                        info!("For device BUREAU, current [{}], target: [{}]", t_current, msg.tc_bureau);
                        determine_action(t_current, msg.tc_bureau)
                    }
                    ("external/rad_couloir", _) if last_rad.mode != RadiatorMode::ECO  => {
                        let t_current = *ext_data.get("couloir").unwrap();
                        info!("For device COULOIR, current [{}], target: [{}]", t_current, msg.tc_couloir);
                        determine_action(t_current, msg.tc_couloir)
                    },
                    ("external/rad_salon", _) if last_rad.mode != RadiatorMode::ECO => {
                        let t_current = *ext_data.get("salon_1").unwrap();
                        info!("For device SALON, current [{}], target: [{}]", t_current, msg.tc_salon_1);
                        determine_action(t_current, msg.tc_salon_1)
                    }
                    ("external/rad_chambre", _) if last_rad.mode != RadiatorMode::ECO => {
                        let t_current = *ext_data.get("chambre_1").unwrap();
                        info!("For device CHAMBRE, current [{}], target: [{}]", t_current, msg.tc_chambre_1);
                        determine_action(t_current, msg.tc_chambre_1)
                    }
                    (_, RadiatorMode::ECO) => RadiatorAction::NoAction,
                    (_, _) => RadiatorAction::NoAction,
                };

                info!("The action to perform is: [{:?}]", &action);

                match action {
                    RadiatorAction::On => {
                        info!("\t🔥 Radiator {} must be set to CFT", &topic);
                        RadiatorMsg(Radiator::from_mode(RadiatorMode::CFT))

                    }
                    RadiatorAction::Off => {
                        info!("\t❄️ Radiator {} must be set to STOP", &topic);
                        RadiatorMsg(Radiator::from_mode(RadiatorMode::STOP))
                    }
                    RadiatorAction::NoAction => {
                        info!("\tRadiator {} must stay the same", &topic);
                        last_message.clone()
                    }
                }
            }
            (RadiatorMsg(_), _) => {
                self.clone()
            }
            (_, _) => {
                self.clone()
            }
        }
    }

    /// Default process for the message
    pub (crate) async fn process(&self, _topic: &str, _args: &[String]) {
        match self {
            RegulationMsg(rm) => {
                info!("NOW EMPTY PROCESS - Default process for RegulationMap, message=[{:?}]", rm);
                // regulate_radiators(&topic, &rm, &args).await;
            }
            RadiatorMsg(msg) => {
                info!("NOW EMPTY PROCESS - Default process for Radiator, message=[{:?}]", msg);
            }
        }
    }

    pub (crate) async fn compute(&self) -> HashMap<String, f64> {
        match self {
            RegulationMsg(msg) => {
                info!("External computing for RegulationMap, message=[{:?}]", msg);
                compute().await
            }
            RadiatorMsg(msg) => {
                info!("External computing for Radiator, message=[{:?}]", msg);
                HashMap::new()
            }
        }
    }

}
