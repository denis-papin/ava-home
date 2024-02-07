use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::{error, info};
use reqwest::header;
use serde_derive::{Deserialize, Serialize};
use tokio_postgres::NoTls;

use crate::device_message::RegulationMap;
use crate::message_enum::MessageEnum::REGULATION_MAP;
use crate::message_enum::RadiatorAction::{NO_ACTION, OFF, ON};
use crate::message_enum::RadiatorMode::{CFT, ECO, FRO, STOP};
use crate::properties::{get_prop_value, set_prop_value};

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
    pub (crate) async fn process(&self, topic: &str, args: &[String]) {
        match self {
            REGULATION_MAP(rm) => {
                info!("Default process for RegulationMap, message=[{:?}]", rm);
                regulate_radiators(&topic, &rm, &args).await;
            }
        }
    }

}

pub (crate) async fn regulate_radiators(topic: &str, regulation_map: &RegulationMap, args: &[String]) {

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
        println!("Device : {}, Température: {}, Créé à: {:?}", device_name, temperature, dt);
    }

    println!("Regulation Map = {:?}", &regulation_map);

    let action_bureau = determine_action(*current_temp.get("bureau").unwrap(), regulation_map.tc_bureau);
    let action_chambre_1 = determine_action(*current_temp.get("chambre_1").unwrap(), regulation_map.tc_chambre_1);
    let action_couloir = determine_action(*current_temp.get("couloir").unwrap(), regulation_map.tc_couloir);
    let action_salon_1 = determine_action(*current_temp.get("salon_1").unwrap(), regulation_map.tc_salon_1);
    // let action_salon_2 = determine_action(*current_temp.get("salon_2").unwrap(), regulation_map.tc_salon_2);

    let str_count = get_prop_value("check_radiator_mode_counter").unwrap();
    let count : u16 = str_count.parse().unwrap();

    info!("Check count: [{}]", count);

    let check_mode = count >= 2; // 0, 1  we don't check
    regulate("bureau", action_bureau, &args, check_mode).await;
    regulate("chambre_1", action_chambre_1, &args, check_mode).await;
    regulate("couloir", action_couloir, &args, check_mode).await;
    regulate("salon", action_salon_1, &args, check_mode).await;

    if check_mode {
        set_prop_value("check_radiator_mode_counter", "0");
    } else {
        set_prop_value("check_radiator_mode_counter", &(count +1).to_string());
    }

    info!("succss!");
}

 #[derive(Clone, PartialEq)]
enum RadiatorAction {
    ON,
    OFF,
    NO_ACTION,
}

impl RadiatorAction {
    fn value(&self) -> &'static str {
        match self {
            ON => "ON",
            OFF => "OFF",
            NO_ACTION => "NO_ACTION",
        }
    }

    fn from_value(value : String) -> Self {
        match value.as_str() {
            "ON" => ON,
            "OFF" => OFF,
            "NO_ACTION" => NO_ACTION,
            _ => NO_ACTION,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RadiatorMode {
    CFT,
    ECO,
    FRO,
    STOP
}

impl RadiatorMode {
    fn from_value(value : String) -> Self {
        match value.as_str() {
            "cft" => CFT,
            "eco" => ECO,
            "fro" => FRO,
            _ => STOP,
        }
    }
}

fn determine_action(t_current: f64, tc: f32) -> RadiatorAction {
    if t_current < tc as f64 - 0.3f64 {
       ON
    } else if t_current > tc as f64 + 0.3f64 {
        OFF
    } else {
        NO_ACTION
    }
}

lazy_static! {
    static ref DEVICE_DID: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("salon", "3wHa7Ja50MhfShUxcmOqvT");
        map.insert("couloir", "JUVo7yMFQtdfZhi25Vo4Bu");
        map.insert("chambre_1", "LNENiFG0MeReR9WtxMebYB");
        map.insert("bureau", "mO7E2B49G1BS8R77UmWIjk");
        map
    };
}

async fn regulate(radiator_name: &str, action: RadiatorAction, args: &[String], check_mode: bool ) {

    info!("Regulate [{}]", &radiator_name);

    let heatzy_pass = args.get(1).unwrap();
    let heatzy_application_id= args.get(2).unwrap();

    let did = DEVICE_DID.get(radiator_name).unwrap();

    let action = if check_mode {
        info!("\t\tCheck mode is ON");
        match get_mode(&heatzy_application_id, "74067d76317946fca0433f684cf1e0a1", &did).await {
            Ok(radiator_mode) => {
                info!("\t\tCurrent Mode is [{:?}]", &radiator_mode);
                match &radiator_mode {
                    CFT => {
                        set_prop_value(radiator_name, RadiatorAction::ON.value());
                        action
                    }
                    ECO | FRO => {
                        set_prop_value(radiator_name, RadiatorAction::NO_ACTION.value());
                        info!("️\t\t️👹 The radiator is manually disable, overriding to NO ACTION");
                        RadiatorAction::NO_ACTION
                    }
                    STOP => {
                        set_prop_value(radiator_name, RadiatorAction::OFF.value());
                        action
                    }
                }
            }
            Err(e) => {
                error!("\t\tError in getting the radiator mode : {:?}", e);
                action
            }
        }
    } else {
        action
    };

    let mode = RadiatorAction::from_value(get_prop_value(radiator_name).unwrap());

    info!("\t\tCurrent memory mode: [{}]", mode.value());

    match action {
        ON => {
            info!("️\t\t🕯️ Must be ON");
        }
        OFF => {
            info!("️\t\t❄️ Must be OFF");
        }
        NO_ACTION => {
            info!("️\t\tNo Action");
        }
    }

    if mode != action && action != NO_ACTION {
        match action {
            ON => {
                info!("\t\t🔥 Set {} to ON", &radiator_name);
            }
            OFF => {
                info!("\t\t❄️ Set {} to OFF", &radiator_name);
            }
            NO_ACTION => {
                info!("️\t\tNo Action");
            }
        }

        // 74067d76317946fca0433f684cf1e0a1
        set_mode(&action, &heatzy_application_id,  &heatzy_pass, "74067d76317946fca0433f684cf1e0a1", &did).await;
        set_prop_value(radiator_name, action.value());
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    token: String,
    uid: String,
    expire_at: u64,
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

async fn get_mode(heatzy_application_id: &str,  heatzy_token: &str, did: &str) -> anyhow::Result<RadiatorMode> {
    let mut custom_header = header::HeaderMap::new();
    custom_header.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
    custom_header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    custom_header.insert("X-Gizwits-Application-Id", heatzy_application_id.parse().unwrap());
    custom_header.insert("X-Gizwits-User-token", heatzy_token.parse().unwrap());

    let url = format!("https://euapi.gizwits.com/app/devdata/{}/latest", did); // device did

    match get_data(&url, custom_header).await {
        Ok(response) => {
            let r : DevData = serde_json::from_str(&response)?;
            Ok(RadiatorMode::from_value(r.attr.mode))
        }
        Err(e) => {
            eprintln!("Error while getting the data : {}", e);
            Err(anyhow!("Error while getting the data"))
        }
    }
}


// {
//  "attrs": {
//     "mode":0 // 0 CONFORT,  1 ECO, 2 HORS GEL, 3 OFF
//  }
// }
async fn set_mode(mode: &RadiatorAction, heatzy_application_id: &str, heatzy_pass: &str, heatzy_token: &str, did: &str) {

    let h_mode = match mode {
        ON => 0,
        OFF => 3,
        _ => 2,
    };

    let data = serde_json::json!({
         "attrs": {
            "mode": h_mode
         }
    });

    let url = format!("https://euapi.gizwits.com/app/control/{}", did); // device did

    let mut custom_header = header::HeaderMap::new();
    custom_header.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
    custom_header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    custom_header.insert("X-Gizwits-Application-Id", heatzy_application_id.parse().unwrap());
    custom_header.insert("X-Gizwits-User-token", heatzy_token.parse().unwrap());

    match post_data(&url, data, custom_header).await {
        Ok(response) => {
            println!("Réponse: {}", response);
        }
        Err(e) => {
            eprintln!("Erreur lors de la requête : {}", e);
            //panic!()
        }
    }
}

async fn login(heatzy_pass: &str, heatzy_application_id: &str) -> LoginResponse {

    let data = serde_json::json!({
            "username": "denis.1@crespe.fr",
            "password": heatzy_pass.clone(),
        });

    // URL de destination
    let url = "https://euapi.gizwits.com/app/login";

    // Header personnalisé
    let mut custom_header = header::HeaderMap::new();
    custom_header.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
    custom_header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    custom_header.insert("X-Gizwits-Application-Id", heatzy_application_id.parse().unwrap());

    // Effectuer la requête POST
    match post_data(url, data, custom_header).await {
        Ok(response) => {
            println!("Réponse: {}", response);
            let login_response : LoginResponse = serde_json::from_str(&response).unwrap();
            login_response
        }
        Err(e) => {
            eprintln!("Erreur lors de la requête : {}", e);
            panic!()
        }
    }

}

async fn get_data(url: &str, headers: header::HeaderMap) -> anyhow::Result<String> {
    // Créer une nouvelle session Reqwest
    let client = reqwest::Client::new();

    // Effectuer la requête GET
    let response = client.get(url)
        .headers(headers)
        .send()
        .await?;

    // Vérifier la réponse HTTP
    if response.status().is_success() {
        // Récupérer le corps de la réponse comme chaîne de caractères
        let body = response.text().await?;
        Ok(body)
    } else {
        Err(anyhow!("{:?}", response))
    }
}

async fn post_data(url: &str, data: serde_json::Value, headers: header::HeaderMap) -> anyhow::Result<String> {
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
        Err(anyhow!("{:?}", response))
    }
}
