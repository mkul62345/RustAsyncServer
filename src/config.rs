use crate::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

#[allow(non_snake_case)]
pub struct Config {
    //Crypt
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,

    // Web
    //pub WEB_FOLDER: String,

    //DB
    pub DB_URL: String,

}

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|ex| {
            panic!("FATAL - LOADING CONFIG - Cause: {ex:?}")
        })
    })
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            //Crypt
            PWD_KEY: get_env_b64u_as_u8("SERVICE_PWD_KEY")?,                   
            TOKEN_KEY: get_env_b64u_as_u8("SERVICE_TOKEN_KEY")?,        
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,      
            // Web
            //WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
            //DB
            DB_URL: get_env("SERVICE_DB_URL")?,                  
            
          

        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_b64u_as_u8(name: &'static str) -> Result<Vec<u8>>{
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}

//Generic for implementors of FromStr
fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T>{
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}