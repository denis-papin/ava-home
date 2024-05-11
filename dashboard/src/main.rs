#![feature(proc_macro_hygiene, decl_macro)]

use std::{fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::process::exit;
use std::sync::{Mutex, RwLock};
use std::time::SystemTime;
use anyhow::anyhow;

use chrono::{DateTime, Utc};
use lazy_static::*;
use log::*;
use rocket::*;
use rocket::{Request, request};
use rocket::config::Environment;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Method, Status};
use rocket::http::ext::IntoCollection;
use rocket::request::FromRequest;
use rocket_contrib::templates::{handlebars, Template};
use rs_uuid::iso::uuid_v4;
use serde_derive::*;

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

#[derive(Serialize)]
struct TemplateContext {
    message: String,
}

#[derive(Serialize)]
struct CssTemplateContext {
    size: u8,
    font_family: String,
}

#[derive(Debug)]
pub struct CORS;

impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        info!("On Response [{}]", &request );
        info!("On Response [{}]", &response.status() );

        let _ = response.status();
        // dbg!(&s);

        if request.method() == Method::Options {
            response.set_status(Status::Ok);
        }

        // TODO : response.adjoin_header(ContentType::JSON );
        response.adjoin_raw_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PATCH, DELETE");
        response.adjoin_raw_header("Access-Control-Allow-Origin", "*");
        response.adjoin_raw_header("Access-Control-Allow-Credentials", "true");
        response.adjoin_raw_header("Access-Control-Allow-Headers", "*");
    }
}

#[get("/index")]
fn index() -> Template {
    let mut context = match build_current_temp_context() {
        Ok(c) => c,
        Err(e) => panic!("{}", e)
    };

    context.insert("ws_hostname".to_string(), get_prop_value("ws.hostname"));
    context.insert("ws_port".to_string(), get_prop_value("ws.port"));

    let handlebars = handlebars::Handlebars::new();
    let template_str = include_str!("../templates/dashboard.hbs");

    let mut handlebars = handlebars::Handlebars::new();
    handlebars.register_template_string("dashboard",template_str).expect("Failed to register template");
    Template::render("dashboard", context)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub (crate) struct RadiatorStatus {
    pub mode: String
}

fn build_current_radiator_state_context(mut trans: &mut SQLTransaction) -> anyhow::Result<HashMap<String, String>> {
    let mut context = HashMap::new();
    let query = SQLQueryBlock {
        sql_query : r"select DISTINCT ON (device_name) device_name, state, ts_create
                    from device_state_history dsh
                    order by  device_name, ts_create DESC".to_string(),
        start : 0,
        length : None,
        params: HashMap::new(),
    };

    let mut sql_result = query.execute(&mut trans).map_err(err_fwd!("💣 Query failed, [{}], follower=[]", &query.sql_query/*, &self.follower*/))?;

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

fn build_current_temp_context() -> anyhow::Result<HashMap<String, String>> {
    let mut context = HashMap::new();

    let mut r_cnx = SQLConnection::new();
    // let-else
    let r_trans = open_transaction(&mut r_cnx).map_err(err_fwd!("💣 Open transaction error, follower=[]"/*, &self.follower*/));
    let Ok(mut trans) = r_trans else {
        return Err(anyhow!("💣 Impossible to connect the database")); // WebType::from_errorset(INTERNAL_DATABASE_ERROR);
    };

    // Ajoute le contexte pour le statut des radiateurs
    context.extend(build_current_radiator_state_context(&mut trans)?.into_iter());

    dbg!(&context);

    // let Ok((open_session_request, password_hash)) = self.search_user(&mut trans, &login_request.login) else {
    //     log_warn!("⛔ login not found, login=[{}], follower=[{}]", &login_request.login, &self.follower);
    //     return WebType::from_errorset(SESSION_LOGIN_DENIED);
    // };

    let mut params = HashMap::new();
    // params.insert("p_login".to_owned(), CellValue::from_raw_string(login.to_string()));

    let query = SQLQueryBlock {
        sql_query : r"SELECT DISTINCT ON (device_name) device_name, temperature, ts_create
        FROM temperature_sensor_history
        ORDER BY device_name, ts_create DESC".to_string(),
        start : 0,
        length : None,
        params,
    };

    let mut sql_result = query.execute(&mut trans).map_err(err_fwd!("💣 Query failed, [{}], follower=[]", &query.sql_query/*, &self.follower*/))?;

    dbg!(&sql_result);

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

    trans.commit()?;
    Ok(context)
}

#[get("/index_live")]
pub fn index_live() -> Template {
    let temps : Vec<TemperatureForSensor> = fetch_temperature();

    // Find a way to loop over the vec in the template
    let mut context = HashMap::new();
    context.insert("salon_temperature", "33,0");
    context.insert("salon_elapse", "7");
    context.insert("bureau_temperature", "23,2");
    context.insert("bureau_elapse", "30");
    context.insert("chambre_temperature", "21,4");
    context.insert("chambre_elapse", "12");
    Template::render("dashboard", &context)
}

#[get("/search_page")]
pub fn search_page() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Page d'accueil");
    context.insert("message", "Bienvenue sur notre site web!");

    Template::render("search", &context)
}

#[get("/input_page")]
pub fn input_page() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Page d'accueil");
    context.insert("message", "Bienvenue sur notre site web!");

    Template::render("input", &context)
}

#[get("/setup_page")]
pub fn setup_page() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Page d'accueil");
    context.insert("message", "Bienvenue sur notre site web!");

    Template::render("setup", &context)
}

#[get("/resort_info")]
fn resort_info() -> Template {
    // Création des données à afficher dans le template
    let mut context = HashMap::new();
    context.insert("componentName", "resort_info");
    context.insert("address", "35, rue du grand canyon");
    // Rendu du template avec les données
    Template::render("resort_info", &context)
}

#[get("/info_bar")]
fn info_bar() -> Template {
    // Création des données à afficher dans le template
    let mut context = HashMap::new();
    context.insert("componentName", "info_bar");
    context.insert("address", "35, rue du grand canyon");
    // Rendu du template avec les données
    Template::render("info_bar", &context)
}

use rocket::response::content::{Css, JavaScript};

/// An example of how to generate a js structure with some dynamic data in it
#[get("/data.js")]
fn script() -> JavaScript<String> {
    let context = TemplateContext {
        message: String::from("Hello, world!"),
    };
    let handlebars = handlebars::Handlebars::new();
    let template_str = include_str!("data/local_data.js");
    let js_code = handlebars.render_template(template_str, &context).unwrap();
    JavaScript(js_code)
}

#[get("/style.css")]
fn style() -> Css<String> {
    let context = CssTemplateContext {
        size: 2,
        font_family: "Helvetica Neue,Helvetica,Arial,sans-serif".to_string(),
    };
    //let js_code = generate_js("style.css", &context);
    let handlebars = handlebars::Handlebars::new();
    let template_str = include_str!("dyn_css/style.css");
    let js_code = handlebars.render_template(template_str, &context).unwrap();
    Css(js_code)
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TokenId(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for TokenId {
    type Error = ();
    fn from_request(my_request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let map = my_request.headers();
        let token = map.get_one("token_id").unwrap();
        request::Outcome::Success(TokenId(token.to_string()))
    }
}

fn create_token(username : &str) -> String {
    format!("{}:{}", uuid_v4(), username)
}

fn parse_token(token_str : &str) -> (String, String) {
    let parts = token_str.split(":").collect::<Vec<&str>>();
    (parts.get(0).unwrap().to_string(), parts.get(1).unwrap().to_string())
}



///
/// Set properties[0] with the configuration file values
///
fn set_props(props : HashMap<String, String>) {
    let mut w = PROPERTIES.write().unwrap();
    let item = w.get_mut(&0).unwrap();
    *item = Box::leak(Box::new(props ));
}

///
/// Build the secret file name from the username
///
fn get_secret_file_name(username : &str) -> String {
    let folder = get_secret_folder();
    let current_filename = format!("{}.crypt", &username);
    let path = Path::new(&folder).join(current_filename);
    path.into_os_string().into_string().unwrap()
}

///
/// Get the secret folder name
///
fn get_secret_folder() -> String {
    let folder = get_prop_value("app.secret-folder");
    folder
}

///
/// Get prop value from the application.properties file
///
fn get_prop_value(prop_name : &str) -> String {
    // https://doc.rust-lang.org/std/sync/struct.RwLock.html
    PROPERTIES.read().unwrap().deref().get(&0).unwrap().deref()
        .get(prop_name).unwrap().to_owned()
}

///
/// Change an entry in the properties
///
fn set_prop_value(prop_name : &str, value : &str ) -> () {

    info!("Define property [{prop_name}], value : [{value}]");

    if let Ok(write_guard) = PROPERTIES.write().as_mut() {
        // the returned write_guard implements `Deref` giving us easy access to the target value
        let map = write_guard.deref_mut();
        if  let Some( item ) = map.get_mut(&0) {
            item.insert(prop_name.to_string(), value.to_string());
        }
    }
    ()
}



///
/// Keep the n more recent file of the <folder> and that starts with prefix.
///
fn keep_newest_files(folder: &str, prefix: &str, n: usize) {
    let mut files: Vec<_> = fs::read_dir(folder)
        .expect("Erreur lors de la lecture du dossier")
        .filter_map(|entry| {
            let entry = entry.expect("Erreur lors de la lecture du fichier");
            let file_path = entry.path();
            let file_name = file_path.file_name()?.to_string_lossy().into_owned();
            if file_name.starts_with(prefix) {
                Some((file_path, entry.metadata().unwrap().modified().unwrap()))
            } else {
                None
            }
        })
        .collect();

    // dbg!(&files);

    if files.len() > n {
        files.sort_by(|(_, time1), (_, time2)| time2.cmp(time1));
        // dbg!(&files);

        let to_be_deleted: Vec<_> = files.drain(n..).collect();
        // files.drain(n..);

        for (file_path, _) in to_be_deleted {
            // dbg!(&file_path);
            fs::remove_file(file_path).expect("Erreur lors de la suppression du fichier");
        }
    }
}

use rocket_contrib::serve::StaticFiles;
use commons_error::*;
use commons_pg::{init_db_pool, SQLConnection, SQLQueryBlock, SQLTransaction};
use crate::dao_db::{fetch_temperature, open_transaction, TemperatureForSensor};

pub fn get_prop_pg_connect_string() -> anyhow::Result<(String,u32)> {
    let db_hostname = get_prop_value("db.hostname");
    let db_port = get_prop_value("db.port");
    let db_name = get_prop_value("db.name");
    let db_user = get_prop_value("db.user");
    let db_password = get_prop_value("db.password");
    let db_pool_size = get_prop_value("db.pool_size").parse::<u32>().map_err(err_fwd!("Cannot read the pool size"))?;
    let cs = format!("host={} port={} dbname={} user={} password={}", db_hostname, db_port, db_name, db_user,db_password);
    Ok((cs, db_pool_size))
}

fn main() {
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

    info!("🚀 Start {}", PROGRAM_NAME);

    // Init DB pool
    let (connect_string, db_pool_size) = match get_prop_pg_connect_string()
        .map_err(err_fwd!("Cannot read the database connection information")) {
        Ok(x) => x,
        Err(e) => {
            log_error!("{:?}", e);
            exit(-64);
        }
    };

    dbg!(&connect_string, &db_pool_size);
    init_db_pool(&connect_string, db_pool_size);

    let mut my_config = Config::new(Environment::Production);
    my_config.set_port(port);

    let base_url = format!("/{}", PROJECT_CODE);
    let base_static_url = format!("/{}/static", PROJECT_CODE);

    rocket::custom(my_config)
        .mount(&base_static_url, StaticFiles::from("static"))
        .mount(&base_url, routes![
            index, index_live, search_page, input_page, style, script, resort_info, info_bar])
        .attach(CORS)
        .attach(Template::fairing())
        .launch();

    info!("🏁 End {}", PROGRAM_NAME);
}


