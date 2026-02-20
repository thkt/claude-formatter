use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Config {
    pub enabled: bool,
    pub formatters: FormattersConfig,
}

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

#[derive(Deserialize)]
struct ProjectConfig {
    enabled: Option<bool>,
    formatters: Option<ProjectFormattersConfig>,
}

#[derive(Deserialize)]
struct ProjectFormattersConfig {
    biome: Option<bool>,
    oxfmt: Option<bool>,
}

const PROJECT_CONFIG_NAME: &str = ".claude-formatter.json";

impl Config {
    pub fn with_project_overrides(self, file_path: &str) -> Self {
        let Some(config_path) = Self::find_project_config(file_path) else {
            return self;
        };

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "formatter: cannot read project config {:?}: {}",
                    config_path, e
                );
                return self;
            }
        };

        match serde_json::from_str::<ProjectConfig>(&content) {
            Ok(project) => self.merge(project),
            Err(e) => {
                eprintln!("formatter: invalid project config {:?}: {}", config_path, e);
                self
            }
        }
    }

    fn merge(mut self, project: ProjectConfig) -> Self {
        if let Some(enabled) = project.enabled {
            self.enabled = enabled;
        }
        if let Some(pf) = project.formatters {
            if let Some(v) = pf.biome {
                self.formatters.biome = v;
            }
            if let Some(v) = pf.oxfmt {
                self.formatters.oxfmt = v;
            }
        }
        self
    }

    fn find_project_config(file_path: &str) -> Option<PathBuf> {
        let mut dir = Path::new(file_path).parent();
        while let Some(d) = dir {
            if d.join(".git").exists() {
                let candidate = d.join(PROJECT_CONFIG_NAME);
                return candidate.exists().then_some(candidate);
            }
            dir = d.parent();
        }
        None
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
    fn find_project_config_at_git_root() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        let config_path = tmp.path().join(".claude-formatter.json");
        fs::write(&config_path, r#"{"formatters":{"oxfmt":false}}"#).unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let result = Config::find_project_config(file_path.to_str().unwrap());
        assert_eq!(result, Some(config_path));
    }

    #[test]
    fn find_project_config_missing_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let result = Config::find_project_config(file_path.to_str().unwrap());
        assert_eq!(result, None);
    }

    #[test]
    fn find_project_config_no_git_returns_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join(".claude-formatter.json");
        fs::write(config_path, "{}").unwrap();

        let file_path = tmp.path().join("app.ts");
        let result = Config::find_project_config(file_path.to_str().unwrap());
        assert_eq!(result, None);
    }

    #[test]
    fn merge_partial_formatters_override() {
        let base = Config::default();
        let project: ProjectConfig =
            serde_json::from_str(r#"{"formatters": {"oxfmt": false}}"#).unwrap();

        let merged = base.merge(project);
        assert!(!merged.formatters.oxfmt);
        assert!(merged.formatters.biome);
    }

    #[test]
    fn merge_enabled_override() {
        let base = Config::default();
        let project: ProjectConfig = serde_json::from_str(r#"{"enabled": false}"#).unwrap();

        let merged = base.merge(project);
        assert!(!merged.enabled);
        assert!(merged.formatters.biome);
    }

    #[test]
    fn merge_empty_project_config_no_change() {
        let base = Config::default();
        let project: ProjectConfig = serde_json::from_str(r#"{}"#).unwrap();

        let merged = base.merge(project);
        assert!(merged.enabled);
        assert!(merged.formatters.biome);
        assert!(merged.formatters.oxfmt);
    }

    #[test]
    fn with_project_overrides_applies_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::write(
            tmp.path().join(".claude-formatter.json"),
            r#"{"formatters":{"oxfmt":false}}"#,
        )
        .unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(!config.formatters.oxfmt);
        assert!(config.formatters.biome);
    }

    #[test]
    fn with_project_overrides_malformed_json_returns_defaults() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::write(
            tmp.path().join(".claude-formatter.json"),
            "not valid json",
        )
        .unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(config.enabled);
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }

    #[test]
    fn with_project_overrides_no_config_returns_unchanged() {
        let tmp = tempfile::TempDir::new().unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }
}
