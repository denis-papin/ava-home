use std::collections::HashMap;

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::{error, info};
use reqwest::header;

use crate::device_message::{Radiator, RadiatorMode};
use crate::device_repo::{RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON};
use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum::RadiatorMsg;

/// Object by enums
#[derive(Debug, Clone)]
pub (crate) enum MessageEnum {
    RadiatorMsg(Radiator)
}

impl MessageEnum {

    pub (crate) fn query_for_state(&self) -> String {
        match self {
            RadiatorMsg(_) => {
                let msg = r#"{"state":""}"#;
                msg.to_string()
            }
        }
    }

    pub (crate) fn raw_message(&self) -> String {
        match self {
            RadiatorMsg(msg) => {
                serde_json::to_string(msg).unwrap() // TODO
            }
        }
    }
    pub (crate) fn json_to_local(&self, json_msg: &str) -> Result<MessageEnum, String> {
        match self {
            RadiatorMsg(_) => {
                Ok(RadiatorMsg(Radiator::from_json(json_msg)?))
            }
        }
    }

    pub (crate) fn default_radiator() -> Self {
        RadiatorMsg(Radiator::new())
    }


    /// Convert the original message to the type of the current Self
    pub (crate) fn to_local(&self, original_message: &MessageEnum, last_message: &MessageEnum) -> Self {
        match self {
            RadiatorMsg(_) => {
                original_message.to_radiator(&last_message)
            }
        }
    }

    /// Convert the current type of message to Temperature Sensor
    fn to_radiator(&self, _last_message: &MessageEnum) -> Self {
        self.clone()
    }

    /// Default process for the message
    pub (crate) async fn process(&self, topic: &str, args: &[String]) {
        match self {
            RadiatorMsg(t) => {
                info!("Default process for Radiator, message=[{:?}]", t);
                command_radiator(&topic, &t, &args).await;
            }
        }
    }
}

lazy_static! {
    static ref DEVICE_DID: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        let b_salon : &'static str = Box::leak(GenericDevice::make_topic(RAD_SALON).into_boxed_str());
        map.insert(b_salon, "3wHa7Ja50MhfShUxcmOqvT");
        map.insert(Box::leak(GenericDevice::make_topic(RAD_COULOIR).into_boxed_str()), "JUVo7yMFQtdfZhi25Vo4Bu");
        map.insert(Box::leak(GenericDevice::make_topic(RAD_CHAMBRE).into_boxed_str()), "LNENiFG0MeReR9WtxMebYB");
        map.insert(Box::leak(GenericDevice::make_topic(RAD_BUREAU).into_boxed_str()), "mO7E2B49G1BS8R77UmWIjk");
        map
    };
}

pub (crate) async fn command_radiator(topic: &str, msg: &Radiator, args: &[String]) {
    info!("Command [{}]", &topic);

    let heatzy_pass = args.get(1).unwrap();
    let heatzy_application_id= args.get(2).unwrap();
    let heatzy_token= args.get(3).unwrap(); // "74067d76317946fca0433f684cf1e0a1"

    let did = DEVICE_DID.get(topic).unwrap();
    set_mode(&msg.mode, &heatzy_application_id,  &heatzy_pass, &heatzy_token, &did).await;
    info!("Radiator status changed!");
}

// {
//  "attrs": {
//     "mode":0 // 0 CONFORT,  1 ECO, 2 HORS GEL, 3 OFF
//  }
// }
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