
use std::env;
use std::path::Path;
use std::process::exit;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

use java_properties::read;
use log::info;

///
/// Read the configuration file whose path is stored in an environment variable.
/// The config file must be in a java properties file format.
///
pub fn read_config( project_code : &str, var_name : &str ) -> HashMap<String, String> {

    println!("Read the conf");

    let doka_env = match env::var(var_name) {
        Ok(env) => env,
        Err(e) => {
            eprintln!("💣 Cannot find the [{}] system variable, {}", &var_name, e);
            exit(-99);
        },
    };

    let config_path = Path::new(&doka_env).join(project_code).join("config/application.properties");

    println!("Read the config file at [{}]", config_path.to_str().unwrap());

    let f = match File::open(&config_path) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("💣 Cannot find the configuration file, e={}", e);
            exit(-89);
        }
    };

    let props = match read(BufReader::new(f)) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("💣 Cannot read the configuration file, e={}", e);
            exit(-79);
        }
    };

    eprintln!("Configuration file : props={:?}", &props);
    props
}