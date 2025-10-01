use std::cell::RefCell;
use std::collections::HashMap;
use std::{env, fs};
use std::sync::Arc;
use std::time::Duration;

use log::info;
use mlua::{IntoLua, Lua, UserData, Value};
use mlua::prelude::{LuaFunction, LuaResult};
use rumqttc::v5::{AsyncClient, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;

use crate::device_repo::{build_device_repo, device_to_listen};
use crate::generic_device::GenericDevice;
use crate::init_loop::{build_init_list, process_initialization_message};
use crate::loops::{build_loops, HardLoop};
use crate::processing::process_incoming_message;

mod loops;
mod device_repo;
mod init_loop;
mod processing;
mod message_enum;
mod generic_device;

const CLIENT_ID: &str = "ava-0.5.0";

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
        server_addr : "raspberrypi.local".to_string(),
        client_id,
        channel_filters,
        keep_alive : 30_000,
    }
}

impl UserData for HardLoop {
    fn add_fields<'lua, F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("device_count", |_, this| Ok(this.devices.len()));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("list_devices", |lua, this, _: ()| {
            let tbl = lua.create_table()?;
            for (i, dev) in this.devices.iter().enumerate() {
                tbl.set(i + 1, dev.borrow().name.clone())?;
            }
            Ok(tbl)
        });
    }
}

async fn call_lua(all_loops: &Vec<HardLoop>) -> LuaResult<()> {
    // VM Lua en mode async
    let lua = Lua::new();

    // Charger le script Lua (en synchrone, mais tu peux le faire en async avec tokio::fs si tu veux)
    let script = fs::read_to_string("/home/denis/Projects/wks-ava-home/ava-home/mqtt5-r/regulator.lua")
        .expect("Impossible de lire regulator.lua");

    lua.load(&script).exec_async().await?;

    let map_table = lua.create_table()?;
    map_table.set(1, "one")?;
    map_table.set("two", 2)?;
    lua.globals().set("map_table", map_table)?;

    lua.globals().set("all_loops", all_loops.clone())?;

    // Récupérer la fonction Lua
    let func: LuaFunction = lua.globals().get("process_map")?;

    // Appeler async
    func.call_async::<_>(()).await?;

    Ok(())
}

#[tokio::main]
async fn main() {

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting AVA 0.5.0");

    info!("Building the device repository");
    let device_repo = build_device_repo();
    let params = parse_params(&device_repo);


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

    // Test Lua call
    let r = call_lua(&all_loops).await;
    info!("Lua Script result {:?}", r);
    //

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



