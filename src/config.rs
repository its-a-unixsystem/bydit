use serde::Deserialize;
use std::fs;
use std::error::Error;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub user_agent: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

pub fn load_config(path: &str, debug_mode: bool) -> Result<Config, Box<dyn Error>> {
    let config_str = fs::read_to_string(path).map_err(|e| {
        if debug_mode {
            let current_dir_msg = match std::env::current_dir() {
                Ok(cd) => format!("Current directory: {:?}", cd),
                Err(_) => "Could not determine current directory.".to_string(),
            };
            eprintln!(
                "Failed to read config file '{}'. Make sure it exists. Error: {}. {}",
                path, e, current_dir_msg
            );
        }
        let err_msg = format!("Failed to read config file '{}': {}", path, e);
        Box::new(std::io::Error::new(e.kind(), err_msg)) as Box<dyn Error>
    })?;

    let config: Config = toml::from_str(&config_str).map_err(|e| {
        if debug_mode {
            eprintln!(
                "Failed to parse config file '{}'. Check its format. Error: {}",
                path, e
            );
        }
        let err_msg = format!("Failed to parse config file '{}': {}", path, e);
        Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, err_msg)) as Box<dyn Error>
    })?;

    Ok(config)
}
