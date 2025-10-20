use std::collections::HashMap;

use crate::external_computing::{compute, determine_action, RadiatorAction};
use crate::message_enum::MessageEnum::{RegulationMap, RegulatorRadiator};
use ava_toolkit::device_message::{RadiatorMode, RegulationMapMsg, RegulatorRadiatorMsg};
use ava_toolkit::generic_device::Locality;
use log::info;
use serde_derive::{Deserialize, Serialize};

/// Object by enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub (crate) enum MessageEnum {
    RegulationMap(RegulationMapMsg),
    RegulatorRadiator(RegulatorRadiatorMsg)
}

impl MessageEnum {
    
    pub (crate) fn default_regulation_map() -> Self {
        RegulationMap(RegulationMapMsg::new())
    }

    pub (crate) fn default_radiator() -> Self {
        RegulatorRadiator(RegulatorRadiatorMsg::new())
    }


    /// Convert the current type of message to Temperature Sensor
    fn to_regulation_map(&self, last_message: &MessageEnum) -> Self {
        last_message.clone()
    }

    /// Convert the current type of message to Radiator
    fn to_radiator(&self, last_message: &MessageEnum, o_ext_data: Option<&HashMap<String, f64>>, o_topic: Option<&str>) -> Self {
        let topic = o_topic.unwrap_or("");

        // TODO handle the error of missing data instead
        let empty = HashMap::new();
        let ext_data = o_ext_data.unwrap_or(&empty);
        
        match (self, last_message) {
            (RegulationMap(msg), RegulatorRadiator(last_rad)) => {

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
                        RegulatorRadiator(RegulatorRadiatorMsg::from_mode(RadiatorMode::CFT))

                    }
                    RadiatorAction::Off => {
                        info!("\t❄️ Radiator {} must be set to STOP", &topic);
                        RegulatorRadiator(RegulatorRadiatorMsg::from_mode(RadiatorMode::STOP))
                    }
                    RadiatorAction::NoAction => {
                        info!("\tRadiator {} must stay the same", &topic);
                        last_message.clone()
                    }
                }
            }
            (RegulatorRadiator(_), _) => {
                self.clone()
            }
            (_, _) => {
                self.clone()
            }
        }
    }
    
}


impl Locality for MessageEnum {
    fn query_for_state(&self) -> String {
        match self {
            RegulatorRadiator(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
            RegulationMap(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    fn find_set_topic(&self, topic: &str) -> String {
        topic.to_string()
    }
    
    fn raw_message(&self) -> String {
        match self {
            RegulationMap(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            RegulatorRadiator(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }

    /// Convert the original message to the type of the current Self
    fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        self.to_local_with_data(original_message, last_message, None, None)
    }

    /// Convert the original message to the type of the current Self
    fn to_local_with_data(&self, original_message: &MessageEnum, last_message: &MessageEnum, ext_data: Option<&HashMap<String, f64>>, topic: Option<&str>) -> Self {
        match self {
            RegulationMap(_) => {
                original_message.to_regulation_map(&last_message)
            }
            RegulatorRadiator(_) => {
                original_message.to_radiator(&last_message, ext_data, topic)
            }
        }
    }

    fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            RegulatorRadiator(_) => {
                Ok(RegulatorRadiator(RegulatorRadiatorMsg::from_json(json_msg)?))
            }
            RegulationMap(_) => {
                Ok(RegulationMap(RegulationMapMsg::from_json(json_msg)?))
            }
        }
    }

    /// Non disponible
    async fn process(&self, _topic: &str, _args: &[String]) {
        match self {
            RegulationMap(rm) => {
                info!("NOW EMPTY PROCESS - Default process for RegulationMap, message=[{:?}]", rm);
            }
            RegulatorRadiator(msg) => {
                info!("NOW EMPTY PROCESS - Default process for Radiator, message=[{:?}]", msg);
            }
        }
    }

    async fn compute(&self) -> Option<HashMap<String, f64>> {
        match self {
            RegulationMap(msg) => {
                info!("External computing for RegulationMap, message=[{:?}]", msg);
                Some(compute().await)
            }
            RegulatorRadiator(msg) => {
                info!("External computing for Radiator, message=[{:?}]", msg);
                None
            }
        }
    }
    
}