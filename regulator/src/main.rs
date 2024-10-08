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
mod properties;
mod external_computing;
mod db_last_message;

const CLIENT_ID: &str = "ava-regulator";

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


// async fn test_db() {
//
//     use tokio_postgres::{NoTls, types::ToSql};
//     // Remplacez ces valeurs par les informations de votre base de données
//     let db_url = "postgresql://denis:dentece3.X@192.168.0.149/avahome";
//
//     // Établissez une connexion à la base de données
//     let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();
//
//     // Spawn une tâche pour gérer la processus de connexion en arrière-plan
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("Connection error: {}", e);
//         }
//     });
//
//     // Exécutez une requête
//     let rows = client
//         //query("SELECT $1::TEXT", &[&"hello".to_owned() as &(dyn ToSql + Sync)])
//         .query("select device_name from temperature_sensor_history order by ts_create desc", &[])
//         .await
//         .unwrap();
//
//     // Affiche les résultats de la requête
//     for row in rows {
//         let value: &str = row.get(0);
//         println!("Value: {}", value);
//     }
// }

#[tokio::main]
async fn main() {

    // run --package regulator --bin regulator -- <heatzy_pass>   <heatzy_key>

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let args: Vec<String> = vec![];

    // let args: Vec<String> = env::args().collect();
    // let heatzy_pass = args.get(1).unwrap();
    // let heatzy_application_id= args.get(2).unwrap();
    // dbg!(heatzy_pass);
    // dbg!(heatzy_application_id);

    info!("Starting AVA regulator 0.5.0");

    // Database
    // let db_hostname = "192.168.0.149";
    // let db_port = 5432;
    // let db_name = "avahome";
    // let db_user = "denis";
    // let db_password = "dentece3.X";
    // let db_pool_size = 5;
    // let connect_string = format!("host={} port={} dbname={} user={} password={}", db_hostname, db_port, db_name, db_user,db_password);

    // Current mode for the radiators : <radiator_name, mode>

    // set_prop_value("salon", "???");
    // set_prop_value("couloir", "???");
    // set_prop_value("chambre_1", "???");
    // set_prop_value("bureau", "???");
    //
    // set_prop_value("check_radiator_mode_counter", "0");

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
            let _ = process_incoming_message(&mut client, &mut eventloop, &mut all_loops, &args).await;
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    println!("Done!");
}



