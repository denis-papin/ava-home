
use std::env;

use std::time::Duration;

use crate::message_enum::MessageEnum;
use ava_toolkit::domotic_factory::DomoticFactory;

use ava_toolkit::hard_loop::HardLoop;
use ava_toolkit::init_loop::process_initialization_message;
use ava_toolkit::processing::process_incoming_message;
use log::info;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, MqttOptions};

mod message_enum;


#[tokio::main]
async fn main() {

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();
    
    info!("Starting AVA 0.5.0");
    info!("Building the device repository");

    let mut domo_factory: DomoticFactory<MessageEnum> = DomoticFactory::new(r"/home/denis/Projects/wks-ava-home/ava-home/luminator/resources/modules.json");
    domo_factory.build_devices();
    
    let all_loops = domo_factory.build_loops();
    let init_list = domo_factory.devices_to_init();
    let device_to_listen = domo_factory.devices_to_listen();

    let args: Vec<String> = vec![];
    let channels = DomoticFactory::extract_channel_from_devices(&device_to_listen);
    
    let mut mqttoptions = MqttOptions::new(&channels.client_id, &channels.server_addr, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(channels.keep_alive as u64));
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
    
    match process_initialization_message(&mut client, &mut eventloop, &init_list).await {
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



