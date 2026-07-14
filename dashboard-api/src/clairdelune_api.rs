use std::collections::HashMap;

use anyhow::anyhow;
use ava_toolkit::device_message::RegulationMapMsg;
use chrono::{DateTime, NaiveDateTime, Utc};
use commons_error::*;
use commons_pg::sql_transaction::CellValue;
use commons_pg::sql_transaction2::{SQLConnection2, SQLQueryBlock2};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
struct RoomConfig {
    room: &'static str,
    radiator_device: &'static str,
    temperature_devices: &'static [&'static str],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneSetting {
    zone: String,
    target_temperature: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatingPlanItem {
    id: i64,
    start_time: String,
    end_time: String,
    crosses_midnight: bool,
    boost_enabled: bool,
    regulation_map: serde_json::Value,
    zone_settings: Vec<ZoneSetting>,
    created_at: String,
}

#[derive(Serialize)]
pub struct HeatingPlanResponse {
    count: usize,
    items: Vec<HeatingPlanItem>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoomScheduleEntry {
    heating_plan_id: i64,
    start_time: String,
    end_time: String,
    crosses_midnight: bool,
    boost_enabled: bool,
    target_temperature: f32,
    created_at: String,
}

#[derive(Serialize)]
pub struct RoomSchedule {
    room: String,
    entries: Vec<RoomScheduleEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatingPlanByRoomResponse {
    room_count: usize,
    rooms: Vec<RoomSchedule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemperatureReading {
    sensor_name: String,
    temperature: f64,
    measured_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeTemperatureSection {
    mode: String,
    start_date_time: String,
    end_date_time: String,
    temperatures: Vec<TemperatureReading>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomTemperatureByModeResponse {
    room: String,
    start_date_time: String,
    end_date_time: String,
    section_count: usize,
    sections: Vec<ModeTemperatureSection>,
}

#[derive(Deserialize)]
pub struct RoomTemperatureByModeQuery {
    #[serde(rename = "room")]
    pub room: String,
    #[serde(rename = "startDateTime")]
    pub start_date_time: String,
    #[serde(rename = "endDateTime")]
    pub end_date_time: String,
}

#[derive(Debug)]
struct HeatingPlanRow {
    id: i64,
    start_time: String,
    end_time: String,
    crosses_midnight: bool,
    boost_enabled: bool,
    regulation_map: serde_json::Value,
    created_at: String,
}

#[derive(Debug, Clone)]
struct ModeHistoryRow {
    mode: String,
    ts_create: String,
    ts_ms: f64,
}

#[derive(Debug, Clone)]
struct TemperatureHistoryRow {
    device_name: String,
    temperature: f64,
    ts_create: String,
    ts_ms: f64,
}

const HEATING_PLAN_SQL: &str = r"
SELECT
    id,
    starting_time::text AS start_time,
    ending_time::text AS end_time,
    end_the_next_day AS crosses_midnight,
    boost AS boost_enabled,
    regulation_map,
    ts_created::text AS created_at
FROM heating_plan
ORDER BY starting_time, id";

const MODE_HISTORY_QUERY: &str = r#"
WITH last_before AS (
    SELECT
        state::jsonb->>'mode' AS mode,
        to_char(ts_create, 'YYYY-MM-DD"T"HH24:MI:SS.MS') AS ts_create,
        EXTRACT(EPOCH FROM ts_create) * 1000 AS ts_ms
    FROM device_state_history
    WHERE device_name = :p_device_name
      AND ts_create < :p_start_datetime::timestamp
      AND state::jsonb ? 'mode'
    ORDER BY ts_create DESC
    LIMIT 1
), within_range AS (
    SELECT
        state::jsonb->>'mode' AS mode,
        to_char(ts_create, 'YYYY-MM-DD"T"HH24:MI:SS.MS') AS ts_create,
        EXTRACT(EPOCH FROM ts_create) * 1000 AS ts_ms
    FROM device_state_history
    WHERE device_name = :p_device_name
      AND ts_create >= :p_start_datetime::timestamp
      AND ts_create <= :p_end_datetime::timestamp
      AND state::jsonb ? 'mode'
)
SELECT mode, ts_create, ts_ms
FROM (
    SELECT * FROM last_before
    UNION ALL
    SELECT * FROM within_range
) mode_changes
ORDER BY ts_ms"#;

fn room_config(room: &str) -> Option<RoomConfig> {
    match room {
        "bureau" => Some(RoomConfig {
            room: "bureau",
            radiator_device: "external/rad_bureau",
            temperature_devices: &["homey/ts_bureau", "zigbee2mqtt/ts_bureau"],
        }),
        "chambre" => Some(RoomConfig {
            room: "chambre",
            radiator_device: "external/rad_chambre",
            temperature_devices: &["homey/ts_chambre_1", "zigbee2mqtt/ts_chambre_1"],
        }),
        "salon" => Some(RoomConfig {
            room: "salon",
            radiator_device: "external/rad_salon",
            temperature_devices: &[
                "homey/ts_salon_1",
                "zigbee2mqtt/ts_salon_1",
                "zigbee2mqtt/ts_salon_2",
            ],
        }),
        "couloir" => Some(RoomConfig {
            room: "couloir",
            radiator_device: "external/rad_couloir",
            temperature_devices: &["homey/ts_couloir", "zigbee2mqtt/ts_couloir"],
        }),
        _ => None,
    }
}

fn parse_iso_datetime(raw: &str) -> anyhow::Result<DateTime<Utc>> {
    let naive =
        NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S").map_err(err_fwd!("Bad date"))?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

fn build_zone_settings(regulation_map: &RegulationMapMsg) -> Vec<ZoneSetting> {
    vec![
        ZoneSetting {
            zone: "tc_bureau".to_string(),
            target_temperature: regulation_map.tc_bureau,
        },
        ZoneSetting {
            zone: "tc_chambre_1".to_string(),
            target_temperature: regulation_map.tc_chambre_1,
        },
        ZoneSetting {
            zone: "tc_couloir".to_string(),
            target_temperature: regulation_map.tc_couloir,
        },
        ZoneSetting {
            zone: "tc_salon_1".to_string(),
            target_temperature: regulation_map.tc_salon_1,
        },
        ZoneSetting {
            zone: "tc_salon_2".to_string(),
            target_temperature: regulation_map.tc_salon_2,
        },
    ]
}

fn build_room_entries(item: &HeatingPlanItem) -> Vec<(String, RoomScheduleEntry)> {
    item.zone_settings
        .iter()
        .map(|setting| {
            (
                setting.zone.clone(),
                RoomScheduleEntry {
                    heating_plan_id: item.id,
                    start_time: item.start_time.clone(),
                    end_time: item.end_time.clone(),
                    crosses_midnight: item.crosses_midnight,
                    boost_enabled: item.boost_enabled,
                    target_temperature: setting.target_temperature,
                    created_at: item.created_at.clone(),
                },
            )
        })
        .collect()
}

fn to_timestamp_ms(raw: &str) -> anyhow::Result<f64> {
    Ok(parse_iso_datetime(raw)?.timestamp_millis() as f64)
}

fn compress_mode_rows(mode_rows: Vec<ModeHistoryRow>) -> Vec<ModeHistoryRow> {
    let mut compacted = Vec::new();
    let mut previous_mode: Option<String> = None;

    for row in mode_rows {
        if previous_mode.as_deref() == Some(row.mode.as_str()) {
            continue;
        }
        previous_mode = Some(row.mode.clone());
        compacted.push(row);
    }

    compacted
}

fn readings_for_section(
    temperature_rows: &[TemperatureHistoryRow],
    start_ms: f64,
    end_ms: f64,
    include_end_boundary: bool,
) -> Vec<TemperatureReading> {
    temperature_rows
        .iter()
        .filter(|row| {
            row.ts_ms >= start_ms
                && if include_end_boundary {
                    row.ts_ms <= end_ms
                } else {
                    row.ts_ms < end_ms
                }
        })
        .map(|row| TemperatureReading {
            sensor_name: row.device_name.clone(),
            temperature: row.temperature,
            measured_at: row.ts_create.clone(),
        })
        .collect()
}

fn to_sections(
    mode_rows: Vec<ModeHistoryRow>,
    temperature_rows: Vec<TemperatureHistoryRow>,
    start_date_time: &str,
    end_date_time: &str,
) -> anyhow::Result<Vec<ModeTemperatureSection>> {
    let start_ms = to_timestamp_ms(start_date_time)?;
    let end_ms = to_timestamp_ms(end_date_time)?;
    let normalized_mode_rows = compress_mode_rows(mode_rows);

    if normalized_mode_rows.is_empty() {
        return Ok(vec![ModeTemperatureSection {
            mode: "UNKNOWN".to_string(),
            start_date_time: start_date_time.to_string(),
            end_date_time: end_date_time.to_string(),
            temperatures: readings_for_section(&temperature_rows, start_ms, end_ms, true),
        }]);
    }

    let mut sections = Vec::new();

    for (index, row) in normalized_mode_rows.iter().enumerate() {
        let section_start_ms = row.ts_ms.max(start_ms);
        let section_start_date_time = if row.ts_ms < start_ms {
            start_date_time.to_string()
        } else {
            row.ts_create.clone()
        };

        let is_last_section = index == normalized_mode_rows.len() - 1;
        let (section_end_ms, section_end_date_time) = if is_last_section {
            (end_ms, end_date_time.to_string())
        } else {
            let next_row = &normalized_mode_rows[index + 1];
            if next_row.ts_ms < end_ms {
                (next_row.ts_ms, next_row.ts_create.clone())
            } else {
                (end_ms, end_date_time.to_string())
            }
        };

        if section_start_ms >= end_ms || section_start_ms >= section_end_ms {
            continue;
        }

        sections.push(ModeTemperatureSection {
            mode: row.mode.clone(),
            start_date_time: section_start_date_time,
            end_date_time: section_end_date_time,
            temperatures: readings_for_section(
                &temperature_rows,
                section_start_ms,
                section_end_ms,
                is_last_section,
            ),
        });
    }

    Ok(sections)
}

fn build_temperature_history_query(devices: &[&str]) -> (String, HashMap<String, CellValue>) {
    let mut sql = String::from(
        "SELECT device_name, temperature, \
         to_char(ts_create, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS') AS ts_create, \
         EXTRACT(EPOCH FROM ts_create) * 1000 AS ts_ms \
         FROM temperature_sensor_history \
         WHERE ts_create >= :p_start_datetime::timestamp \
           AND ts_create <= :p_end_datetime::timestamp \
           AND device_name IN (",
    );

    let mut params = HashMap::new();
    params.insert(
        "p_start_datetime".to_string(),
        CellValue::from_raw_string("".to_string()),
    );
    params.insert(
        "p_end_datetime".to_string(),
        CellValue::from_raw_string("".to_string()),
    );

    for (index, device) in devices.iter().enumerate() {
        if index > 0 {
            sql.push_str(", ");
        }
        let param_name = format!("p_device_{}", index);
        sql.push(':');
        sql.push_str(&param_name);
        params.insert(
            param_name,
            CellValue::from_raw_string((*device).to_string()),
        );
    }

    sql.push_str(") ORDER BY ts_create, device_name");
    (sql, params)
}

fn map_heating_plan_row(
    sql_result: &commons_pg::sql_transaction::SQLDataSet,
) -> anyhow::Result<HeatingPlanRow> {
    Ok(HeatingPlanRow {
        id: sql_result.get_int("id").ok_or(anyhow!("Wrong id"))?,
        start_time: sql_result
            .get_string("start_time")
            .ok_or(anyhow!("Wrong start_time"))?,
        end_time: sql_result
            .get_string("end_time")
            .ok_or(anyhow!("Wrong end_time"))?,
        crosses_midnight: sql_result
            .get_bool("crosses_midnight")
            .ok_or(anyhow!("Wrong crosses_midnight"))?,
        boost_enabled: sql_result
            .get_bool("boost_enabled")
            .ok_or(anyhow!("Wrong boost_enabled"))?,
        regulation_map: sql_result
            .get_json("regulation_map")
            .ok_or(anyhow!("Wrong regulation_map"))?,
        created_at: sql_result
            .get_string("created_at")
            .ok_or(anyhow!("Wrong created_at"))?,
    })
}

pub async fn get_heating_plan() -> anyhow::Result<HeatingPlanResponse> {
    let mut cnx = SQLConnection2::from_pool().await.map_err(tr_fwd!())?;
    let mut trans = cnx.begin().await.map_err(tr_fwd!())?;
    let query = SQLQueryBlock2 {
        sql_query: HEATING_PLAN_SQL.to_string(),
        start: 0,
        length: None,
        params: HashMap::new(),
    };

    let mut sql_result = query
        .execute(&mut trans)
        .await
        .map_err(err_fwd!("Heating plan query failed"))?;
    let mut items = Vec::new();

    while sql_result.next() {
        let row = map_heating_plan_row(&sql_result)?;
        let regulation_map_msg: RegulationMapMsg =
            serde_json::from_value(row.regulation_map.clone())
                .map_err(err_fwd!("Bad regulation_map"))?;

        items.push(HeatingPlanItem {
            id: row.id,
            start_time: row.start_time,
            end_time: row.end_time,
            crosses_midnight: row.crosses_midnight,
            boost_enabled: row.boost_enabled,
            regulation_map: row.regulation_map,
            zone_settings: build_zone_settings(&regulation_map_msg),
            created_at: row.created_at,
        });
    }

    trans.commit().await?;

    Ok(HeatingPlanResponse {
        count: items.len(),
        items,
    })
}

pub async fn get_heating_plan_by_room() -> anyhow::Result<HeatingPlanByRoomResponse> {
    let all = get_heating_plan().await?;
    let mut grouped: HashMap<String, Vec<RoomScheduleEntry>> = HashMap::new();

    for item in &all.items {
        for (room, entry) in build_room_entries(item) {
            grouped.entry(room).or_default().push(entry);
        }
    }

    let mut rooms: Vec<RoomSchedule> = grouped
        .into_iter()
        .map(|(room, mut entries)| {
            entries.sort_by(|left, right| {
                left.start_time
                    .cmp(&right.start_time)
                    .then(left.heating_plan_id.cmp(&right.heating_plan_id))
            });
            RoomSchedule { room, entries }
        })
        .collect();

    rooms.sort_by(|left, right| left.room.cmp(&right.room));

    Ok(HeatingPlanByRoomResponse {
        room_count: rooms.len(),
        rooms,
    })
}

pub async fn get_room_temperature_by_mode(
    query_params: &RoomTemperatureByModeQuery,
) -> anyhow::Result<RoomTemperatureByModeResponse> {
    let config = room_config(&query_params.room).ok_or(anyhow!(
        "Unknown room. Expected one of: bureau, chambre, salon, couloir."
    ))?;

    let start_dt = parse_iso_datetime(&query_params.start_date_time).map_err(err_fwd!(
        "startDateTime and endDateTime must be valid ISO date-time strings."
    ))?;
    let end_dt = parse_iso_datetime(&query_params.end_date_time).map_err(err_fwd!(
        "startDateTime and endDateTime must be valid ISO date-time strings."
    ))?;

    if start_dt >= end_dt {
        return Err(anyhow!("startDateTime must be earlier than endDateTime."));
    }

    let mut cnx = SQLConnection2::from_pool().await.map_err(tr_fwd!())?;
    let mut trans = cnx.begin().await.map_err(tr_fwd!())?;

    let mut mode_params = HashMap::new();
    mode_params.insert(
        "p_device_name".to_string(),
        CellValue::from_raw_string(config.radiator_device.to_string()),
    );
    mode_params.insert(
        "p_start_datetime".to_string(),
        CellValue::from_raw_string(query_params.start_date_time.clone()),
    );
    mode_params.insert(
        "p_end_datetime".to_string(),
        CellValue::from_raw_string(query_params.end_date_time.clone()),
    );

    let mode_query = SQLQueryBlock2 {
        sql_query: MODE_HISTORY_QUERY.to_string(),
        start: 0,
        length: None,
        params: mode_params,
    };

    let mut mode_sql_result = mode_query
        .execute(&mut trans)
        .await
        .map_err(err_fwd!("Mode history query failed"))?;
    let mut mode_rows = Vec::new();
    while mode_sql_result.next() {
        mode_rows.push(ModeHistoryRow {
            mode: mode_sql_result
                .get_string("mode")
                .ok_or(anyhow!("Wrong mode"))?,
            ts_create: mode_sql_result
                .get_string("ts_create")
                .ok_or(anyhow!("Wrong ts_create"))?,
            ts_ms: mode_sql_result
                .get_double("ts_ms")
                .ok_or(anyhow!("Wrong ts_ms"))?,
        });
    }

    let (temperature_sql, mut temperature_params) =
        build_temperature_history_query(config.temperature_devices);
    temperature_params.insert(
        "p_start_datetime".to_string(),
        CellValue::from_raw_string(query_params.start_date_time.clone()),
    );
    temperature_params.insert(
        "p_end_datetime".to_string(),
        CellValue::from_raw_string(query_params.end_date_time.clone()),
    );

    let temperature_query = SQLQueryBlock2 {
        sql_query: temperature_sql,
        start: 0,
        length: None,
        params: temperature_params,
    };

    let mut temperature_sql_result = temperature_query
        .execute(&mut trans)
        .await
        .map_err(err_fwd!("Temperature history query failed"))?;
    let mut temperature_rows = Vec::new();
    while temperature_sql_result.next() {
        temperature_rows.push(TemperatureHistoryRow {
            device_name: temperature_sql_result
                .get_string("device_name")
                .ok_or(anyhow!("Wrong device_name"))?,
            temperature: temperature_sql_result
                .get_double("temperature")
                .ok_or(anyhow!("Wrong temperature"))?,
            ts_create: temperature_sql_result
                .get_string("ts_create")
                .ok_or(anyhow!("Wrong ts_create"))?,
            ts_ms: temperature_sql_result
                .get_double("ts_ms")
                .ok_or(anyhow!("Wrong ts_ms"))?,
        });
    }

    trans.commit().await?;

    let sections = to_sections(
        mode_rows,
        temperature_rows,
        &query_params.start_date_time,
        &query_params.end_date_time,
    )?;

    Ok(RoomTemperatureByModeResponse {
        room: config.room.to_string(),
        start_date_time: query_params.start_date_time.clone(),
        end_date_time: query_params.end_date_time.clone(),
        section_count: sections.len(),
        sections,
    })
}
