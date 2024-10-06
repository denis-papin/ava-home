use std::env;
use std::time::Duration;

use chrono::{Local, NaiveTime};
use log::info;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions};
use tokio::time::interval;

use crate::device_message::RegulationMap;
use crate::device_repo::{build_device_repo, REGULATE_RADIATOR};
use crate::message_enum::MessageEnum;

mod device_message;
mod device_repo;
mod message_enum;
mod generic_device;


const CLIENT_ID: &str = "ava-regulator-heart-beat";

#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub keep_alive :  u16,
}

/// Build the list of channel to listen
fn parse_params() -> Params {
    let client_id = CLIENT_ID.to_string();
    Params {
        server_addr : "192.168.0.149".to_string(),
        client_id,
        //channel_filters,
        keep_alive : 30_000,
    }
}


#[tokio::main]
async fn main() {

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting AVA regulator-heart-beat 0.5.0");

    // Devices
    info!("Building the device repository");
    let device_repo = build_device_repo();
    let params = parse_params();

    // Mosquitto

    let mut mqttoptions = MqttOptions::new(&params.client_id, &params.server_addr, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(params.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials("ava", "avatece3.X");

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let msg_jour = MessageEnum::RegulationMsg(RegulationMap {
        tc_bureau: 21.0,
        tc_salon_1: 23.0,
        tc_salon_2: 0.0,
        tc_chambre_1: 19.0,
        tc_couloir: 23.0,
        mode: 'J',
    });

    let msg_fin_jour = MessageEnum::RegulationMsg(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 23.0,
        tc_salon_2: 0.0,
        tc_chambre_1: 23.0,
        tc_couloir: 23.0,
        mode: 'H',
    });

    let msg_nuit = MessageEnum::RegulationMsg(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 19.0,
        tc_salon_2: 0.0,
        tc_chambre_1: 23.0,
        tc_couloir: 19.0,
        mode: 'N',
    });

    let device = device_repo.get(REGULATE_RADIATOR).unwrap().as_ref().borrow();

    //  5*60
    let mut interval = interval(Duration::from_secs(5*60));
    loop {
        interval.tick().await;

        // Obtenez l'heure actuelle
        let now = Local::now();

        // Obtenez l'heure actuelle au format d'heure (heures et minutes)
        let current_time = now.time();

        // Définissez les heures de début et de fin
        let j_start_time = NaiveTime::from_hms_opt(7, 0, 0).expect("Invalid time");
        let j_end_time = NaiveTime::from_hms_opt(22, 0, 0).expect("Invalid time");
        let h_end_time = NaiveTime::from_hms_opt(23, 59, 0).expect("Invalid time");

        // Vérifiez si l'heure actuelle est entre 7h00 et 22h00
        let msg = if current_time > j_start_time && current_time <= j_end_time {
            info!("L'heure actuelle est entre 7h00 et 22h00.");
            &msg_jour
        } else if current_time > j_end_time && current_time <= h_end_time  {
            info!("L'heure actuelle est entre 22h00 et 00h00.");
            &msg_fin_jour
        } else {
            info!("L'heure actuelle est entre 00h00 et 7h00.");
            &msg_nuit
        };

        info!("prepare to send :  [{:?}]", &msg);
        let _ = device.publish_message_topic(&mut client, msg).await;
        info!("Sent regulation map notification");

        while let Ok(notification) = eventloop.poll().await {
            let mut end_loop = false;
            match notification {
                Event::Incoming(Incoming::PubAck(pub_ack)) => {
                    info!("📩  PubAck ({:?})", &pub_ack);
                    end_loop = true;
                }
                _ => {}
            }
            if end_loop {
                break;
            }
        }
    }
}



