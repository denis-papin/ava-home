use crate::message_enum::MessageEnum;
use ava_toolkit::domotic_factory::DomoticFactory;
use ava_toolkit::hard_loop::HardLoop;
use ava_toolkit::init_loop::process_initialization_message;
use ava_toolkit::processing::process_incoming_message;
use common_config::conf_reader::{read_config, read_env};
use common_config::properties::{get_prop_value, set_prop_values};
use log::{error, info};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, MqttOptions};
use std::env;
use std::time::Duration;

mod message_enum;

#[tokio::main]
async fn main() {
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    info!("Starting AVA radiator-ctrl 0.5.0");

    const PROJECT_CODE: &str = "radiator-ctrl";
    const VAR_NAME: &str = "AVA_ENV";

    let o_config_file = read_env(&VAR_NAME);

    // Read the application config's file
    println!(
        "😎 Config file using PROJECT_CODE={} VAR_NAME={}",
        PROJECT_CODE, VAR_NAME
    );

    let props = read_config(
        PROJECT_CODE,
        &o_config_file,
        &Some("AVA_CLUSTER_PROFILE".to_string()),
    );
    set_prop_values(props);

    let factory_message_dir = read_props_or_die("factory.dir");
    let module_file = read_props_or_die("module");
    let mqtt_port = read_props_or_die("mqtt.port").parse::<u16>().unwrap(); // TODO
    let mqtt_user = read_props_or_die("mqtt.user");
    let mqtt_password = read_props_or_die("mqtt.password");
    let mqtt_host = read_props_or_die("mqtt.host");

    let mut domo_factory: DomoticFactory<MessageEnum> =
        DomoticFactory::new(module_file, factory_message_dir);
    domo_factory.build_devices();

    let all_loops = domo_factory.build_loops();
    let init_list = domo_factory.devices_to_init();
    let device_to_listen = domo_factory.devices_to_listen();

    let channels = DomoticFactory::extract_channel_from_devices(&device_to_listen, &mqtt_host);

    let mut mqttoptions = MqttOptions::new(&channels.client_id, &channels.server_addr, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(channels.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials(mqtt_user, mqtt_password);

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 100);

    for p in &channels.channel_filters {
        info!("Subscribe to [{}]", p.0);
        client
            .subscribe(p.0.clone(), QoS::AtLeastOnce)
            .await
            .unwrap();
    }

    let loop_finder = |topic: &str| HardLoop::find_loops(topic, &all_loops);

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

fn read_props_or_die(property_name: &str) -> String {
    let value = match get_prop_value(property_name) {
        Ok(file) => file,
        Err(e) => {
            error!("{}", e);
            panic!("Cannot find the property")
        }
    };
    value
}
