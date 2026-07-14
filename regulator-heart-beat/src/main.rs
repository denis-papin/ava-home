use std::env;
use std::process::exit;
use std::time::Duration;

use crate::dao::get_current_regulation_map;
use crate::message_enum::MessageEnum;
use ava_toolkit::domotic_factory::DomoticFactory;
use common_config::conf_reader::{read_config, read_env};
use common_config::properties::{get_prop_pg_connect_string, get_prop_value, set_prop_values};
use commons_error::*;
use commons_pg::sql_transaction2::init_db_pool2;
use log::info;
use log::*;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions};
use tokio::time::interval;

mod dao;
mod message_enum;

pub(crate) const REGULATE_RADIATOR: &str = "regulate_radiator";

#[tokio::main]
async fn main() {
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    info!("Starting AVA regulator-heart-beat 0.5.0");

    const PROJECT_CODE: &str = "regulator-heart-beat";
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

    let props = common_config::conf_reader::read_config(
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

    let device_to_listen = domo_factory.devices_to_listen();
    let device_repo = domo_factory.repo();

    let channels = DomoticFactory::extract_channel_from_devices(&device_to_listen, &mqtt_host);

    // Devices
    info!("Building the device repository");
    let mut mqttoptions = MqttOptions::new(&channels.client_id, &channels.server_addr, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(channels.keep_alive as u64));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials(mqtt_user, mqtt_password);

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 15);

    // Init DB pool
    let (connect_string, db_pool_size) = match get_prop_pg_connect_string()
        .map_err(err_fwd!("Cannot read the database connection information"))
    {
        Ok(x) => x,
        Err(e) => {
            log_error!("{:?}", e);
            exit(-64);
        }
    };

    log_info!("Connnect String : [{}]", &connect_string);

    let _r = init_db_pool2(&connect_string, db_pool_size).await;

    let device = device_repo
        .get(REGULATE_RADIATOR)
        .unwrap()
        .as_ref()
        .borrow();

    //  5 minutes
    let mut interval = interval(Duration::from_secs(5 * 60));
    loop {
        interval.tick().await;

        if let Ok(reg_plan) = get_current_regulation_map().await {
            info!(
                "L'heure actuelle est entre {} et {}.",
                reg_plan.0, reg_plan.1
            );
            let msg = MessageEnum::RegulationMap(reg_plan.2);

            info!("prepare to send :  [{:?}]", &msg);
            let _ = device.publish_message(&mut client, &msg).await;
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
