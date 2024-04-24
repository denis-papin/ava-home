use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use log::info;
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;

use crate::device_repo::{build_device_repo, device_to_listen};
use crate::generic_device::GenericDevice;
use crate::init_loop::{build_init_list, process_initialization_message};
use crate::loops::build_loops;
use crate::processing::process_incoming_message;

mod device_lock;
mod device_message;
mod loops;
mod device_repo;
mod init_loop;
mod processing;
mod message_enum;
mod generic_device;

const CLIENT_ID: &str = "ava-event-storage";

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

    info!("Starting AVA event-storage 0.5.0");

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

    for p in &params.channel_filters {
        info!("Subscribe to [{}]", p.0);
        client.subscribe(p.0.clone(), QoS::AtMostOnce).await.unwrap();
    }

    let mut init_list = build_init_list(&device_repo);
    let mut all_loops = build_loops(&device_repo);

    match process_initialization_message(&mut client, &mut eventloop, &mut init_list).await {
        Ok(_) => {
            info!("Process incoming messages");
            let _ = process_incoming_message(&mut client, &mut eventloop, &mut all_loops).await;
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    println!("Done!");
}



