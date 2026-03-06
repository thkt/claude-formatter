//! Configuration loading and merging.
//!
//! Supports `.claude/tools.json` (under the `formatter` key) at the git root,
//! with `.claude-formatter.json` as a legacy fallback.
//! Partial override semantics on top of all-enabled defaults.

use crate::resolve::find_git_root;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub enabled: bool,
    pub formatters: FormattersConfig,
}

pub struct FormattersConfig {
    pub biome: bool,
    pub oxfmt: bool,
    pub eof_newline: bool,
}

impl Default for FormattersConfig {
    fn default() -> Self {
        Self {
            biome: true,
            oxfmt: true,
            eof_newline: true,
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
    #[serde(rename = "eofNewline")]
    eof_newline: Option<bool>,
}

const TOOLS_CONFIG_FILE: &str = ".claude/tools.json";
const LEGACY_CONFIG_FILE: &str = ".claude-formatter.json";

#[derive(Deserialize)]
struct ToolsConfig {
    formatter: Option<ProjectConfig>,
}

impl Config {
    pub fn with_project_overrides(self, file_path: &str) -> Self {
        let Some(git_root) = find_git_root(file_path) else {
            return self;
        };

        // Try .claude/tools.json first, then legacy .claude-formatter.json
        let tools_path = git_root.join(TOOLS_CONFIG_FILE);
        if tools_path.exists() {
            return self.load_tools_config(&tools_path);
        }

        let legacy_path = git_root.join(LEGACY_CONFIG_FILE);
        if legacy_path.exists() {
            return self.load_legacy_config(&legacy_path);
        }

        self
    }

    fn load_tools_config(self, path: &PathBuf) -> Self {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("formatter: cannot read config {:?}: {}", path, e);
                return self;
            }
        };
        let tools: ToolsConfig = match serde_json::from_str(&content) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("formatter: invalid config {:?}: {}", path, e);
                return self;
            }
        };
        match tools.formatter {
            Some(project) => self.merge(project),
            None => self,
        }
    }

    fn load_legacy_config(self, path: &PathBuf) -> Self {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("formatter: cannot read project config {:?}: {}", path, e);
                return self;
            }
        };
        match serde_json::from_str::<ProjectConfig>(&content) {
            Ok(project) => self.merge(project),
            Err(e) => {
                eprintln!("formatter: invalid project config {:?}: {}", path, e);
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
            if let Some(v) = pf.eof_newline {
                self.formatters.eof_newline = v;
            }
        }
        self
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
        assert!(config.formatters.eof_newline);
    }

    #[test]
    fn with_project_overrides_from_tools_json() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(
            tmp.path().join(TOOLS_CONFIG_FILE),
            r#"{"formatter": {"formatters":{"oxfmt":false}}}"#,
        )
        .unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(!config.formatters.oxfmt);
        assert!(config.formatters.biome);
    }

    #[test]
    fn with_project_overrides_from_legacy_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::write(
            tmp.path().join(LEGACY_CONFIG_FILE),
            r#"{"formatters":{"oxfmt":false}}"#,
        )
        .unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(!config.formatters.oxfmt);
        assert!(config.formatters.biome);
    }

    #[test]
    fn tools_json_takes_priority_over_legacy() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(
            tmp.path().join(TOOLS_CONFIG_FILE),
            r#"{"formatter": {"formatters":{"oxfmt":false}}}"#,
        )
        .unwrap();
        fs::write(
            tmp.path().join(LEGACY_CONFIG_FILE),
            r#"{"formatters":{"biome":false}}"#,
        )
        .unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        // tools.json wins: oxfmt=false, biome stays default (true)
        assert!(!config.formatters.oxfmt);
        assert!(config.formatters.biome);
    }

    #[test]
    fn tools_json_without_formatter_key_returns_defaults() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(
            tmp.path().join(TOOLS_CONFIG_FILE),
            r#"{"reviews": {"some": "config"}}"#,
        )
        .unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }

    #[test]
    fn with_project_overrides_no_git_returns_unchanged() {
        let tmp = tempfile::TempDir::new().unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }

    #[test]
    fn with_project_overrides_malformed_json_returns_defaults() {
        let tmp = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        fs::write(tmp.path().join(TOOLS_CONFIG_FILE), "not valid json").unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let config = Config::default().with_project_overrides(file_path.to_str().unwrap());
        assert!(config.enabled);
        assert!(config.formatters.biome);
        assert!(config.formatters.oxfmt);
    }

    #[test]
    fn merge_partial_formatters_override() {
        let base = Config::default();
        let project: ProjectConfig =
            serde_json::from_str(r#"{"formatters": {"oxfmt": false}}"#).unwrap();

        let merged = base.merge(project);
        assert!(!merged.formatters.oxfmt);
        assert!(merged.formatters.biome);
        assert!(merged.formatters.eof_newline);
    }

    #[test]
    fn merge_eof_newline_override() {
        let base = Config::default();
        let project: ProjectConfig =
            serde_json::from_str(r#"{"formatters": {"eofNewline": false}}"#).unwrap();

        let merged = base.merge(project);
        assert!(!merged.formatters.eof_newline);
        assert!(merged.formatters.oxfmt);
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
}
