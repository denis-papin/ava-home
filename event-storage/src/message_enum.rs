use log::info;
use tokio_postgres::{NoTls, types::ToSql};

use crate::device_message::{Radiator, TempSensor};
use crate::message_enum::MessageEnum::{RadiatorMsg, TempSensorMsg};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    TempSensorMsg(TempSensor),
    RadiatorMsg(Radiator)
}

impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            TempSensorMsg(_) => {
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
            TempSensorMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
            RadiatorMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            TempSensorMsg(_) => {
                Ok(TempSensorMsg(TempSensor::from_json(json_msg)?))
            }
            RadiatorMsg(_) => {
                Ok(RadiatorMsg(Radiator::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_temp_sensor() -> Self {
        TempSensorMsg(TempSensor::new())
    }

    pub (crate) fn default_radiator() -> Self {
        RadiatorMsg(Radiator::new())
    }

    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            TempSensorMsg(_) => {
                original_message.to_temp_sensor(&last_message)
            }
            RadiatorMsg(_) => {
                original_message.to_radiator(&last_message)
            }
        }
    }

    /// Convert the current type of message to Temperature Sensor
    fn to_temp_sensor(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    fn to_radiator(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    /// Default process for the message
    pub (crate) async fn process(&self, topic: &str) {
        let json_msg = self.raw_message();
        match self {
            TempSensorMsg(msg) => {
                info!("Default process for TempSensor, message=[{:?}]", msg);
                insert_temp(&topic, &msg).await;
            }
            RadiatorMsg(msg) => {
                info!("NOW EMPTY PROCESS - Default process for Radiator, message=[{:?}]", msg);
                db_put_device_state(&topic, &json_msg).await;
            }
        }
    }
}

pub (crate) async fn db_put_device_state(topic: &str, json_msg: &str) {
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
    // let device_name = topic;
    // let temperature = temp.temperature as f64;
    // let ts_create = chrono::Utc::now();

    // Exécutez une requête d'insertion
    // let query = "INSERT INTO temperature_sensor_history (device_name, temperature, ts_create) VALUES ($1, $2, now())";

    let query =  r#"INSERT INTO public.device_state_history
                                (device_name, state, ts_create)
                                VALUES($1, $2, timezone('UTC', current_timestamp))"#;

    let values: &[&(dyn ToSql + Sync)] = &[&topic, &json_msg];
    let _ = client.execute(query, values).await.unwrap();

    println!("Données insérées avec succès!");
}

pub (crate) async fn insert_temp(topic: &str, temp: &TempSensor) {
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
