use std::collections::HashMap;
use std::env;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::sync::RwLock;
use std::time::Duration;

use chrono::{Local, NaiveTime};
use lazy_static::lazy_static;
use log::info;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions};
use tokio::time::interval;
use log::*;
use commons_error::*;
use commons_pg::sql_transaction2::init_db_pool2;
use crate::conf_reader::read_config;
use crate::dao::get_current_regulation_map;

use crate::device_message::RegulationMap;
use crate::device_repo::{build_device_repo, REGULATE_RADIATOR};
use crate::regulation_message::MessageEnum;

mod device_message;
mod device_repo;
mod regulation_message;
mod generic_device;
mod dao;
mod conf_reader;

// PROPERTIES must be locked when on write, but not locked on read actions
// It contains a double map { 0 : { "server.port" : 30040, "app.secret-folder" : "/secret", .... },... }
// where only the map[0] is used in our case.
lazy_static! {
    #[derive(Debug)]
    static ref PROPERTIES : RwLock<HashMap<u32, &'static mut HashMap<String,String>> > = RwLock::new(
        {
            let mut m = HashMap::new();
            let props : HashMap<String,String> = HashMap::new();
            m.insert(0, Box::leak(Box::new( props )));
            m
        });
}

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

fn set_props(props : HashMap<String, String>) {
    let mut w = PROPERTIES.write().unwrap();
    let item = w.get_mut(&0).unwrap();
    *item = Box::leak(Box::new(props ));
}


///
/// Get prop value from the application.properties file
///
fn get_prop_value(prop_name : &str) -> String {
    // https://doc.rust-lang.org/std/sync/struct.RwLock.html
    PROPERTIES.read().unwrap().deref().get(&0).unwrap().deref()
        .get(prop_name).unwrap().to_owned()
}


pub fn get_prop_pg_connect_string() -> anyhow::Result<(String,u32)> {
    let db_hostname = get_prop_value("db.hostname");
    let db_port = get_prop_value("db.port");
    let db_name = get_prop_value("db.name");
    let db_user = get_prop_value("db.user");
    let db_password = get_prop_value("db.password");
    let db_pool_size = get_prop_value("db.pool_size").parse::<u32>().map_err(err_fwd!("Cannot read the pool size"))?;
    // let cs = format!("host={} port={} dbname={} user={} password={}", db_hostname, db_port, db_name, db_user,db_password);
    let cs = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_hostname, db_port, db_name
    );
    Ok((cs, db_pool_size))
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
    });

    let msg_fin_jour = MessageEnum::RegulationMsg(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 23.0,
        tc_salon_2: 0.0,
        tc_chambre_1: 23.0,
        tc_couloir: 23.0,
    });

    let msg_nuit = MessageEnum::RegulationMsg(RegulationMap {
        tc_bureau: 19.0,
        tc_salon_1: 19.0,
        tc_salon_2: 0.0,
        tc_chambre_1: 23.0,
        tc_couloir: 19.0,
    });

    //
    const PROJECT_CODE: &str = "dashboard"; // TODO ...
    const VAR_NAME: &str = "DASH_ENV"; // AVA_ENV ???
    println!("😎 Config file using PROJECT_CODE={} VAR_NAME={}", PROJECT_CODE, VAR_NAME);

    let props = read_config(PROJECT_CODE, VAR_NAME);
    set_props(props);
    let port = get_prop_value("server.port").parse::<u16>().unwrap();
    let log_config: String = get_prop_value("log4rs.config");
    let log_config_path = Path::new(&log_config);

    println!("😎 Read log properties from {:?}", &log_config_path);

    // match log4rs::init_file(&log_config_path, Default::default()) {
    //     Err(e) => {
    //         eprintln!("{:?} {:?}", &log_config_path, e);
    //         exit(-59);
    //     }
    //     Ok(_) => {}
    // }
    //
    // log_info!("Init logs ok");

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

    let device = device_repo.get(REGULATE_RADIATOR).unwrap().as_ref().borrow();

    //  5*60
    let mut interval = interval(Duration::from_secs(5*60));
    loop {
        interval.tick().await;

        if let Ok(reg_plan) = get_current_regulation_map().await {
            info!("L'heure actuelle est entre {} et {}.", reg_plan.0, reg_plan.1);
            let msg = MessageEnum::RegulationMsg(reg_plan.2);

            info!("prepare to send :  [{:?}]", &msg);
            let _ = device.publish_message_topic(&mut client, &msg).await;
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
