use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;
use std::sync::RwLock;
use axum::http::Method;

use axum::response::Html;
use axum::Router;
use axum::routing::get;
use lazy_static::*;
use log::*;
use serde_derive::*;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use commons_error::*;
use commons_pg::sql_transaction2::init_db_pool2;
use conf_reader::*;
use crate::dao::get_current_regulation_map;

use crate::dao_db::build_current_temp_context;

mod conf_reader;
mod dao_db;
mod dao;
mod device_message;

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

lazy_static! {
    static ref FILE_LOCKER: Mutex<u16> = Mutex::new(0);
}

// #[get("/index")]
async fn index() -> Html<String> {
    let mut context = match build_current_temp_context().await {
        Ok(c) => c,
        Err(e) => panic!("{}", e)
    };

    context.insert("ws_hostname".to_string(), get_prop_value("ws.hostname"));
    context.insert("ws_port".to_string(), get_prop_value("ws.port"));

    if let Ok(reg_map) = get_current_regulation_map().await {
        context.insert("tc_salon".to_string(), reg_map.2.tc_salon_1.to_string());
        context.insert("tc_bureau".to_string(), reg_map.2.tc_bureau.to_string());
        context.insert("tc_chambre".to_string(), reg_map.2.tc_chambre_1.to_string());
        context.insert("tc_couloir".to_string(), reg_map.2.tc_couloir.to_string());
    }

    let template_str = include_str!("../templates/dashboard.hbs");

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.register_template_string("dashboard",template_str).expect("Failed to register template");

    // Template::render("dashboard", context)
    Html( handlebars
              .render("dashboard", & context)
              .expect("Failed to render template"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenId(pub String);

///
/// Set properties[0] with the configuration file values
///
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
    const PROGRAM_NAME: &str = "DASH - Tha Ava Home Dashboard";
    println!("😎 Init {}", PROGRAM_NAME);

    const PROJECT_CODE: &str = "dashboard";
    const VAR_NAME: &str = "DASH_ENV";
    println!("😎 Config file using PROJECT_CODE={} VAR_NAME={}", PROJECT_CODE, VAR_NAME);

    let props = read_config(PROJECT_CODE, VAR_NAME);
    set_props(props);
    let port = get_prop_value("server.port").parse::<u16>().unwrap();
    let log_config: String = get_prop_value("log4rs.config");
    let log_config_path = Path::new(&log_config);

    println!("😎 Read log properties from {:?}", &log_config_path);

    match log4rs::init_file(&log_config_path, Default::default()) {
        Err(e) => {
            eprintln!("{:?} {:?}", &log_config_path, e);
            exit(-59);
        }
        Ok(_) => {}
    }

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

    log_info!("🚀 Start {} on port {}", PROGRAM_NAME, port);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH, Method::DELETE])
        .allow_origin(Any) // You can restrict origins instead of using Any
        .allow_headers(Any)
        //.allow_credentials(true)
       ;

    // Build our application with some routes
    let base_url = format!("/{}", PROJECT_CODE);
    let key_routes = Router::new()
        .route("/index", get(index))
        .nest_service("/static", ServeDir::new("static"))
        .layer(cors);

    let app = Router::new().nest(&base_url, key_routes);

    // let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    log_info!("🏁 End {}", PROGRAM_NAME);
}


