use std::collections::HashMap;
use log::info;
use tokio_postgres::{NoTls, types::ToSql};

use ava_toolkit::device_message::{BasicSwitchMsg, MoveSensorMsg, RegulatorRadiatorMsg, TempSensorMsg};
use ava_toolkit::generic_device::Locality;
use crate::message_enum::MessageEnum::{BasicSwitch, MoveSensor, Radiator, TempSensor};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    TempSensor(TempSensorMsg),
    MoveSensor(MoveSensorMsg),
    BasicSwitch(BasicSwitchMsg),
    Radiator(RegulatorRadiatorMsg)
}

impl MessageEnum {

    pub (crate) fn default_temp_sensor() -> Self {
        TempSensor(TempSensorMsg::new())
    }

    pub (crate) fn default_move_sensor() -> Self {
        MoveSensor(MoveSensorMsg::new())
    }

    pub (crate) fn default_basic_switch() -> Self {
        BasicSwitch(BasicSwitchMsg::new())
    }

    pub (crate) fn default_radiator() -> Self {
        Radiator(RegulatorRadiatorMsg::new())
    }

    /// Convert the current type of message to Temperature Sensor
    fn to_temp_sensor(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    /// Convert the current type of message to Move Sensor
    fn to_move_sensor(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    /// Convert the current type of message to Basic Switch
    fn to_basic_switch(&self, _last_message: &MessageEnum) -> Self {
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
            MoveSensor(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
            BasicSwitch(_) => {
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
            MoveSensor(msg) => {
                serde_json::to_string(msg).unwrap() // TODO handle error
            }
            BasicSwitch(msg) => {
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
            MoveSensor(_) => {
                original_message.to_move_sensor(&last_message)
            }
            BasicSwitch(_) => {
                original_message.to_basic_switch(&last_message)
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
            MoveSensor(_) => {
                Ok(MoveSensor(MoveSensorMsg::from_json(json_msg)?))
            }
            MessageEnum::BasicSwitch(_) => {
                Ok(BasicSwitch(BasicSwitchMsg::from_json(json_msg)?))
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
            MoveSensor(msg) => {
                info!("Default process for MoveSensor, message=[{:?}]", msg);
                db_put_device_state(&topic, &json_msg).await;
            }
            BasicSwitch(msg) => {
                info!("Default process for BasicSwitch, message=[{:?}]", msg);
                db_put_device_state(&topic, &json_msg).await;
            }
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
pub (crate) async fn db_put_device_state(topic: &str, json_msg: &str) {
    let db_url = "postgresql://denis:dentece3.X@192.168.0.149/avahome"; // TODO ...

    // Établissez une connexion à la base de données
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

    // Spawn une tâche pour gérer la processus de connexion en arrière-plan
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query =  r#"INSERT INTO public.device_state_history
                                (device_name, state, ts_create)
                                VALUES($1, $2, timezone('UTC', current_timestamp))"#;

    let values: &[&(dyn ToSql + Sync)] = &[&topic, &json_msg];
    let _ = client.execute(query, values).await.unwrap();

    println!("Données insérées avec succès!");
}

/// Insère les données de température dans la base de données
pub (crate) async fn insert_temp(topic: &str, temp: &TempSensorMsg) {
    // Remplacez ces valeurs par les informations de votre base de données
    let db_url = "postgresql://denis:dentece3.X@192.168.0.149/avahome";

    // Établissez une connexion à la base de données
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

    // Spawn une tâche pour gérer la processus de connexion en arrière-plan
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Données à insérer
    let device_name = topic;
    let temperature = temp.temperature as f64;
    // let ts_create = chrono::Utc::now();

    // Exécutez une requête d'insertion
    let query = "INSERT INTO temperature_sensor_history (device_name, temperature, ts_create) VALUES ($1, $2, timezone('UTC', current_timestamp))";
    let values: &[&(dyn ToSql + Sync)] = &[&device_name, &temperature];

    let _ = client.execute(query, values).await.unwrap();

    println!("Données insérées avec succès!");
}
