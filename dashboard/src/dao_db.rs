use std::collections::HashMap;

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use commons_error::*;
use commons_pg::sql_transaction2::{SQLConnection2, SQLQueryBlock2, SQLTransaction2};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct RadiatorStatus {
    pub mode: String,
}

pub(crate) async fn build_current_radiator_state_context(
    mut trans: &mut SQLTransaction2<'_>,
) -> anyhow::Result<HashMap<String, String>> {
    let mut context = HashMap::new();
    let query = SQLQueryBlock2 {
        // TODO find a better way to filter the device_name; use different table ?
        sql_query: r"select DISTINCT ON (device_name) device_name, state, ts_create
                    from device_state_history dsh where device_name like 'external/rad_%'
                    order by device_name, ts_create DESC"
            .to_string(),
        start: 0,
        length: None,
        params: HashMap::new(),
    };

    let r_sql_result = query.execute(&mut trans).await;
    let mut sql_result = r_sql_result.map_err(err_fwd!(
        "💣 Query failed, [{}], follower=[]",
        &query.sql_query /*, &self.follower*/
    ))?;

    while sql_result.next() {
        let device_name: String = sql_result
            .get_string("device_name")
            .ok_or(anyhow!("Wrong device_name"))?;
        let json_state: String = sql_result
            .get_string("state")
            .ok_or(anyhow!("Wrong state"))?;
        //let _ts_create = sql_result.get_timestamp_as_datetime("ts_create").ok_or(anyhow!("Wrong ts_create"))?;

        let status: RadiatorStatus = serde_json::from_str(json_state.as_str()).unwrap();

        match device_name.as_str() {
            "external/rad_bureau" => {
                context.insert("bureau_status".to_string(), status.mode);
                insert_radiator_status_presentation(&mut context, "bureau");
            }
            "external/rad_chambre" => {
                context.insert("chambre_status".to_string(), status.mode);
                insert_radiator_status_presentation(&mut context, "chambre");
            }
            "external/rad_couloir" => {
                context.insert("couloir_status".to_string(), status.mode);
                insert_radiator_status_presentation(&mut context, "couloir");
            }
            "external/rad_salon" => {
                context.insert("salon_status".to_string(), status.mode);
                insert_radiator_status_presentation(&mut context, "salon");
            }
            _ => {}
        }
    }
    Ok(context)
}

pub(crate) async fn build_current_temp_context() -> anyhow::Result<HashMap<String, String>> {
    let mut context = HashMap::new();
    let mut latest_by_room: HashMap<String, (f64, DateTime<Utc>)> = HashMap::new();

    let mut cnx = SQLConnection2::from_pool().await.map_err(tr_fwd!())?;
    let mut trans = cnx.begin().await.map_err(tr_fwd!())?;

    // Ajoute le contexte pour le statut des radiateurs
    context.extend(
        build_current_radiator_state_context(&mut trans)
            .await?
            .into_iter(),
    );

    let params = HashMap::new();

    let query = SQLQueryBlock2 {
        sql_query: r"SELECT DISTINCT ON (device_name) device_name, temperature, ts_create
        FROM temperature_sensor_history
        WHERE device_name IN (
            'zigbee2mqtt/ts_bureau',
            'homey/ts_bureau',
            'zigbee2mqtt/ts_chambre_1',
            'homey/ts_chambre_1',
            'zigbee2mqtt/ts_couloir',
            'homey/ts_couloir',
            'zigbee2mqtt/ts_salon_1',
            'homey/ts_salon_1'
        )
        ORDER BY device_name, ts_create DESC"
            .to_string(),
        start : 0,
        length : None,
        params,
    };

    let mut sql_result = query.execute(&mut trans).await.map_err(err_fwd!(
        "💣 Query failed, [{}], follower=[]",
        &query.sql_query /*, &self.follower*/
    ))?;

    while sql_result.next() {
        let device_name: String = sql_result
            .get_string("device_name")
            .ok_or(anyhow!("Wrong device_name"))?;
        let temperature: f64 = sql_result
            .get_double("temperature")
            .ok_or(anyhow!("Wrong temperature"))?;
        let ts_create = sql_result
            .get_timestamp_as_datetime("ts_create")
            .ok_or(anyhow!("Wrong ts_create"))?;

        if let Some(room) = sensor_name_to_room(&device_name) {
            let should_replace = latest_by_room
                .get(room)
                .map(|(_, latest_ts_create)| ts_create > *latest_ts_create)
                .unwrap_or(true);

            if should_replace {
                latest_by_room.insert(room.to_string(), (temperature, ts_create));
            }
        }
    }

    for (room, (temperature, ts_create)) in latest_by_room {
        let elapse_time_string = build_elapsed_time_string(ts_create);
        let temperature_string = format!("{:.1}", temperature).replace(".", ",");

        match room.as_str() {
            "bureau" => {
                context.insert("bureau_temperature".to_string(), temperature_string);
                context.insert("bureau_elapse".to_string(), elapse_time_string);
                context.insert("bureau_ts_create".to_string(), ts_create.to_rfc3339());
            }
            "chambre" => {
                context.insert("chambre_temperature".to_string(), temperature_string);
                context.insert("chambre_elapse".to_string(), elapse_time_string);
                context.insert("chambre_ts_create".to_string(), ts_create.to_rfc3339());
            }
            "couloir" => {
                context.insert("couloir_temperature".to_string(), temperature_string);
                context.insert("couloir_elapse".to_string(), elapse_time_string);
                context.insert("couloir_ts_create".to_string(), ts_create.to_rfc3339());
            }
            "salon" => {
                context.insert("salon_temperature".to_string(), temperature_string);
                context.insert("salon_elapse".to_string(), elapse_time_string);
                context.insert("salon_ts_create".to_string(), ts_create.to_rfc3339());
            }
            _ => {}
        }
    }

    let _r = trans.commit().await?;

    Ok(context)
}

fn build_elapsed_time_string(ts_create: DateTime<Utc>) -> String {
    let elapsed_minutes = Utc::now()
        .signed_duration_since(ts_create)
        .num_minutes()
        .abs();
    let elapsed_hours = elapsed_minutes / 60;
    let remaining_minutes = elapsed_minutes % 60;

    if elapsed_hours > 0 {
        format!("{} h {} min", elapsed_hours, remaining_minutes)
    } else {
        format!("{} min", remaining_minutes)
    }
}

fn insert_radiator_status_presentation(context: &mut HashMap<String, String>, room: &str) {
    let status_key = format!("{}_status", room);
    let status = context
        .get(&status_key)
        .map(|value| value.to_uppercase())
        .unwrap_or_else(|| "STOP".to_string());

    let (label, bg_class, text_class) = match status.as_str() {
        "CFT" => ("En chauffe", "bg-green-500", "text-green-500"),
        "ECO" => ("Desactive", "bg-yellow-500", "text-yellow-500"),
        _ => ("Arret", "bg-red-500", "text-red-500"),
    };

    context.insert(format!("{}_status_label", room), label.to_string());
    context.insert(format!("{}_status_bg_class", room), bg_class.to_string());
    context.insert(format!("{}_status_text_class", room), text_class.to_string());
}

fn sensor_name_to_room(device_name: &str) -> Option<&'static str> {
    match device_name {
        "zigbee2mqtt/ts_bureau" | "homey/ts_bureau" => Some("bureau"),
        "zigbee2mqtt/ts_chambre_1" | "homey/ts_chambre_1" => Some("chambre"),
        "zigbee2mqtt/ts_couloir" | "homey/ts_couloir" => Some("couloir"),
        "zigbee2mqtt/ts_salon_1" | "homey/ts_salon_1" => Some("salon"),
        _ => None,
    }
}
