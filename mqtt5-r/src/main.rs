use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use log::info;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, MqttOptions};

use crate::device_repo::{build_device_repo, device_to_listen};
use crate::loops::{build_init_list, build_loops};
use crate::message_enum::MessageEnum;
use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;
use ava_toolkit::init_loop::process_initialization_message;
use ava_toolkit::processing::process_incoming_message;

mod device_repo;
mod loops;
mod message_enum;

const CLIENT_ID: &str = "ava-0.5.0";

#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr: String,
    pub client_id: String,
    pub channel_filters: Vec<(String, QoS)>,
    pub keep_alive: u16,
}

/// Build the list of channel to listen
fn parse_params(device_repo: &HashMap<String, Arc<RefCell<GenericDevice<MessageEnum>>>>) -> Params {
    let client_id = CLIENT_ID.to_string();

    let mut channel_filters: Vec<(String, QoS)> = vec![];
    for dev in device_to_listen(&device_repo) {
        let dd = dev.as_ref().borrow();
        let topic = dd.get_topic();
        channel_filters.push((topic, QoS::AtMostOnce));
    }

    Params {
        server_addr: "raspberrypi.local".to_string(),
        client_id,
        channel_filters,
        keep_alive: 30_000,
    }
}

#[tokio::main]
async fn main() {
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    info!("Starting AVA 0.5.0");

    info!("Building the device repository");
    let args: Vec<String> = vec![];
    let device_repo = build_device_repo();
    let params = parse_params(&device_repo);

    let mut mqttoptions = MqttOptions::new(&params.client_id, &params.server_addr, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(params.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    let mqtt_user = env::var("AVA_MQTT_USER").expect("Missing AVA_MQTT_USER");
    let mqtt_password = env::var("AVA_MQTT_PASSWORD").expect("Missing AVA_MQTT_PASSWORD");
    mqttoptions.set_credentials(mqtt_user, mqtt_password);

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 15);

    for p in &params.channel_filters {
        info!("Subscribe to [{}]", p.0);
        client
            .subscribe(p.0.clone(), QoS::AtLeastOnce)
            .await
            .unwrap();
    }

    let mut init_list = build_init_list(&device_repo);
    let all_loops = build_loops(&device_repo);

    let loop_finder = |topic: &str| HardLoop::find_loops(topic, &all_loops);

    match process_initialization_message(&mut client, &mut eventloop, &mut init_list).await {
        Ok(_) => {
            info!("Process incoming messages");
            let _ = process_incoming_message(&mut client, &mut eventloop, &args, loop_finder).await;
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    println!("Done!");
}
