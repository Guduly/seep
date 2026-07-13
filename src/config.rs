use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub discord_token: String,
}

fn config_path() -> PathBuf {
    dirs::config_dir().unwrap().join("seep").join("config.toml")
}

pub fn save(token: &str) -> color_eyre::Result<()> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap())?;
    let config = Config {
        discord_token: token.to_string(),
    };
    let text = toml::to_string(&config)?;
    fs::write(path, text)?;
    Ok(())
}

pub fn load() -> Option<Config> {
    let path = config_path();
    let text = fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}
