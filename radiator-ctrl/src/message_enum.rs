use std::collections::HashMap;

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::{error, info};
use reqwest::header;
use reqwest::header::GetAll;
use serde_derive::{Deserialize, Serialize};
use ava_toolkit::device_message::{RegulatorRadiatorMsg, RadiatorMode};
use ava_toolkit::generic_device::{GenericDevice, Locality, EXTERNAL_FAMILY};
use common_config::properties::get_prop_value;
use crate::message_enum::MessageEnum::RegulatorRadiator;

/// Object by enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub (crate) enum MessageEnum {
    RegulatorRadiator(RegulatorRadiatorMsg)
}

impl MessageEnum {
    /// Convert the current type of message to Temperature Sensor
    fn to_radiator(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }
}

impl Locality for MessageEnum {
    fn query_for_state(&self) -> String {
        match self {
            RegulatorRadiator(_) => {
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
            RegulatorRadiator(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }

    /// Convert the original message to the type of the current Self
    fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            RegulatorRadiator(_) => {
                original_message.to_radiator(&last_message)
            }
        }
    }

    fn to_local_with_data(&self, original_message: &Self, last_message: &Self, _ext_data: Option<&HashMap<String, f64>>, _topic: Option<&str>) -> Self {
        self.to_local(original_message, last_message)
    }
    
    fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            RegulatorRadiator(_) => {
                Ok(RegulatorRadiator(RegulatorRadiatorMsg::from_json(json_msg)?))
            }
        }
    }

    
    /// Default process for the message
    async fn process(&self, topic: &str, args: &[String]) {
        match self {
            RegulatorRadiator(t) => {
                info!("Default process for Radiator, message=[{:?}]", t);
                command_radiator(&topic, &t, &args).await;
            }
        }
    }

    async fn compute(&self) -> Option<HashMap<String, f64>> {
        None
    }
}

const RAD_SALON: &str = "rad_salon";
const RAD_BUREAU: &str = "rad_bureau";
const RAD_COULOIR: &str = "rad_couloir";
const RAD_CHAMBRE: &str = "rad_chambre";

// TODO  In the general configuration file, setup this mapping and write a global construction from it.
lazy_static! {
    static ref DEVICE_DID: HashMap<&'static str, &'static str> = HashMap::from([
        (
            &*Box::leak(GenericDevice::<MessageEnum>::make_topic(EXTERNAL_FAMILY, RAD_SALON).into_boxed_str()),
            "3wHa7Ja50MhfShUxcmOqvT"
        ),
        (
            &*Box::leak(GenericDevice::<MessageEnum>::make_topic(EXTERNAL_FAMILY, RAD_COULOIR).into_boxed_str()),
            "JUVo7yMFQtdfZhi25Vo4Bu"
        ),
        (
            &*Box::leak(GenericDevice::<MessageEnum>::make_topic(EXTERNAL_FAMILY, RAD_CHAMBRE).into_boxed_str()),
            "LNENiFG0MeReR9WtxMebYB"
        ),
        (
            &*Box::leak(GenericDevice::<MessageEnum>::make_topic(EXTERNAL_FAMILY, RAD_BUREAU).into_boxed_str()),
            "mO7E2B49G1BS8R77UmWIjk"
        ),
    ]);
}

// TODO : we could remove the args param all along the cascade of routines
pub (crate) async fn command_radiator(topic: &str, msg: &RegulatorRadiatorMsg, args: &[String]) {
    info!("Command [{}]", &topic);

    // let heatzy_pass = args.get(1).unwrap();
    let heatzy_pass = get_prop_value("heatzy.pass").unwrap(); // TODO
    // let heatzy_application_id= args.get(2).unwrap();
    let heatzy_application_id = get_prop_value("heatzy.application.id").unwrap();
    // let heatzy_token= args.get(3).unwrap(); // "74067d76317946fca0433f684cf1e0a1"
    let heatzy_token= get_prop_value("heatzy.token").unwrap();

    let did = DEVICE_DID.get(topic).unwrap();
    set_mode(&msg.mode, &heatzy_application_id,  &heatzy_pass, &heatzy_token, &did).await;
    info!("Radiator status changed!");
}


///  Les modes sont  0 CONFORT,  1 ECO, 2 HORS GEL, 3 OFF
async fn set_mode(mode: &RadiatorMode, heatzy_application_id: &str, _heatzy_pass: &str, heatzy_token: &str, did: &str) {

    let h_mode = match mode {
        RadiatorMode::CFT => 0,
        RadiatorMode::ECO => 1,
        RadiatorMode::FRO => 2,
        RadiatorMode::STOP => 3,
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
            info!("Réponse: {}", response);
        }
        Err(e) => {
            error!("Erreur lors de la requête : {}", e);
        }
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