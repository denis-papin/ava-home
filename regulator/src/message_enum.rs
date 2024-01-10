use std::time::SystemTime;
use chrono::{DateTime, Utc};
use log::info;
use reqwest::header;
use tokio_postgres::{NoTls, types::ToSql};

use crate::device_message::{RegulationMap};
use crate::message_enum::MessageEnum::{REGULATION_MAP};

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    REGULATION_MAP(RegulationMap),
}

impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            REGULATION_MAP(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            MessageEnum::REGULATION_MAP(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            REGULATION_MAP(_) => {
                Ok(REGULATION_MAP(RegulationMap::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_regulation_map() -> Self {
        REGULATION_MAP(RegulationMap::new())
    }

    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            REGULATION_MAP(_) => {
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
        match self {
            REGULATION_MAP(rm) => {
                info!("Default process for RegulationMap, message=[{:?}]", rm);
                regulate_radiators(&topic, &rm).await;
            }
        }
    }

}

pub (crate) async fn regulate_radiators(topic: &str, regulation_map: &RegulationMap) {
    // URL de la base de données PostgreSQL
    let db_url = "postgresql://denis:dentece3.X@192.168.0.149/avahome";

    // Établir une connexion à la base de données
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

    // Spawn une tâche pour gérer la processus de connexion en arrière-plan
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erreur de connexion : {}", e);
        }
    });

    // Exécuter une requête de sélection pour obtenir les températures les plus récentes par device_name
    let query = "SELECT DISTINCT ON (device_name) device_name, temperature, ts_create
                 FROM temperature_sensor_history
                 ORDER BY device_name, ts_create DESC";

    let rows = client.query(query, &[]).await.unwrap();

    // Traiter les résultats
    for row in rows {
        let device_name: String = row.get("device_name");
        let temperature: f64 = row.get("temperature");
        let ts_create: SystemTime = row.get("ts_create");
        let dt: DateTime<Utc> = ts_create.clone().into();

        if device_name == "zigbee2mqtt/ts_bureau" {
            regule(&device_name, temperature, regulation_map.tc_bureau).await;
        }

        // Faites quelque chose avec les données lues, par exemple, imprimez-les
        println!("Nom du dispositif: {}, Température: {}, Créé à: {:?}", device_name, temperature, dt);
    }

    println!("succès!");
}

const APPLICATION_ID: &str = ""; // FIXEME DON'T

async fn regule(device_name: &str, temperature: f64, tc: f32 ) {
    if temperature < tc as f64 - 0.5f64 {
        // Set the radiator on "CONFORT"
        println!("Set Bureau to CONFORT");

        // FIXME : DON'T
        let data = serde_json::json!({
            "username": "denis.1@crespe.fr",
            "password": ""
        });

        // URL de destination
        let url = "https://euapi.gizwits.com/app/login";

        // Header personnalisé
        let mut custom_header = header::HeaderMap::new();
        custom_header.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
        custom_header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        custom_header.insert("X-Gizwits-Application-Id", header::HeaderValue::from_static(APPLICATION_ID));

        // Effectuer la requête POST
        match post_data(url, data, custom_header).await {
            Ok(response) => {
                println!("Réponse: {}", response);
            }
            Err(e) => {
                eprintln!("Erreur lors de la requête : {}", e);
            }
        }

    } else if temperature > tc as f64 + 0.5f64 {
        // Set the radiator on "ON"
        println!("Set Bureau to OFF")
    } else {
        // Nothing to do
        println!("Set Bureau to --")
    }
}

async fn post_data(url: &str, data: serde_json::Value, headers: header::HeaderMap) -> Result<String, reqwest::Error> {
    // Créer une nouvelle session Reqwest
    let client = reqwest::Client::new();

    // Effectuer la requête POST
    let response = client.post(url)
        .headers(headers)
        .json(&data)
        .send()
        .await?;

    // Vérifier la réponse HTTP
    if response.status().is_success() {
        // Récupérer le corps de la réponse comme chaîne de caractères
        let body = response.text().await?;
        Ok(body)
    } else {
        // Err(reqwest::Error::new(reqwest::StatusCode::from_u16(response.status().as_u16()).unwrap(), response.text().await?))
        panic!();
    }
}
