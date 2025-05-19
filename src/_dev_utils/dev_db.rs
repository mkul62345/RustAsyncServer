use std::{fs, path::PathBuf, time::Duration};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;
use crate::config::config;

type Db = Pool<Postgres>;

const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://pg:pg@localhost/app_db";

//SQL files
const SQL_RECREATE_DB: &str = "sql/dev_init/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_init";


pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>>{
    info!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    {
        let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
        pexec(&root_db, SQL_RECREATE_DB).await?;
    }

    //Fetch sql fiiles.
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
    .filter_map(|entry| entry.ok().map(|e| e.path()))
    .collect();

    //Sorting to make sure the sql files are executed in the correct sequence.
    paths.sort();

    //Execute each .sql file.
    let app_db = new_db_pool(PG_DEV_APP_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); //Trick for windows path.

            //Only take the .sql and skip the SQL_RECREATE_DB
            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pexec(&app_db, &path).await?;
            }
        }
    }

    Ok(())
}

async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    info!("{:<12} - pexec: {file}", "FOR-DEV-ONLY");

    let content = fs::read_to_string(file)?;
    let sqls: Vec<&str> = content.split(';').collect();
    for sql in sqls {
        sqlx::query(sql).execute(db).await?;
    }

    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_con_url)
        .await
}