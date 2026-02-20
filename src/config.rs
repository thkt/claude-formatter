use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub formatters: FormattersConfig,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct FormattersConfig {
    pub biome: bool,
    pub oxfmt: bool,
}

impl Default for FormattersConfig {
    fn default() -> Self {
        Self {
            biome: true,
            oxfmt: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            formatters: FormattersConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = match Self::config_search_paths(std::env::current_exe().ok().as_deref())
            .into_iter()
            .find(|p| p.exists())
        {
            Some(p) => p,
            None => return Config::default(),
        };

        match fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<Config>(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("formatter: invalid config at {:?}: {}", config_path, e);
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!("formatter: cannot read config {:?}: {}", config_path, e);
                Config::default()
            }
        }
    }

    fn config_search_paths(exe_path: Option<&std::path::Path>) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(config_dir) = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        {
            paths.push(config_dir.join("claude-formatter/config.json"));
        }

        if let Some(exe_dir) = exe_path.and_then(|p| p.parent()) {
            paths.push(exe_dir.join("config.json"));
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_all_enabled() {
        let config = Config::default();
        assert!(config.enabled);
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
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
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }

    #[test]
    fn parse_formatters_config() {
        let json = r#"{"formatters": {"biome": false}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }

    #[test]
    fn parse_oxfmt_disabled() {
        let json = r#"{"formatters": {"oxfmt": false}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.formatters.biome);
        assert!(!config.formatters.oxfmt);
    }

    #[test]
    fn config_search_paths_xdg_first() {
        let paths = Config::config_search_paths(Some(std::path::Path::new("/usr/bin/formatter")));
        let xdg_idx = paths
            .iter()
            .position(|p| p.to_string_lossy().contains("claude-formatter"));
        let exe_idx = paths
            .iter()
            .position(|p| p.to_string_lossy().contains("/usr/"));
        match (xdg_idx, exe_idx) {
            (Some(x), Some(e)) => {
                assert!(x < e, "XDG should come before exe-adjacent");
            }
            _ => panic!("expected both XDG and exe-adjacent paths (is HOME set?)"),
        }
    }

    #[test]
    fn config_search_paths_no_cwd() {
        let paths = Config::config_search_paths(Some(std::path::Path::new("/usr/bin/formatter")));
        assert!(
            !paths.contains(&PathBuf::from("config.json")),
            "CWD config.json should not be in search paths"
        );
    }
}
