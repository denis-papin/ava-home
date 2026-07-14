use std::collections::HashMap;

use lazy_static::lazy_static;
use log::{error, info};
use ava_toolkit::device_message::{RegulatorRadiatorMsg, RadiatorMode};
use ava_toolkit::generic_device::{GenericDevice, Locality, EXTERNAL_FAMILY};
use common_config::properties::{get_prop_value, set_prop_value};
use radiator_toolkit::HeatzyClient;
use serde_derive::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
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
    static ref HEATZY_TOKEN: Arc<RwLock<String>> = Arc::new(RwLock::new(
        get_prop_value("heatzy.token").expect("Cannot read heatzy.token")
    ));
}

// TODO : we could remove the args param all along the cascade of routines
pub (crate) async fn command_radiator(topic: &str, msg: &RegulatorRadiatorMsg, _args: &[String]) {
    info!("Command [{}]", &topic);

    let heatzy_username = get_prop_value("heatzy.username").unwrap();
    let heatzy_password = get_prop_value("heatzy.password").unwrap();
    let heatzy_application_id = get_prop_value("heatzy.application.id").unwrap();

    let did = DEVICE_DID.get(topic).unwrap();
    match set_mode(
        &msg.mode,
        &heatzy_application_id,
        &heatzy_username,
        &heatzy_password,
        did,
    )
    .await
    {
        Ok(()) => info!("Radiator status changed!"),
        Err(e) => error!("Erreur lors de la requête : {}", e),
    }
}


///  Les modes sont  0 CONFORT,  1 ECO, 2 HORS GEL, 3 OFF
async fn set_mode(
    mode: &RadiatorMode,
    heatzy_application_id: &str,
    heatzy_username: &str,
    heatzy_password: &str,
    did: &str,
) -> anyhow::Result<()> {
    let client = HeatzyClient::new(
        heatzy_application_id,
        heatzy_username,
        heatzy_password,
        HEATZY_TOKEN.clone(),
    );
    let previous_token = client.current_token()?;

    client.set_mode(did, *mode).await?;

    let current_token = client.current_token()?;
    if current_token != previous_token {
        set_prop_value("heatzy.token", &current_token);
        info!("Heatzy token refreshed and saved into runtime configuration");
    }
    info!("Commande radiateur envoyée avec succès");
    Ok(())
}
