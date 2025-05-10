use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Represents the user-defined configuration, possibly partial.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub log_type: Option<String>,  // "txt" or "json"
    pub file_name: Option<String>, // Log file name
    pub min_level: Option<String>, // Minimum log level (e.g., "info", "warn", "error", "debug")
    pub to_file: Option<bool>,     // Whether to write to file (default: true)
}

impl Config {
    /// Reads a configuration file from a `.toml` path and parses it into a `Config`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path).map_err(ConfigError::Io)?;
        toml::from_str::<Self>(&contents).map_err(ConfigError::Toml)
    }

    /// Finalizes the configuration by filling in defaults for any missing field.
    pub fn finalize(self) -> FinalConfig {
        FinalConfig {
            log_type: self.log_type.unwrap_or_else(|| "txt".to_string()),
            file_name: self.file_name.unwrap_or_else(|| "log".to_string()),
            min_level: self.min_level.unwrap_or_else(|| "info".to_string()),
            to_file: self.to_file.unwrap_or(true), // Default: true
        }
    }
}

/// The fully specified and resolved configuration used internally.
#[derive(Debug, Clone)]
pub struct FinalConfig {
    pub log_type: String,
    pub file_name: String,
    pub min_level: String,
    pub to_file: bool,
}

/// Errors that might occur during configuration loading.
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_finalize_with_all_fields() {
        let config = Config {
            log_type: Some("json".to_string()),
            file_name: Some("mylog".to_string()),
            min_level: Some("warn".to_string()),
            to_file: Some(false),
        };
        let final_config = config.finalize();
        assert_eq!(final_config.log_type, "json");
        assert_eq!(final_config.file_name, "mylog");
        assert_eq!(final_config.min_level, "warn");
        assert!(!final_config.to_file);
    }

    #[test]
    fn test_finalize_with_defaults() {
        let config = Config {
            log_type: None,
            file_name: None,
            min_level: None,
            to_file: None,
        };
        let final_config = config.finalize();
        assert_eq!(final_config.log_type, "txt");
        assert_eq!(final_config.file_name, "log");
        assert_eq!(final_config.min_level, "info");
        assert!(final_config.to_file);
    }

    #[test]
    fn test_from_file_success() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.toml");

        let toml_content = r#"
            log_type = "json"
            file_name = "myapp"
            min_level = "error"
            to_file = false
        "#;

        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(&file_path).unwrap();
        assert_eq!(config.log_type.unwrap(), "json");
        assert_eq!(config.file_name.unwrap(), "myapp");
        assert_eq!(config.min_level.unwrap(), "error");
        assert_eq!(config.to_file.unwrap(), false);
    }

    #[test]
    fn test_from_file_missing_file() {
        let result = Config::from_file("nonexistent.toml");
        assert!(matches!(result, Err(ConfigError::Io(_))));
    }

    #[test]
    fn test_from_file_malformed_toml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("bad.toml");

        let toml_content = r#"
            log_type = "json"
            file_name = "log
        "#;

        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let result = Config::from_file(&file_path);
        assert!(matches!(result, Err(ConfigError::Toml(_))));
    }
}
