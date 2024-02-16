use std::collections::HashMap;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use log::info;
use serde_derive::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Debug, Clone, PartialEq)]
pub (crate) enum RadiatorAction {
    On,
    Off,
    NoAction,
}


pub(crate) async fn compute() -> HashMap<String, f64> {

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

    let mut current_temp: HashMap<String, f64> = HashMap::new();

    // Traiter les résultats
    for row in rows {
        let device_name: String = row.get("device_name");
        let temperature: f64 = row.get("temperature");
        let ts_create: SystemTime = row.get("ts_create");
        let dt: DateTime<Utc> = ts_create.clone().into();

        match device_name.as_str() {
            "zigbee2mqtt/ts_bureau" => {
                current_temp.insert("bureau".to_string(), temperature);
            }
            "zigbee2mqtt/ts_chambre_1" => {
                current_temp.insert("chambre_1".to_string(), temperature);
            }
            "zigbee2mqtt/ts_couloir" => {
                current_temp.insert("couloir".to_string(), temperature);
            }
            "zigbee2mqtt/ts_salon_1" => {
                current_temp.insert("salon_1".to_string(), temperature);
            }
            "zigbee2mqtt/ts_salon_2" => {
                current_temp.insert("salon_2".to_string(), temperature);
            }
            _ => {}
        }
        info!("Device : {}, Température: {}, Créé à: {:?}", device_name, temperature, dt);
    }
    current_temp
}


pub (crate) fn determine_action(t_current: f64, tc: f32) -> RadiatorAction {
    if t_current < tc as f64 - 0.3f64 {
        RadiatorAction::On
    } else if t_current > tc as f64 + 0.3f64 {
        RadiatorAction::Off
    } else {
        RadiatorAction::NoAction
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    mode: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DevData {
    did: String,
    updated_at: u32,
    attr: Attribute,
}

// async fn get_mode(heatzy_application_id: &str,  heatzy_token: &str, did: &str) -> anyhow::Result<RadiatorMode> {
//     let mut custom_header = header::HeaderMap::new();
//     custom_header.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
//     custom_header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
//     custom_header.insert("X-Gizwits-Application-Id", heatzy_application_id.parse().unwrap());
//     custom_header.insert("X-Gizwits-User-token", heatzy_token.parse().unwrap());
//
//     let url = format!("https://euapi.gizwits.com/app/devdata/{}/latest", did); // device did
//
//     match get_data(&url, custom_header).await {
//         Ok(response) => {
//             let r : DevData = serde_json::from_str(&response)?;
//             Ok(RadiatorMode::from_value(r.attr.mode))
//         }
//         Err(e) => {
//             eprintln!("Error while getting the data : {}", e);
//             Err(anyhow!("Error while getting the data"))
//         }
//     }
// }


// async fn get_data(url: &str, headers: header::HeaderMap) -> anyhow::Result<String> {
//     // Créer une nouvelle session Reqwest
//     let client = reqwest::Client::new();
//
//     // Effectuer la requête GET
//     let response = client.get(url)
//         .headers(headers)
//         .send()
//         .await?;
//
//     // Vérifier la réponse HTTP
//     if response.status().is_success() {
//         // Récupérer le corps de la réponse comme chaîne de caractères
//         let body = response.text().await?;
//         Ok(body)
//     } else {
//         Err(anyhow!("{:?}", response))
//     }
// }

