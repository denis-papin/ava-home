use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use ava_toolkit::device_message::RadiatorMode;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{Local, NaiveTime};
use common_config::properties::set_prop_value;
use log::{error, info};
use radiator_toolkit::HeatzyClient;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::NoTls;

const RAD_SALON: &str = "external/rad_salon";
const RAD_BUREAU: &str = "external/rad_bureau";
const RAD_COULOIR: &str = "external/rad_couloir";
const RAD_CHAMBRE: &str = "external/rad_chambre";
const TS_SALON: &str = "homey/ts_salon_1";
const TS_BUREAU: &str = "homey/ts_bureau";
const TS_COULOIR: &str = "homey/ts_couloir";
const TS_CHAMBRE: &str = "homey/ts_chambre_1";

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

const SUPPORTED_ROOMS: &str = "bureau, chambre, couloir, salon";
const SUPPORTED_DIRECT_MODES: &str = "CFT, STOP, ECO";

#[derive(Clone, Copy, Debug)]
struct RadiatorConfig {
    room: &'static str,
    radiator: &'static str,
    did: &'static str,
}

const RADIATORS: [RadiatorConfig; 4] = [
    RadiatorConfig {
        room: "bureau",
        radiator: "external/rad_bureau",
        did: "mO7E2B49G1BS8R77UmWIjk",
    },
    RadiatorConfig {
        room: "chambre",
        radiator: "external/rad_chambre",
        did: "LNENiFG0MeReR9WtxMebYB",
    },
    RadiatorConfig {
        room: "couloir",
        radiator: "external/rad_couloir",
        did: "JUVo7yMFQtdfZhi25Vo4Bu",
    },
    RadiatorConfig {
        room: "salon",
        radiator: "external/rad_salon",
        did: "3wHa7Ja50MhfShUxcmOqvT",
    },
];

/// Shared application state injected in Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
    pub heatzy_application_id: String,
    pub heatzy_token: Arc<RwLock<String>>,
    pub heatzy_username: String,
    pub heatzy_password: String,
}

/// Payload expected by POST /update-radiator.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRadiatorRequest {
    pub chambre: f64,
    pub salon: f64,
    pub couloir: f64,
    pub bureau: f64,
}

/// API response with only the radiators that were actually changed.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRadiatorResponse {
    pub updated_radiators: Vec<UpdatedRadiator>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatedRadiator {
    pub radiator: String,
    pub status: String,
}

/// Payload expected by POST /radiator/:room.
#[derive(Debug, Serialize, Deserialize)]
pub struct SetRadiatorModeRequest {
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SetRadiatorModeResponse {
    pub radiator: String,
    pub status: String,
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

/// Decision returned by temperature control logic.
#[derive(Debug, Clone, PartialEq)]
enum RadiatorAction {
    On,
    Off,
    NoAction,
}

/// Direct command endpoint: set one radiator mode without using temperature-based computation.
pub async fn set_radiator_mode(
    Path(room): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<SetRadiatorModeRequest>,
) -> Result<Json<SetRadiatorModeResponse>, (StatusCode, String)> {
    info!(
        "🚀 Process direct radiator mode request for room [{}] and mode [{}]",
        room, payload.mode
    );

    let radiator = find_radiator_by_room(&room)?;
    let requested_mode = parse_direct_mode(&payload.mode)?;

    let (client, _connection) = open_db_connection(&state).await.map_err(internal_error)?;
    let current_states = get_latest_radiator_states(&client)
        .await
        .map_err(internal_error)?;

    if current_states.get(radiator.radiator) == Some(&requested_mode) {
        info!(
            "\t✅ Radiator {} already in direct mode [{}], no update needed",
            radiator.radiator,
            mode_as_str(requested_mode)
        );
        return Ok(Json(SetRadiatorModeResponse {
            radiator: radiator.radiator.to_string(),
            status: mode_as_str(requested_mode).to_string(),
        }));
    }

    apply_radiator_mode_change(&client, &state, radiator, requested_mode)
        .await
        .map_err(internal_error)?;

    Ok(Json(SetRadiatorModeResponse {
        radiator: radiator.radiator.to_string(),
        status: mode_as_str(requested_mode).to_string(),
    }))
}

/// Main endpoint: compute required changes, call Heatzy, then persist state changes.
pub async fn update_radiator(
    State(state): State<AppState>,
    Json(payload): Json<UpdateRadiatorRequest>,
) -> Result<Json<UpdateRadiatorResponse>, (StatusCode, String)> {
    info!("🚀 Process radiator update request: {:?}", payload);

    // 1) Open a dedicated DB connection for this request.
    let (client, _connection) = open_db_connection(&state).await.map_err(internal_error)?;

    save_input_temperatures(&client, &payload)
        .await
        .map_err(internal_error)?;
    info!("📝 Stored incoming temperatures into temperature_sensor_history");

    // 2) Load target temperatures from the currently active heating plan.
    let regulation_map = get_current_regulation_map(&client)
        .await
        .map_err(internal_error)?;
    info!("✅ Current regulation map: {:?}", regulation_map);

    // 3) Load latest persisted radiator states to avoid unnecessary calls.
    let current_states = get_latest_radiator_states(&client)
        .await
        .map_err(internal_error)?;
    info!("✅ Latest radiator states: {:?}", current_states);

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

    let mut updated_radiators = Vec::new();

    // 4) For each room: compute action, call Heatzy only when needed, then persist new state.
    for radiator in RADIATORS {
        info!(
            "Prepare the message to send for the device: [{}]",
            radiator.radiator
        );

        let current_mode = current_states
            .get(radiator.radiator)
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Missing current state in DB for radiator [{}]",
                        radiator.radiator
                    ),
                )
            })?
            .clone();

        // Business rule inherited from regulator: ECO mode is manually forced and must not be overridden.
        if current_mode == RadiatorMode::ECO {
            info!(
                "\t🧊 Radiator {} is in ECO mode, no override will be applied",
                radiator.radiator
            );
            continue;
        }

        let t_current = *room_temperatures.get(radiator.radiator).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Missing temperature for radiator [{}]", radiator.radiator),
            )
        })?;

        let t_target = *room_targets.get(radiator.radiator).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Missing target temperature for radiator [{}]", radiator.radiator),
            )
        })?;

        info!(
            "For device {}, current [{}], target: [{}]",
            radiator.radiator, t_current, t_target
        );

        let action = determine_action(t_current, t_target);
        info!("The action to perform is: [{:?}]", action);

        let next_mode = match action {
            RadiatorAction::On => Some(RadiatorMode::CFT),
            RadiatorAction::Off => Some(RadiatorMode::STOP),
            RadiatorAction::NoAction => None,
        };

        if let Some(next_mode) = next_mode {
            if next_mode != current_mode {
                info!(
                    "\t✅ Radiator {} current mode [{}], next mode [{}]",
                    radiator.radiator,
                    mode_as_str(current_mode),
                    mode_as_str(next_mode)
                );

                apply_radiator_mode_change(&client, &state, radiator, next_mode)
                    .await
                    .map_err(internal_error)?;

                updated_radiators.push(UpdatedRadiator {
                    radiator: radiator.radiator.to_string(),
                    status: mode_as_str(next_mode).to_string(),
                });
            } else {
                info!(
                    "\t✅ Radiator {} already in mode [{}], no update needed",
                    radiator.radiator,
                    mode_as_str(current_mode)
                );
            }
        } else {
            info!("\t✅ Radiator {} must stay the same", radiator.radiator);
        }
    }

    info!("🏁 Updated radiators response: {:?}", updated_radiators);
    Ok(Json(UpdateRadiatorResponse { updated_radiators }))
}

async fn open_db_connection(
    state: &AppState,
) -> anyhow::Result<(tokio_postgres::Client, tokio::task::JoinHandle<()>)> {
    let (client, connection) = tokio_postgres::connect(&state.db_url, NoTls).await?;

    let join_handle = tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    Ok((client, join_handle))
}

async fn apply_radiator_mode_change(
    client: &tokio_postgres::Client,
    state: &AppState,
    radiator: RadiatorConfig,
    mode: RadiatorMode,
) -> anyhow::Result<()> {
    info!(
        "\t📡 Send Heatzy command for radiator {} with DID {}",
        radiator.radiator, radiator.did
    );

    set_heatzy_mode(
        &state.heatzy_application_id,
        &state.heatzy_username,
        &state.heatzy_password,
        state.heatzy_token.clone(),
        radiator.did,
        mode,
    )
    .await?;

    save_radiator_state(client, radiator.radiator, mode).await?;
    info!(
        "\t📝 Persisted new state for radiator {} as [{}]",
        radiator.radiator,
        mode_as_str(mode)
    );

    Ok(())
}

async fn save_input_temperatures(
    client: &tokio_postgres::Client,
    payload: &UpdateRadiatorRequest,
) -> anyhow::Result<()> {
    let readings = [
        (TS_BUREAU, payload.bureau),
        (TS_CHAMBRE, payload.chambre),
        (TS_COULOIR, payload.couloir),
        (TS_SALON, payload.salon),
    ];

    let query = r#"INSERT INTO temperature_sensor_history (device_name, temperature, ts_create)
VALUES ($1, $2, timezone('UTC', current_timestamp))"#;

    for (device_name, temperature) in readings {
        client.execute(query, &[&device_name, &temperature]).await?;
        info!(
            "\t📝 Stored input temperature for sensor {} = [{}]",
            device_name, temperature
        );
    }

    Ok(())
}

/// Load the active heating plan according to local time.
async fn get_current_regulation_map(
    client: &tokio_postgres::Client,
) -> anyhow::Result<RegulationMap> {
    let local_time: NaiveTime = Local::now().time();

    let row = client
        .query_opt(CURRENT_REGULATION_MAP_SQL, &[&local_time])
        .await?
        .ok_or_else(|| anyhow!("No current regulation map found"))?;

    let reg_json: String = row.try_get("regulation_map_json")?;
    let reg: Value = serde_json::from_str(&reg_json)?;
    let reg_map: RegulationMap = serde_json::from_value(reg)?;
    Ok(reg_map)
}

/// Load most recent radiator state for each radiator topic.
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

/// Persist the new status right after a successful Heatzy update.
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

/// Call Heatzy control API to switch the radiator mode.
async fn set_heatzy_mode(
    heatzy_application_id: &str,
    heatzy_username: &str,
    heatzy_password: &str,
    heatzy_token: Arc<RwLock<String>>,
    did: &str,
    mode: RadiatorMode,
) -> anyhow::Result<()> {
    let client = HeatzyClient::new(
        heatzy_application_id,
        heatzy_username,
        heatzy_password,
        heatzy_token,
    );
    let previous_token = client.current_token()?;

    client.set_mode(did, mode).await?;

    let current_token = client.current_token()?;
    if current_token != previous_token {
        set_prop_value("heatzy.token", &current_token);
        info!("Heatzy token refreshed and saved into runtime configuration");
    }
    Ok(())
}

fn find_radiator_by_room(room: &str) -> Result<RadiatorConfig, (StatusCode, String)> {
    RADIATORS
        .iter()
        .find(|radiator| radiator.room == room)
        .copied()
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!(
                    "Unknown room [{}]. Supported rooms: {}",
                    room, SUPPORTED_ROOMS
                ),
            )
        })
}

fn parse_direct_mode(mode: &str) -> Result<RadiatorMode, (StatusCode, String)> {
    let parsed = mode_from_str(mode).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!(
                "Unsupported direct mode [{}]. Supported modes: {}",
                mode, SUPPORTED_DIRECT_MODES
            ),
        )
    })?;

    match parsed {
        RadiatorMode::CFT | RadiatorMode::STOP | RadiatorMode::ECO => Ok(parsed),
        _ => Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Unsupported direct mode [{}]. Supported modes: {}",
                mode, SUPPORTED_DIRECT_MODES
            ),
        )),
    }
}

/// Hysteresis copied from existing regulator logic.
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::routing::post;
    use axum::Router;
    use tower::util::ServiceExt;

    fn test_state() -> AppState {
        AppState {
            db_url: "postgres://invalid".to_string(),
            heatzy_application_id: "app".to_string(),
            heatzy_token: Arc::new(RwLock::new("token".to_string())),
            heatzy_username: "user".to_string(),
            heatzy_password: "password".to_string(),
        }
    }

    #[test]
    fn mode_from_str_supports_known_values() {
        assert_eq!(mode_from_str("CFT").unwrap(), RadiatorMode::CFT);
        assert_eq!(mode_from_str("STOP").unwrap(), RadiatorMode::STOP);
        assert_eq!(mode_from_str("ECO").unwrap(), RadiatorMode::ECO);
        assert_eq!(mode_from_str("FRO").unwrap(), RadiatorMode::FRO);
        assert!(mode_from_str("INVALID").is_err());
    }

    #[test]
    fn parse_direct_mode_rejects_unsupported_modes() {
        assert_eq!(parse_direct_mode("CFT").unwrap(), RadiatorMode::CFT);
        assert_eq!(parse_direct_mode("STOP").unwrap(), RadiatorMode::STOP);
        assert_eq!(parse_direct_mode("ECO").unwrap(), RadiatorMode::ECO);
        assert_eq!(
            parse_direct_mode("FRO").unwrap_err().0,
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn find_radiator_by_room_supports_public_rooms() {
        assert_eq!(
            find_radiator_by_room("bureau").unwrap().radiator,
            "external/rad_bureau"
        );
        assert_eq!(
            find_radiator_by_room("chambre").unwrap().radiator,
            "external/rad_chambre"
        );
        assert_eq!(
            find_radiator_by_room("couloir").unwrap().radiator,
            "external/rad_couloir"
        );
        assert_eq!(
            find_radiator_by_room("salon").unwrap().radiator,
            "external/rad_salon"
        );
        assert_eq!(
            find_radiator_by_room("garage").unwrap_err().0,
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn determine_action_preserves_hysteresis() {
        assert_eq!(determine_action(18.0, 19.0), RadiatorAction::On);
        assert_eq!(determine_action(20.0, 19.0), RadiatorAction::Off);
        assert_eq!(determine_action(19.1, 19.0), RadiatorAction::NoAction);
    }

    #[tokio::test]
    async fn direct_mode_endpoint_rejects_invalid_room() {
        let app = Router::new()
            .route("/radiator/:room", post(set_radiator_mode))
            .with_state(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/radiator/garage")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"mode":"CFT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn direct_mode_endpoint_rejects_invalid_mode() {
        let app = Router::new()
            .route("/radiator/:room", post(set_radiator_mode))
            .with_state(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/radiator/bureau")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"mode":"FRO"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8(body.to_vec())
            .unwrap()
            .contains("Supported modes: CFT, STOP, ECO"));
    }
}
