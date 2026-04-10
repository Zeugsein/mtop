use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub interval_ms: u32,
    pub temp_unit: String,
    pub sort_mode: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            interval_ms: 1000,
            temp_unit: "celsius".to_string(),
            sort_mode: "score".to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("mtop").join("config.toml")
}

/// Load config from ~/.config/mtop/config.toml. Returns defaults on any error.
pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("mtop: invalid config {}: {}", path.display(), e);
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}

/// Save config to ~/.config/mtop/config.toml. Creates directory if needed.
pub fn save(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(cfg)?;
    std::fs::write(&path, contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.theme, "default");
        assert_eq!(cfg.interval_ms, 1000);
        assert_eq!(cfg.temp_unit, "celsius");
        assert_eq!(cfg.sort_mode, "score");
    }

    #[test]
    fn deserialize_full_config() {
        let toml_str = r#"
theme = "monokai"
interval_ms = 500
temp_unit = "fahrenheit"
sort_mode = "cpu"
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.theme, "monokai");
        assert_eq!(cfg.interval_ms, 500);
        assert_eq!(cfg.temp_unit, "fahrenheit");
        assert_eq!(cfg.sort_mode, "cpu");
    }

    #[test]
    fn deserialize_partial_config() {
        let toml_str = r#"
theme = "nord"
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.theme, "nord");
        assert_eq!(cfg.interval_ms, 1000); // default
        assert_eq!(cfg.sort_mode, "score"); // default
    }

    #[test]
    fn deserialize_empty_config() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.theme, "default");
        assert_eq!(cfg.interval_ms, 1000);
    }

    #[test]
    fn unknown_keys_ignored() {
        let toml_str = r#"
theme = "horizon"
unknown_future_key = true
another = 42
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.theme, "horizon");
    }

    #[test]
    fn serialize_roundtrip() {
        let cfg = Config {
            theme: "dracula".to_string(),
            interval_ms: 750,
            temp_unit: "celsius".to_string(),
            sort_mode: "memory".to_string(),
        };
        let serialized = toml::to_string_pretty(&cfg).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.theme, "dracula");
        assert_eq!(deserialized.interval_ms, 750);
        assert_eq!(deserialized.sort_mode, "memory");
    }

    #[test]
    fn config_path_contains_mtop() {
        let path = config_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("mtop"), "config path should contain 'mtop': {}", path_str);
        assert!(path_str.ends_with("config.toml"), "config path should end with config.toml: {}", path_str);
    }
}
