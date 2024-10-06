use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::{Deref};
use std::path::Path;
use std::process::exit;
use std::sync::RwLock;

use anyhow::anyhow;
use axum::response::Html;
use axum::Router;
use axum::routing::get;
use chrono::Utc;
use lazy_static::*;
use log::*;
use serde_derive::*;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

use commons_error::*;
use commons_pg::sql_transaction2::{init_db_pool2, SQLConnection2, SQLQueryBlock2, SQLTransaction2};
use conf_reader::*;

mod conf_reader;
mod all_tests;
mod dao_db;

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


// #[derive(Debug)]
// pub struct CORS;
//
// impl Fairing for CORS {
//     fn info(&self) -> Info {
//         Info {
//             name: "Add CORS headers to responses",
//             kind: Kind::Response
//         }
//     }
//
//     fn on_response(&self, request: &Request, response: &mut Response) {
//         info!("On Response [{}]", &request );
//         info!("On Response [{}]", &response.status() );
//
//         let _ = response.status();
//         // dbg!(&s);
//
//         if request.method() == Method::Options {
//             response.set_status(Status::Ok);
//         }
//
//         // TODO : response.adjoin_header(ContentType::JSON );
//         response.adjoin_raw_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PATCH, DELETE");
//         response.adjoin_raw_header("Access-Control-Allow-Origin", "*");
//         response.adjoin_raw_header("Access-Control-Allow-Credentials", "true");
//         response.adjoin_raw_header("Access-Control-Allow-Headers", "*");
//     }
// }

// async fn toto() -> Html<String> {
//
//     log_info!("Start Toto");
//
//     let mut handlebars = Handlebars::new();
//     let template_str = include_str!("../templates/dashboard.hbs");
//
//     log_info!("Handlebar created");
//
//     handlebars
//         .register_template_string("dashboard", template_str)
//         .expect("Failed to register template");
//
//     log_info!("Template registered");
//
//     // Créer un contexte à l'aide de `serde_json::Map`
//     let mut context = HashMap::new();
//
//     // Insertion des valeurs dans le contexte
//     context.insert("ws_hostname".to_string(), "AAAA".to_string());
//     context.insert("ws_port".to_string(), "BBBB".to_string());
//
//     log_info!("Feed context");
//
//     // Utiliser le moteur Handlebars du state
//     let rendered =
//         handlebars
//         .render("dashboard", & context)
//         .expect("Failed to render template");
//
//     log_info!("-New content : {}", rendered);
//
//     Html(rendered)
// }


// #[get("/index")]
async fn index() -> Html<String> {
    let mut context = match build_current_temp_context().await {
        Ok(c) => c,
        Err(e) => panic!("{}", e)
    };

    context.insert("ws_hostname".to_string(), get_prop_value("ws.hostname"));
    context.insert("ws_port".to_string(), get_prop_value("ws.port"));

    let template_str = include_str!("../templates/dashboard.hbs");

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.register_template_string("dashboard",template_str).expect("Failed to register template");

    // Template::render("dashboard", context)
    Html( handlebars
              .render("dashboard", & context)
              .expect("Failed to render template"))
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub (crate) struct RadiatorStatus {
    pub mode: String
}

async fn build_current_radiator_state_context(mut trans: &mut SQLTransaction2<'_>) -> anyhow::Result<HashMap<String, String>> {
    let mut context = HashMap::new();
    let query = SQLQueryBlock2 {
        sql_query : r"select DISTINCT ON (device_name) device_name, state, ts_create
                    from device_state_history dsh
                    order by  device_name, ts_create DESC".to_string(),
        start : 0,
        length : None,
        params: HashMap::new(),
    };

    let r_sql_result = query.execute(&mut trans).await;
    let mut sql_result = r_sql_result.map_err(err_fwd!("💣 Query failed, [{}], follower=[]", &query.sql_query/*, &self.follower*/))?;

    while sql_result.next() {
        let device_name : String = sql_result.get_string("device_name").ok_or(anyhow!("Wrong device_name"))?;
        let json_state: String = sql_result.get_string("state").ok_or(anyhow!("Wrong state"))?;
        //let _ts_create = sql_result.get_timestamp_as_datetime("ts_create").ok_or(anyhow!("Wrong ts_create"))?;

        let status : RadiatorStatus = serde_json::from_str(json_state.as_str()).unwrap();

        match device_name.as_str() {
            "external/rad_bureau" => {
                context.insert("bureau_status".to_string(), status.mode);
            }
            "external/rad_chambre" => {
                context.insert("chambre_status".to_string(), status.mode);
            }
            "external/rad_couloir" => {
                context.insert("couloir_status".to_string(), status.mode);
            }
            "external/rad_salon" => {
                context.insert("salon_status".to_string(), status.mode);
            }
            _ => {}
        }
    }
    Ok(context)
}

async fn build_current_temp_context() -> anyhow::Result<HashMap<String, String>> {
    let mut context = HashMap::new();

    let mut cnx = SQLConnection2::from_pool().await.map_err(tr_fwd!())?;
    let mut trans = cnx.begin().await.map_err(tr_fwd!())?;

    // Ajoute le contexte pour le statut des radiateurs
    context.extend(build_current_radiator_state_context(&mut trans).await?.into_iter());

    let params = HashMap::new();

    let query = SQLQueryBlock2 {
        sql_query : r"SELECT DISTINCT ON (device_name) device_name, temperature, ts_create
        FROM temperature_sensor_history
        ORDER BY device_name, ts_create DESC".to_string(),
        start : 0,
        length : None,
        params,
    };

    let mut sql_result = query.execute(&mut trans).await.map_err(err_fwd!("💣 Query failed, [{}], follower=[]", &query.sql_query/*, &self.follower*/))?;

    while sql_result.next() {
        let device_name : String = sql_result.get_string("device_name").ok_or(anyhow!("Wrong device_name"))?;
        let temperature : f64 = sql_result.get_double("temperature").ok_or(anyhow!("Wrong temperature"))?;
        let ts_create = sql_result.get_timestamp_as_datetime("ts_create").ok_or(anyhow!("Wrong ts_create"))?;

        let elapse_time_min = Utc::now().signed_duration_since(ts_create).num_minutes().abs();
        let elapse_time_hour = elapse_time_min / 60;
        let remain_time_min = elapse_time_min % 60;
        let elapse_time_string = if elapse_time_hour > 0 {
            format!("{} h {}", elapse_time_hour, remain_time_min)
        } else {
            format!("{}", elapse_time_min)
        };

        let temperature_string = format!("{:.1}", temperature).replace(".", ",");
        match device_name.as_str() {
            "zigbee2mqtt/ts_bureau" => {
                // current_temp.insert("bureau".to_string(), temperature);
                context.insert("bureau_temperature".to_string(), temperature_string);
                context.insert("bureau_elapse".to_string(), elapse_time_string);
            }
            "zigbee2mqtt/ts_chambre_1" => {
                context.insert("chambre_temperature".to_string(), temperature_string);
                context.insert("chambre_elapse".to_string(), elapse_time_string);
            }
            "zigbee2mqtt/ts_couloir" => {
                context.insert("couloir_temperature".to_string(), temperature_string);
                context.insert("couloir_elapse".to_string(),elapse_time_string);
            }
            "zigbee2mqtt/ts_salon_1" => {
                context.insert("salon_temperature".to_string(), temperature_string);
                context.insert("salon_elapse".to_string(), elapse_time_string);
            }
            "zigbee2mqtt/ts_salon_2" => {
                // nothing to do
            }
            _ => {}
        }
    }

    let _r = trans.commit().await?;

    Ok(context)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenId(pub String);

// impl<'a, 'r> FromRequest<'a, 'r> for TokenId {
//     type Error = ();
//     fn from_request(my_request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
//         let map = my_request.headers();
//         let token = map.get_one("token_id").unwrap();
//         request::Outcome::Success(TokenId(token.to_string()))
//     }
// }

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

    // Build our application with some routes
    let base_url = format!("/{}", PROJECT_CODE);
    let key_routes = Router::new()
        .route("/index", get(index))
        .nest_service("/static", ServeDir::new("static"));

    let app = Router::new().nest(&base_url, key_routes);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    log_info!("🏁 End {}", PROGRAM_NAME);
}


