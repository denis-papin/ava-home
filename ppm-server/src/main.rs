#![feature(proc_macro_hygiene, decl_macro)]

use std::{fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::process::exit;
use std::sync::{Mutex, RwLock};

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


///
/// ✨  Main component...
///
// Fonction pour gérer la page d'accueil
#[get("/index")]
fn index() -> Template {
    // Création des données à afficher dans le template
    let mut context = HashMap::new();
    context.insert("title", "Page d'accueil");
    context.insert("message", "Bienvenue sur notre site web!");

    // Rendu du template avec les données
    Template::render("index", &context)
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

fn main() {
    const PROGRAM_NAME: &str = "PPM Pretty Password Manager";

    println!("😎 Init {}", PROGRAM_NAME);

    const PROJECT_CODE: &str = "ppm";
    const VAR_NAME: &str = "PPM_ENV";

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

    let mut my_config = Config::new(Environment::Production);
    my_config.set_port(port);

    let base_url = format!("/{}", PROJECT_CODE);
    let base_static_url = format!("/{}/static", PROJECT_CODE);

    rocket::custom(my_config)
        .mount(&base_static_url, StaticFiles::from("static"))
        .mount(&base_url, routes![
            index, search_page, input_page, setup_page, style, script, resort_info, info_bar])
        .attach(CORS)
        .attach(Template::fairing())
        .launch();

    info!("🏁 End {}", PROGRAM_NAME);
}


