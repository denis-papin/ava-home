use std::collections::HashMap;
use anyhow::anyhow;
use chrono::{Local, NaiveTime};
use serde_json::Value;
use commons_error::*;
use commons_pg::sql_transaction2::{SQLConnection2, SQLQueryBlock2};
use commons_pg::sql_transaction::CellValue;
use crate::device_message::RegulationMap;


const CURRENT_REGULATION_MAP_SQL : &str = "SELECT id, starting_time, ending_time, end_the_next_day, boost, regulation_map, ts_created
FROM public.heating_plan
WHERE boost = false
AND (
    (
        end_the_next_day = true
        AND (
            (starting_time <= :p_localtime)
            OR (:p_localtime < ending_time)
        )
    )
    OR (
        end_the_next_day = false
        AND starting_time <= :p_localtime
        AND :p_localtime < ending_time
    )
)
ORDER BY ts_created DESC";

pub (crate) async fn get_current_regulation_map() -> anyhow::Result<(NaiveTime, NaiveTime, RegulationMap)> {

    let mut params = HashMap::new();
    params.insert("p_localtime".to_owned(), CellValue::from_opt_naivetime(Some(Local::now().time())));

    let mut cnx = SQLConnection2::from_pool().await.map_err(tr_fwd!())?;
    let mut trans = cnx.begin().await.map_err(tr_fwd!())?;

    let query = SQLQueryBlock2 {
        sql_query : CURRENT_REGULATION_MAP_SQL.to_string(),
        start : 0,
        length : Some(1),
        params,
    };

    let mut sql_result = query.execute(&mut trans).await.map_err(err_fwd!("💣 Query failed, [{}]", &query.sql_query))?;

    if sql_result.next() {

        // let id = sql_result.get_int("id");
        let starting_time = sql_result.get_naivetime("starting_time")
            .ok_or_else(|| anyhow!("Aucune valeur trouvée pour starting_time"))?;
        let ending_time = sql_result.get_naivetime("ending_time")
            .ok_or_else(|| anyhow!("Aucune valeur trouvée pour ending_time"))?;

        // Gestion de l'extraction du JSON qui est une Option
        let reg: Option<Value> = sql_result.get_json("regulation_map");

        // Gestion de l'Option: Si le JSON est None, on retourne une erreur
        let reg = reg.ok_or_else(|| anyhow!("Aucune valeur trouvée pour regulation_map"))?;

        // Désérialisation du JSON en RegulationMap
        let reg_map: RegulationMap = serde_json::from_value(reg)
            .map_err(|e| anyhow!("Erreur lors de la désérialisation de la regulation_map: {}", e))?;

        Ok((starting_time, ending_time, reg_map))
    } else {
        Err(anyhow!("Impossible de trouver un plan de régulation"))
    }
}