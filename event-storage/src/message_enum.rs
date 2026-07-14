use std::collections::HashMap;
use log::info;
use serde_derive::{Deserialize, Serialize};

use ava_toolkit::device_message::{RegulatorRadiatorMsg, TempSensorMsg};
use ava_toolkit::generic_device::Locality;
use crate::message_enum::MessageEnum::{Radiator, TempSensor};

/// Object by enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub (crate) enum MessageEnum {
    TempSensor(TempSensorMsg),
    Radiator(RegulatorRadiatorMsg)
}

impl MessageEnum {
    
    /// Convert the current type of message to Temperature Sensor
    fn to_temp_sensor(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }
    
    fn to_radiator(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

}


impl Locality for MessageEnum {
   fn query_for_state(&self) -> String {
        match self {
            TempSensor(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
            Radiator(_) => {
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
            TempSensor(msg) => {
                serde_json::to_string(msg).unwrap() // TODO handle error
            }
            Radiator(msg) => {
                serde_json::to_string(msg).unwrap() // TODO handle error
            }
        }
    }
    /// Convert the original message to the type of the current Self
    fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            TempSensor(_) => {
                original_message.to_temp_sensor(&last_message)
            }
            Radiator(_) => {
                original_message.to_radiator(&last_message)
            }
        }
    }

    fn to_local_with_data(&self,original_message: &Self, last_message: &Self, _ext_data: Option<&HashMap<String, f64>>, _topic: Option<&str>) -> Self {
        self.to_local(original_message, last_message)
    }
    
    fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            TempSensor(_) => {
                Ok(TempSensor(TempSensorMsg::from_json(json_msg)?))
            }
            Radiator(_) => {
                Ok(Radiator(RegulatorRadiatorMsg::from_json(json_msg)?))
            }
        }
    }

    /// Actions liées à l'arrivée des différents messages à enregistrer
    async fn process(&self, topic: &str, _args: &[String]) {
        let json_msg = self.raw_message();
        match self {
            TempSensor(msg) => {
                info!("Default process for TempSensor, message=[{:?}]", msg);
                insert_temp(&topic, &msg).await;
            }
            // MoveSensor(msg) => {
            //     info!("Default process for MoveSensor, message=[{:?}]", msg);
            //     db_put_device_state(&topic, &json_msg).await;
            // }
            // BasicSwitch(msg) => {
            //     info!("Default process for BasicSwitch, message=[{:?}]", msg);
            //     db_put_device_state(&topic, &json_msg).await;
            // }
            Radiator(msg) => {
                info!("Default process for Radiator, message=[{:?}]", msg);
                db_put_device_state(&topic, &json_msg).await;
            }
        }
    }

    async fn compute(&self) -> Option<HashMap<String, f64>> {
        None
    }
    
}

/// Insère les données de l'état du périphérique dans la base de données
pub (crate) async fn db_put_device_state(_topic: &str, _json_msg: &str) {
    panic!("Please, grab the db information from the config file !!");
}

/// Insère les données de température dans la base de données
pub (crate) async fn insert_temp(_topic: &str, _temp: &TempSensorMsg) {
    panic!("Please, grab the db information from the config file !!");
}
