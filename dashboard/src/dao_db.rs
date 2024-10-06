// use std::collections::HashMap;
//
// use chrono::{DateTime, Utc};
// use log::info;
//
// use commons_error::*;
// use commons_pg::{SQLConnection, SQLDataSet, SQLQueryBlock, SQLTransaction};
//
// #[derive(Debug)]
// pub (crate) struct TemperatureForSensor {
//     pub sensor_name: String,
//     pub temp : f32,
//     pub elapse_min: u32,
// }
//
// pub fn open_transaction( r_cnx: &'_ mut anyhow::Result<SQLConnection>) -> anyhow::Result<SQLTransaction<'_>> {
//     let cnx = match r_cnx.as_mut().map_err(err_fwd!("Fail opening db connection")) {
//         Ok(x) => {x}
//         Err(_) => {
//             return Err(anyhow::anyhow!("_"));
//         }
//     };
//     let trans = cnx.sql_transaction().map_err(err_fwd!("Fail starting a transaction"))?;
//     Ok(trans)
// }
//
// pub (crate) fn fetch_temperature() -> Vec<TemperatureForSensor> {
//
//     let mut r_cnx = SQLConnection::new();
//     let mut trans = open_transaction(&mut r_cnx).map_err(err_fwd!("💣 Open transaction error")).unwrap(); // TODO
//     // let Ok(mut trans) = r_trans else {
//     //     return WebType::from_errorset(INTERNAL_DATABASE_ERROR);
//     // };
//
//     let mut params = HashMap::new();
//     // params.insert("p_item_id".to_owned(), p_item_id);
//
//     let sql_query = "SELECT DISTINCT ON (device_name) device_name, temperature, ts_create
//                  FROM temperature_sensor_history
//                  ORDER BY device_name, ts_create DESC".to_string();
//
//     let query = SQLQueryBlock {
//         sql_query,
//         start : 0,
//         length : None,
//         params,
//     };
//
//     let mut sql_result : SQLDataSet =  query.execute(&mut trans).map_err(err_fwd!("Query failed, [{}]", &query.sql_query)).unwrap(); // TODO
//
//     // dbg!(sql_result);
//
//     // Obtenez l'heure courante UTC
//     let now: DateTime<Utc> = Utc::now();
//
//     let mut temperatures : Vec<TemperatureForSensor> = vec![];
//     // Traiter les résultats
//     while sql_result.next() {
//         let device_name : String = sql_result.get_string("device_name").unwrap_or("".to_owned());
//         // let device_name: String = row.get("device_name");
//         let temperature : f64 = sql_result.get_double("temperature").unwrap();
//         // let temperature: f64 = row.get("temperature");
//         let ts_create  = sql_result.get_timestamp_as_datetime("ts_create").unwrap();
//         //let ts_create: SystemTime = row.get("ts_create");
//         let dt: DateTime<Utc> = ts_create.clone().into();
//         // Calcule la différence de temps en minutes
//         let difference = now.signed_duration_since(dt);
//
//         // Convertit la différence en minutes (Duration implémente From<Duration> pour i64)
//         let difference_in_minutes = difference.num_minutes() + 60; // FIXME the db - the ts from the DB is a localtime so we must add 60 minutes !!!
//
//         dbg!(difference_in_minutes);
//
//         let tfs = TemperatureForSensor {
//             sensor_name: device_name.clone(),
//             temp: temperature as f32,
//             elapse_min: difference_in_minutes as u32,
//         };
//         temperatures.push(tfs);
//         info!("Device : {}, Température: {}, Créé à: {:?}", device_name, temperature, dt);
//     }
//
//     let _ = trans.commit().unwrap();
//
//     temperatures
// }