use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_search_paths(std::env::current_exe().ok().as_deref())
            .into_iter()
            .find(|p| p.exists())
            .unwrap_or_default();

        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<Config>(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("formatter: invalid config at {:?}: {}", config_path, e);
                    Config::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
            Err(_) => Config::default(),
        }
    }

    fn config_search_paths(exe_path: Option<&std::path::Path>) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(exe_dir) = exe_path.and_then(|p| p.parent()) {
            paths.push(exe_dir.join("config.json"));
        }

        paths.push(PathBuf::from("config.json"));

        if let Some(config_dir) = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        {
            paths.push(config_dir.join("claude-formatter/config.json"));
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enabled() {
        let config = Config::default();
        assert!(config.enabled);
    }

    #[test]
    fn parse_disabled_config() {
        let json = r#"{"enabled": false}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn parse_empty_config_uses_defaults() {
        let json = "{}";
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
    }
}
