use std::{thread::sleep, time::Duration};

use rusqlite::{Connection, Error};
use serde::{Deserialize, Serialize};
use tokio::time;

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    db_is_ready: bool,
}

fn status(name: &str, conn: &Connection) -> Result<Status, Error> {
    let result: String = conn
        .query_row("PRAGMA sync_status", [], |row| row.get(0))
        .unwrap();

    let status: Status = serde_json::from_str(result.as_str()).unwrap();

    println!("{name}: {:#?}", status.db_is_ready);

    Ok(status)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let primary = Connection::open("file:primary?node=primary&bind=tcp://127.0.0.1:6667")?;

    primary.execute(
        "CREATE TABLE IF NOT EXISTS test (key string, value string)",
        [],
    )?;

    loop {
        if status("primary", &primary).unwrap().db_is_ready {
            break;
        }

        sleep(Duration::from_secs(1));
    }

    let secondary = Connection::open("file:secondary?node=secondary&connect=tcp://127.0.0.1:6667")?;

    let primary_monitor = tokio::task::spawn_blocking(move || loop {
        let result: i32 = primary
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();

        println!("primary: {:#?}", result);

        sleep(Duration::from_secs(1));
    });

    let secondary_increment = tokio::spawn(async move {
        loop {
            if status("secondary", &secondary).unwrap().db_is_ready {
                break;
            }

            time::sleep(time::Duration::from_secs(1)).await;
        }

        loop {
            println!("secondary: insert");

            secondary
                .execute(
                    "INSERT INTO test VALUES (CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )
                .unwrap();

            time::sleep(time::Duration::from_secs(1)).await;
        }
    });

    tokio::select! {
        _ = primary_monitor => {},
        _ = secondary_increment => {},
    }

    Ok(())
}
