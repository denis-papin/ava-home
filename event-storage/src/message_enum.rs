use log::info;
use tokio_postgres::{NoTls, types::ToSql};

use crate::device_message::TempSensor;
use crate::message_enum::MessageEnum::TEMP_SENSOR;

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    TEMP_SENSOR(TempSensor)
}



impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            TEMP_SENSOR(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            TEMP_SENSOR(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            TEMP_SENSOR(_) => {
                Ok(TEMP_SENSOR(TempSensor::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_temp_sensor() -> Self {
        TEMP_SENSOR(TempSensor::new())
    }


    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            TEMP_SENSOR(_) => {
                original_message.to_temp_sensor(&last_message)
            }
        }
    }

    /// Convert the current type of message to Temperature Sensor
    fn to_temp_sensor(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    /// Default process for the message
    pub (crate) async fn process(&self, topic: &str) {

        // let mut r_cnx = SQLConnection::new();
        // let r_trans = open_transaction(&mut r_cnx).map_err(err_fwd!("💣 Open transaction error"));
        // let Ok(mut trans) = r_trans else {
        //     panic!("Cannot open db transaction!"); // TODO
        // };

        match self {
            TEMP_SENSOR(t) => {
                info!("Default process for TempSensor, message=[{:?}]", t);
                insert_temp(&topic, &t).await;
            }
        }
    }

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
    let query = "INSERT INTO temperature_sensor_history (device_name, temperature, ts_create) VALUES ($1, $2, now())";
    let values: &[&(dyn ToSql + Sync)] = &[&device_name, &temperature];

    let _ = client.execute(query, values).await.unwrap();

    println!("Données insérées avec succès!");
}
