use axum::extract::Path as AxumPath;
use axum::extract::Query;
use axum::http::{Method, StatusCode};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::path::Path as FsPath;
use std::process::exit;

use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use common_config::conf_reader::{read_config, read_env};
use common_config::properties::{
    get_prop_pg_connect_string, get_prop_value, set_prop_values,
};
use log::{error, info};
use tower_http::cors::{Any, CorsLayer};

use crate::clairdelune_api::{
    get_heating_plan, get_heating_plan_by_room, get_room_temperature_by_mode,
    RoomTemperatureByModeQuery,
};
use crate::dao::get_current_regulation_map;
use crate::dao_db::RadiatorStatus;
use commons_pg::sql_transaction2::init_db_pool2;

use crate::dao_db::build_current_temp_context;

mod clairdelune_api;
mod dao;
mod dao_db;

async fn index2_data() -> Json<HashMap<String, String>> {
    Json(build_dashboard_context().await)
}

async fn heating_plan() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload = get_heating_plan().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unable to load heating_plan from PostgreSQL: {}", e),
        )
    })?;

    Ok(Json(serde_json::to_value(payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unable to serialize heating_plan response: {}", e),
        )
    })?))
}

async fn heating_plan_by_room() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload = get_heating_plan_by_room().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unable to load heating_plan from PostgreSQL: {}", e),
        )
    })?;

    Ok(Json(serde_json::to_value(payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unable to serialize heating_plan_by_room response: {}", e),
        )
    })?))
}

async fn room_temperature_by_mode(
    Query(params): Query<RoomTemperatureByModeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let payload = get_room_temperature_by_mode(&params).await.map_err(|e| {
        let message = e.to_string();
        let status = if message.contains("Unknown room")
            || message.contains("valid ISO")
            || message.contains("earlier than")
        {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, message)
    })?;

    Ok(Json(serde_json::to_value(payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Unable to serialize room_temperature_by_mode response: {}",
                e
            ),
        )
    })?))
}

async fn index2_radiator(
    AxumPath(room): AxumPath<String>,
    Json(payload): Json<RadiatorStatus>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let base_url = get_optional_prop_value("radiator-api.base-url")
        .unwrap_or_else(|| "http://127.0.0.1:30055".to_string());
    let url = format!("{}/radiator/{}", base_url.trim_end_matches('/'), room);

    let client = reqwest::Client::new();
    let response = client.post(url).json(&payload).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Cannot reach radiator-api: {}", e),
        )
    })?;

    let status = response.status();
    let response_text = response.text().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Cannot read radiator-api response: {}", e),
        )
    })?;

    if !status.is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("radiator-api returned {}: {}", status, response_text),
        ));
    }

    let json_body: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Invalid JSON returned by radiator-api: {}", e),
        )
    })?;

    Ok(Json(json_body))
}

fn get_optional_prop_value(prop_name: &str) -> Option<String> {
    get_prop_value(prop_name).ok()
}

async fn build_dashboard_context() -> HashMap<String, String> {
    let mut context = match build_current_temp_context().await {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };

    if let Ok(reg_map) = get_current_regulation_map().await {
        context.insert("tc_salon".to_string(), reg_map.2.tc_salon_1.to_string());
        context.insert("tc_bureau".to_string(), reg_map.2.tc_bureau.to_string());
        context.insert("tc_chambre".to_string(), reg_map.2.tc_chambre_1.to_string());
        context.insert("tc_couloir".to_string(), reg_map.2.tc_couloir.to_string());
    }

    context
}

#[tokio::main]
async fn main() {
    const PROGRAM_NAME: &str = "DASH - The Ava Home Dashboard API";
    println!("😎 Init {}", PROGRAM_NAME);

    const PROJECT_CODE: &str = "dashboard-api";
    const VAR_NAME: &str = "AVA_ENV";
    println!(
        "😎 Config file using PROJECT_CODE={} VAR_NAME={}",
        PROJECT_CODE, VAR_NAME
    );

    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );

    let o_config_file = read_env(VAR_NAME);
    let props = read_config(
        PROJECT_CODE,
        &o_config_file,
        &Some("AVA_CLUSTER_PROFILE".to_string()),
    );
    set_prop_values(props);

    let port = get_prop_value("server.port")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let log_config = get_prop_value("log4rs.config").unwrap();
    let log_config_path = FsPath::new(&log_config);

    println!("😎 Read log properties from {:?}", &log_config_path);

    match log4rs::init_file(&log_config_path, Default::default()) {
        Err(e) => {
            eprintln!("{:?} {:?}", &log_config_path, e);
            exit(-59);
        }
        Ok(_) => {}
    }

    // Init DB pool
    let (connect_string, db_pool_size) = match get_prop_pg_connect_string() {
        Ok(x) => x,
        Err(e) => {
            error!("{:?}", e);
            exit(-64);
        }
    };

    info!("Connnect String : [{}]", &connect_string);

    let _r = init_db_pool2(&connect_string, db_pool_size).await;

    info!("🚀 Start {} on port {}", PROGRAM_NAME, port);

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH, Method::DELETE])
        .allow_origin(Any) // You can restrict origins instead of using Any
        .allow_headers(Any)
        //.allow_credentials(true)
       ;

    // Build our application with some routes
    let base_url = format!("/{}", PROJECT_CODE);
    let key_routes = Router::new()
        .route("/index/data", get(index2_data))
        .route("/index/radiator/:room", post(index2_radiator))
        .route("/index2/data", get(index2_data))
        .route("/index2/radiator/:room", post(index2_radiator))
        .route("/heating_plan", get(heating_plan))
        .route("/heating_plan_by_room", get(heating_plan_by_room))
        .route("/room_temperature_by_mode", get(room_temperature_by_mode))
        .layer(cors);

    let app = Router::new().nest(&base_url, key_routes);

    // let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    info!("🏁 End {}", PROGRAM_NAME);
}
