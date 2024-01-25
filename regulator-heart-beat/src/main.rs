use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use chrono::{Local, NaiveTime};

use log::info;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;
use tokio::time::interval;

use crate::device_message::RegulationMap;
use crate::device_repo::{build_device_repo, device_to_listen, REGULATE_RADIATOR};
use crate::generic_device::GenericDevice;
use crate::message_enum::MessageEnum;

mod device_lock;
mod device_message;
mod loops;
mod device_repo;
mod init_loop;
mod processing;
mod message_enum;
mod generic_device;

const CLIENT_ID: &str = "ava-regulator-heart-beat-0.5.0";

#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(String, QoS)>,
    pub keep_alive :  u16,
}

/// Build the list of channel to listen
fn parse_params(device_repo: &HashMap<String, Arc<RefCell<GenericDevice>>>) -> Params {
    let client_id = CLIENT_ID.to_string();

    let mut channel_filters: Vec<(String, QoS)> = vec![];
    for dev in device_to_listen(&device_repo) {
        let dd = dev.as_ref().borrow();
        let topic = dd.get_topic();
        channel_filters.push((topic, QoS::AtMostOnce));
    }

    Params {
        server_addr : "192.168.0.149".to_string(),
        client_id,
        channel_filters,
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
    let params = parse_params(&device_repo);

    // Mosquitto

    let mut mqttoptions = MqttOptions::new(&params.client_id, &params.server_addr, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(params.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials("ava", "avatece3.X");

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let msg_jour = MessageEnum::REGULATION_MAP(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 23.0,
        tc_salon_2: 19.0,
        tc_chambre_1: 19.0,
        tc_couloir: 23.0,
        mode: 'J',
    });

    let msg_nuit = MessageEnum::REGULATION_MAP(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 19.0,
        tc_salon_2: 19.0,
        tc_chambre_1: 23.0,
        tc_couloir: 19.0,
        mode: 'N',
    });

    let msg_fin_jour = MessageEnum::REGULATION_MAP(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 23.0,
        tc_salon_2: 23.0,
        tc_chambre_1: 23.0,
        tc_couloir: 23.0,
        mode: 'H',
    });

    let device = device_repo.get(REGULATE_RADIATOR).unwrap().as_ref().borrow();

    let mut interval = interval(Duration::from_secs(5*60));
    loop {
        interval.tick().await;

        // Obtenez l'heure actuelle
        let now = Local::now();

        // Obtenez l'heure actuelle au format d'heure (heures et minutes)
        let current_time = now.time();

        // Définissez les heures de début et de fin
        let j_start_time = NaiveTime::from_hms(7, 0, 0);
        let j_end_time = NaiveTime::from_hms(22, 0, 0);

        let h_end_time = NaiveTime::from_hms(23, 59, 0);

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
        let a = device.publish_message_topic(&mut client, msg).await;

        println!("Sent regulation map notification");
        while let Ok(notification) = eventloop.poll().await {
            let mut end_loop = false;
            match notification {
                Event::Incoming(Incoming::Publish(publish)) => {
                    info!("🧶 Publish : [{:?}]", &publish);
                }
                Event::Incoming(Incoming::ConnAck(connack)) => {

                }
                Event::Incoming(Incoming::PubAck(pubAck)) => {
                    info!("📩  PubAck ({:?})", &pubAck);
                    end_loop = true;
                }
                _ => {}
            }
            if end_loop {
                break;
            }
        }

    }

    println!("Done!");
}



