use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use log::info;
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;
use ava_toolkit::domotic_factory::DomoticFactory;
use crate::device_repo::{build_device_repo, device_to_listen};
use ava_toolkit::generic_device::GenericDevice;
use ava_toolkit::hard_loop::HardLoop;
use ava_toolkit::init_loop::process_initialization_message;
use ava_toolkit::processing::process_incoming_message;
use crate::loops::{build_init_list, build_loops};
use crate::message_enum::MessageEnum;

mod loops;
mod device_repo;
mod message_enum;
mod external_computing;

const CLIENT_ID: &str = "ava-regulator";

#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(String, QoS)>,
    pub keep_alive :  u16,
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
        server_addr : "192.168.0.149".to_string(),
        client_id,
        channel_filters,
        keep_alive : 30_000,
    }
}

#[tokio::main]
async fn main() {

    // run --package regulator --bin regulator -- <heatzy_pass>   <heatzy_key>

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting AVA regulator 0.5.0");

    let mut domo_factory: DomoticFactory<MessageEnum> = DomoticFactory::new(r"/home/denis/Projects/wks-ava-home/ava-home/luminator/resources/modules.json");
    domo_factory.build_devices();

    // let device_repo = base.repo();
    let all_loops = domo_factory.build_loops();
    let init_list = domo_factory.devices_to_init();
    let device_to_listen = domo_factory.devices_to_listen();

    let args: Vec<String> = vec![];
    let channels = DomoticFactory::extract_channel_from_devices(&device_to_listen);

    let mut mqttoptions = MqttOptions::new(&channels.client_id, &channels.server_addr, 1883);
    mqttoptions.set_keep_alive(Duration(channels.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials("ava", "avatece3.X");

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    for p in &channels.channel_filters {
        info!("Subscribe to [{}]", p.0);
        client.subscribe(p.0.clone(), QoS::AtMostOnce).await.unwrap();
    }

    let loop_finder = |topic: &str| {
        HardLoop::find_loops(topic, &all_loops)
    };

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



