use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::process::exit;

use anyhow::anyhow;
use ava_toolkit::device_message::RadiatorMode;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use chrono::Local;
use common_config::conf_reader::{read_config, read_env};
use common_config::properties::{get_prop_pg_connect_string, get_prop_value, set_prop_values};
use log::{error, info};
use reqwest::header;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::NoTls;

const RAD_SALON: &str = "external/rad_salon";
const RAD_BUREAU: &str = "external/rad_bureau";
const RAD_COULOIR: &str = "external/rad_couloir";
const RAD_CHAMBRE: &str = "external/rad_chambre";

const CURRENT_REGULATION_MAP_SQL: &str = r"SELECT id, starting_time, ending_time, end_the_next_day, boost, regulation_map::text AS regulation_map_json, ts_created
FROM public.heating_plan
WHERE boost = false
AND (
    (
        end_the_next_day = true
        AND (
            (starting_time <= $1)
            OR ($1 < ending_time)
        )
    )
    OR (
        end_the_next_day = false
        AND starting_time <= $1
        AND $1 < ending_time
    )
)
ORDER BY ts_created DESC
LIMIT 1";

const LATEST_RADIATOR_STATE_SQL: &str = r"SELECT DISTINCT ON (device_name) device_name, state, ts_create
FROM public.device_state_history
WHERE device_name LIKE 'external/rad_%'
ORDER BY device_name, ts_create DESC";

#[derive(Clone)]
struct AppState {
    db_url: String,
    heatzy_application_id: String,
    heatzy_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateRadiatorRequest {
    chambre: f64,
    salon: f64,
    couloir: f64,
    bureau: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateRadiatorResponse {
    updated_radiators: Vec<UpdatedRadiator>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdatedRadiator {
    radiator: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RadiatorState {
    mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegulationMap {
    tc_bureau: f32,
    tc_salon_1: f32,
    tc_salon_2: f32,
    tc_chambre_1: f32,
    tc_couloir: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum RadiatorAction {
    On,
    Off,
    NoAction,
}

#[tokio::main]
async fn main() {
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    info!("Starting AVA radiator-api 0.1.0");

    const PROJECT_CODE: &str = "radiator-api";
    const VAR_NAME: &str = "AVA_ENV";

    let o_config_file = read_env(VAR_NAME);

    info!(
        "Config file using PROJECT_CODE={} VAR_NAME={}",
        PROJECT_CODE, VAR_NAME
    );

    let props = read_config(
        PROJECT_CODE,
        &o_config_file,
        &Some("AVA_CLUSTER_PROFILE".to_string()),
    );
    set_prop_values(props);

    let port = read_props_or_die("server.port")
        .parse::<u16>()
        .unwrap_or(30055);
    let (db_url, _) = match get_prop_pg_connect_string() {
        Ok(v) => v,
        Err(e) => {
            error!("Cannot read database properties: {}", e);
            exit(-64);
        }
    };

    let app_state = AppState {
        db_url,
        heatzy_application_id: read_props_or_die("heatzy.application.id"),
        heatzy_token: read_props_or_die("heatzy.token"),
    };

    let app = Router::new()
        .route("/update-radiator", post(update_radiator))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("radiator-api listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn update_radiator(
    State(state): State<AppState>,
    Json(payload): Json<UpdateRadiatorRequest>,
) -> Result<Json<UpdateRadiatorResponse>, (StatusCode, String)> {
    let (client, connection) = tokio_postgres::connect(&state.db_url, NoTls)
        .await
        .map_err(internal_error)?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    let regulation_map = get_current_regulation_map(&client)
        .await
        .map_err(internal_error)?;

    let current_states = get_latest_radiator_states(&client)
        .await
        .map_err(internal_error)?;

    let room_temperatures = HashMap::from([
        (RAD_BUREAU.to_string(), payload.bureau),
        (RAD_CHAMBRE.to_string(), payload.chambre),
        (RAD_COULOIR.to_string(), payload.couloir),
        (RAD_SALON.to_string(), payload.salon),
    ]);

    let room_targets = HashMap::from([
        (RAD_BUREAU.to_string(), regulation_map.tc_bureau),
        (RAD_CHAMBRE.to_string(), regulation_map.tc_chambre_1),
        (RAD_COULOIR.to_string(), regulation_map.tc_couloir),
        (RAD_SALON.to_string(), regulation_map.tc_salon_1),
    ]);

    let did_by_radiator = HashMap::from([
        (RAD_SALON.to_string(), "3wHa7Ja50MhfShUxcmOqvT"),
        (RAD_COULOIR.to_string(), "JUVo7yMFQtdfZhi25Vo4Bu"),
        (RAD_CHAMBRE.to_string(), "LNENiFG0MeReR9WtxMebYB"),
        (RAD_BUREAU.to_string(), "mO7E2B49G1BS8R77UmWIjk"),
    ]);

    let mut updated_radiators = Vec::new();

    for radiator in [RAD_BUREAU, RAD_CHAMBRE, RAD_COULOIR, RAD_SALON] {
        let current_mode = current_states
            .get(radiator)
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Missing current state in DB for radiator [{}]", radiator),
                )
            })?
            .clone();

        if current_mode == RadiatorMode::ECO {
            info!("{} in ECO mode => no action", radiator);
            continue;
        }

        let t_current = *room_temperatures.get(radiator).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Missing temperature for radiator [{}]", radiator),
            )
        })?;

        let t_target = *room_targets.get(radiator).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Missing target temperature for radiator [{}]", radiator),
            )
        })?;

        let action = determine_action(t_current, t_target);
        let next_mode = match action {
            RadiatorAction::On => Some(RadiatorMode::CFT),
            RadiatorAction::Off => Some(RadiatorMode::STOP),
            RadiatorAction::NoAction => None,
        };

        if let Some(next_mode) = next_mode {
            if next_mode != current_mode {
                let did = did_by_radiator
                    .get(radiator)
                    .ok_or_else(|| internal_error(anyhow!("Missing DID mapping")))?;

                set_heatzy_mode(
                    &state.heatzy_application_id,
                    &state.heatzy_token,
                    did,
                    next_mode,
                )
                .await
                .map_err(internal_error)?;

                save_radiator_state(&client, radiator, next_mode)
                    .await
                    .map_err(internal_error)?;

                updated_radiators.push(UpdatedRadiator {
                    radiator: radiator.to_string(),
                    status: mode_as_str(next_mode).to_string(),
                });
            }
        }
    }

    Ok(Json(UpdateRadiatorResponse { updated_radiators }))
}

async fn get_current_regulation_map(
    client: &tokio_postgres::Client,
) -> anyhow::Result<RegulationMap> {
    let local_time = Local::now().time().format("%H:%M:%S").to_string();

    let row = client
        .query_opt(CURRENT_REGULATION_MAP_SQL, &[&local_time])
        .await?
        .ok_or_else(|| anyhow!("No current regulation map found"))?;

    let reg_json: String = row.try_get("regulation_map_json")?;
    let reg: Value = serde_json::from_str(&reg_json)?;
    let reg_map: RegulationMap = serde_json::from_value(reg)?;
    Ok(reg_map)
}

async fn get_latest_radiator_states(
    client: &tokio_postgres::Client,
) -> anyhow::Result<HashMap<String, RadiatorMode>> {
    let rows = client.query(LATEST_RADIATOR_STATE_SQL, &[]).await?;

    let mut states = HashMap::new();
    for row in rows {
        let device_name: String = row.try_get("device_name")?;
        let state_json: String = row.try_get("state")?;
        let state: RadiatorState = serde_json::from_str(&state_json)?;
        states.insert(device_name, mode_from_str(&state.mode)?);
    }

    Ok(states)
}

async fn save_radiator_state(
    client: &tokio_postgres::Client,
    radiator: &str,
    mode: RadiatorMode,
) -> anyhow::Result<()> {
    let state = serde_json::to_string(&RadiatorState {
        mode: mode_as_str(mode).to_string(),
    })?;

    let query = r#"INSERT INTO public.device_state_history (device_name, state, ts_create)
VALUES($1, $2, timezone('UTC', current_timestamp))"#;

    client.execute(query, &[&radiator, &state]).await?;
    Ok(())
}

async fn set_heatzy_mode(
    heatzy_application_id: &str,
    heatzy_token: &str,
    did: &str,
    mode: RadiatorMode,
) -> anyhow::Result<()> {
    let h_mode = match mode {
        RadiatorMode::CFT => 0,
        RadiatorMode::ECO => 1,
        RadiatorMode::FRO => 2,
        RadiatorMode::STOP => 3,
    };

    let data = serde_json::json!({
        "attrs": {
            "mode": h_mode
        }
    });

    let url = format!("https://euapi.gizwits.com/app/control/{}", did);

    let mut custom_header = header::HeaderMap::new();
    custom_header.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("reqwest"),
    );
    custom_header.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    custom_header.insert(
        "X-Gizwits-Application-Id",
        heatzy_application_id.parse().unwrap(),
    );
    custom_header.insert("X-Gizwits-User-token", heatzy_token.parse().unwrap());

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .headers(custom_header)
        .json(&data)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Heatzy error status: {}", response.status()))
    }
}

fn determine_action(t_current: f64, tc: f32) -> RadiatorAction {
    if t_current < tc as f64 - 0.3f64 {
        RadiatorAction::On
    } else if t_current > tc as f64 + 0.3f64 {
        RadiatorAction::Off
    } else {
        RadiatorAction::NoAction
    }
}

fn mode_from_str(mode: &str) -> anyhow::Result<RadiatorMode> {
    match mode {
        "CFT" => Ok(RadiatorMode::CFT),
        "ECO" => Ok(RadiatorMode::ECO),
        "FRO" => Ok(RadiatorMode::FRO),
        "STOP" => Ok(RadiatorMode::STOP),
        _ => Err(anyhow!("Unknown mode [{}]", mode)),
    }
}

fn mode_as_str(mode: RadiatorMode) -> &'static str {
    match mode {
        RadiatorMode::CFT => "CFT",
        RadiatorMode::ECO => "ECO",
        RadiatorMode::FRO => "FRO",
        RadiatorMode::STOP => "STOP",
    }
}

fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
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
