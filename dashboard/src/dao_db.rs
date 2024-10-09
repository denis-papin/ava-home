use std::collections::HashMap;
use commons_error::*;
use anyhow::anyhow;
use chrono::Utc;
use serde_derive::{Deserialize, Serialize};
use commons_pg::sql_transaction2::{SQLConnection2, SQLQueryBlock2, SQLTransaction2};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub (crate) struct RadiatorStatus {
    pub mode: String
}

pub (crate) async fn build_current_radiator_state_context(mut trans: &mut SQLTransaction2<'_>) -> anyhow::Result<HashMap<String, String>> {
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

pub (crate) async fn build_current_temp_context() -> anyhow::Result<HashMap<String, String>> {
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
