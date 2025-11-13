use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use toml;

const APPLICATION_DIR: &str = "bydit";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub user_agent: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

pub fn load_config(filename: &str, debug_mode: bool) -> Result<Config, Box<dyn Error>> {
    load_config_with_env(filename, debug_mode, EnvironmentPaths::from_process())
}

fn load_config_with_env(
    filename: &str,
    debug_mode: bool,
    env_paths: EnvironmentPaths,
) -> Result<Config, Box<dyn Error>> {
    let mut searched_paths = Vec::new();

    for candidate in candidate_paths_with_env(filename, &env_paths) {
        match fs::read_to_string(&candidate) {
            Ok(config_str) => {
                if debug_mode {
                    eprintln!("Using config file at {}", candidate.display());
                }
                return parse_config(&config_str, &candidate, debug_mode);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                searched_paths.push(candidate);
            }
            Err(err) => {
                let err_msg = format!(
                    "Failed to read config file '{}': {}",
                    candidate.display(),
                    err
                );
                return Err(Box::new(io::Error::new(err.kind(), err_msg)));
            }
        }
    }

    let message = format!(
        "Failed to locate config file '{}'. Checked paths: {}",
        filename,
        format_paths(&searched_paths)
    );
    if debug_mode {
        eprintln!("{}", message);
    }
    Err(Box::new(io::Error::new(ErrorKind::NotFound, message)))
}

fn parse_config(
    contents: &str,
    origin: &Path,
    debug_mode: bool,
) -> Result<Config, Box<dyn Error>> {
    toml::from_str(contents).map_err(|e| {
        if debug_mode {
            eprintln!(
                "Failed to parse config file '{}'. Check its format. Error: {}",
                origin.display(),
                e
            );
        }
        let err_msg = format!("Failed to parse config file '{}': {}", origin.display(), e);
        Box::new(io::Error::new(io::ErrorKind::InvalidData, err_msg)) as Box<dyn Error>
    })
}

#[derive(Clone, Debug)]
struct EnvironmentPaths {
    xdg_config_home: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
    current_dir: Option<PathBuf>,
}

impl EnvironmentPaths {
    fn from_process() -> Self {
        Self {
            xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            xdg_data_home: env::var_os("XDG_DATA_HOME").map(PathBuf::from),
            home_dir: env::var_os("HOME").map(PathBuf::from),
            current_dir: env::current_dir().ok(),
        }
    }
}

fn candidate_paths_with_env(filename: &str, env_paths: &EnvironmentPaths) -> Vec<PathBuf> {
    let requested = Path::new(filename);

    if requested.is_absolute() {
        return vec![requested.to_path_buf()];
    }

    let mut paths = Vec::new();
    let base_dir = env_paths
        .current_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    paths.push(base_dir.join(requested));

    if let Some(config_home) = &env_paths.xdg_config_home {
        paths.push(config_home.join(APPLICATION_DIR).join(requested));
    } else if let Some(home_dir) = &env_paths.home_dir {
        paths.push(
            home_dir
                .join(".config")
                .join(APPLICATION_DIR)
                .join(requested),
        );
    }

    if let Some(data_home) = &env_paths.xdg_data_home {
        paths.push(data_home.join(APPLICATION_DIR).join(requested));
    } else if let Some(home_dir) = &env_paths.home_dir {
        paths.push(
            home_dir
                .join(".local")
                .join("share")
                .join(APPLICATION_DIR)
                .join(requested),
        );
    }

    paths
}

fn format_paths(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return "none".to_string();
    }

    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use tempfile::tempdir;

    fn sample_config() -> &'static str {
        "user_agent = \"ua\"\nclient_id = \"id\"\nclient_secret = \"secret\"\nusername = \"user\"\npassword = \"pass\"\n"
    }

    #[test]
    fn candidate_order_prefers_xdg_homes() {
        let env_paths = EnvironmentPaths {
            xdg_config_home: Some(PathBuf::from("/tmp/xdg_config")),
            xdg_data_home: Some(PathBuf::from("/tmp/xdg_data")),
            home_dir: Some(PathBuf::from("/home/demo")),
            current_dir: Some(PathBuf::from("/work/project")),
        };

        let candidates = candidate_paths_with_env("config.toml", &env_paths);

        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/work/project/config.toml"),
                PathBuf::from("/tmp/xdg_config/bydit/config.toml"),
                PathBuf::from("/tmp/xdg_data/bydit/config.toml"),
            ]
        );
    }

    #[test]
    fn candidate_order_falls_back_to_home_dirs() {
        let env_paths = EnvironmentPaths {
            xdg_config_home: None,
            xdg_data_home: None,
            home_dir: Some(PathBuf::from("/home/demo")),
            current_dir: Some(PathBuf::from("/work/project")),
        };

        let candidates = candidate_paths_with_env("config.toml", &env_paths);

        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/work/project/config.toml"),
                PathBuf::from("/home/demo/.config/bydit/config.toml"),
                PathBuf::from("/home/demo/.local/share/bydit/config.toml"),
            ]
        );
    }

    #[test]
    fn load_config_reads_from_xdg_config_home() -> Result<(), Box<dyn Error>> {
        let temp = tempdir()?;
        let config_home = temp.path().join("xdg_config");
        fs::create_dir_all(config_home.join(APPLICATION_DIR))?;
        let config_path = config_home.join(APPLICATION_DIR).join("config.toml");
        fs::write(&config_path, sample_config())?;

        let env_paths = EnvironmentPaths {
            xdg_config_home: Some(config_home),
            xdg_data_home: None,
            home_dir: Some(temp.path().to_path_buf()),
            current_dir: Some(temp.path().to_path_buf()),
        };

        let config = load_config_with_env("config.toml", false, env_paths)?;
        assert_eq!(config.username, "user");
        Ok(())
    }

    #[test]
    fn load_config_reports_checked_paths() {
        let env_paths = EnvironmentPaths {
            xdg_config_home: Some(PathBuf::from("/tmp/xdg_config")),
            xdg_data_home: None,
            home_dir: Some(PathBuf::from("/home/demo")),
            current_dir: Some(PathBuf::from("/work/project")),
        };

        let err = load_config_with_env("missing-config.toml", false, env_paths)
            .expect_err("Expected missing config error");
        let io_err = err.downcast::<io::Error>().unwrap();

        assert_eq!(io_err.kind(), ErrorKind::NotFound);
        assert!(io_err
            .to_string()
            .contains("/tmp/xdg_config/bydit/missing-config.toml"));
    }
}
