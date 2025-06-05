use std::{fs, path::PathBuf, time::Duration};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;
use crate::{config::config, crypt::{pwd, EncryptContent}, ctx::{self, Ctx}, model::{user::{User, UserBackendModelController}, ModelManager}};

type Db = Pool<Postgres>;

const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";

//SQL files
const SQL_RECREATE_DB: &str = "sql/dev_init/00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_init";

const DEMO_PWD: &str = "demo";

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
    let app_db = new_db_pool(&config().DB_URL).await?;
    for path in paths {
        if let Some(path) = path.to_str() {
            let path = path.replace('\\', "/"); //Trick for windows path.

            //Only take the .sql and skip the SQL_RECREATE_DB
            if path.ends_with(".sql") && path != SQL_RECREATE_DB {
                pexec(&app_db, &path).await?;
            }
        }
    }

    // Init model layer
    let mm = ModelManager::new().await?;
    let ctx = Ctx::root_ctx();

    // Set demo1 password
    let demo1_user: User = UserBackendModelController::first_user_by_username(&ctx, &mm, "demo1")
        .await?
        .unwrap();

    /* // This works, the function in the bottom does nothing somewhy with the same code.
    let db = mm.db();
    let sql = format!(r#"UPDATE "user" SET "pwd" = $1 WHERE "id" = $2"#);  

    let pwd = pwd::encrypt_pwd(&EncryptContent{
            content: "jeff".to_string(),
            salt: "jeff".to_string()
        })?;

    let res = sqlx::query(&sql)
        .bind(pwd.to_string())
        .bind(demo1_user.id)
        .execute(db)
        .await?;
    */
   
    //TODO: Make this work, currently does nothing and postgres pops an error on some SELECT * FROM user WHERE id \ ID doesnt exist
    UserBackendModelController::update_pwd(&ctx, &mm, demo1_user.id, DEMO_PWD);

    info!("{:<12} - init_dev_db() - set demo1 pwd", "FOR-DEV-ONLY");

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


