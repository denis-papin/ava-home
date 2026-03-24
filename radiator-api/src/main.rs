use std::env;
use std::net::SocketAddr;
use std::process::exit;

use axum::routing::post;
use axum::Router;
use common_config::conf_reader::{read_config, read_env};
use common_config::properties::{get_prop_pg_connect_string, get_prop_value, set_prop_values};
use log::{error, info};

mod api;

#[tokio::main]
async fn main() {
    // 1) Logging initialization.
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    info!("Starting AVA radiator-api 0.1.0");

    const PROJECT_CODE: &str = "radiator-api";
    const VAR_NAME: &str = "AVA_ENV";

    // 2) Load service configuration from AVA config system.
    let o_config_file = read_env(VAR_NAME);
    let props = read_config(
        PROJECT_CODE,
        &o_config_file,
        &Some("AVA_CLUSTER_PROFILE".to_string()),
    );
    set_prop_values(props);

    let port = read_props_or_die("server.port")
        .parse::<u16>()
        .unwrap_or(30055);

    // 3) Build shared app state consumed by request handlers.
    let (db_url, _) = match get_prop_pg_connect_string() {
        Ok(v) => v,
        Err(e) => {
            error!("Cannot read database properties: {}", e);
            exit(-64);
        }
    };

    let app_state = api::AppState {
        db_url,
        heatzy_application_id: read_props_or_die("heatzy.application.id"),
        heatzy_token: read_props_or_die("heatzy.token"),
    };

    // 4) Start HTTP server with a single business endpoint.
    let app = Router::new()
        .route("/update-radiator", post(api::update_radiator))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("radiator-api listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn read_props_or_die(property_name: &str) -> String {
    match get_prop_value(property_name) {
        Ok(value) => value,
        Err(e) => {
            error!("{}", e);
            panic!("Cannot find property [{}]", property_name);
        }
    }
}
