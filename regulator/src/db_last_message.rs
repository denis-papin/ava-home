use log::info;
use tokio_postgres::NoTls;

pub (crate) async fn db_get_device_state(topic: &str) -> String {


    // URL de la base de données PostgreSQL
    let db_url = "postgresql://denis:dentece3.X@192.168.0.149/avahome";

    // Établir une connexion à la base de données
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await.unwrap();

    // Spawn une tâche pour gérer la processus de connexion en arrière-plan
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erreur de connexion : {}", e);
        }
    });

    // Query the db to find the most recent state for the device
    let query = format!(r#"select dsh.state from device_state_history dsh
                        where dsh.device_name = '{0}'
                        order by dsh.device_name, dsh.ts_create desc"#, &topic);

    let rows = client.query(&query, &[]).await.unwrap();

    if rows.is_empty() {
        String::from("")
    } else {
        let state: String = rows.get(0).unwrap().get("state");
        info!("Device : {}, State: {}", &topic, &state);
        state
    }
    // r#"{"mode":"FRO"}"#.to_string()
}