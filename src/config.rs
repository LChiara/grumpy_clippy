use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

/// Enum representing different grumpiness level
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GrumpinessLevel {
    Mild,
    Sarcastic,
    Rude,
}

impl fmt::Display for GrumpinessLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for GrumpinessLevel {
    type Err = ConfigError; // Custom Error Type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mild" => Ok(GrumpinessLevel::Mild),
            "sarcastic" => Ok(GrumpinessLevel::Sarcastic),
            "rude" => Ok(GrumpinessLevel::Rude),
            _ => Err(ConfigError::InvalidGrumpinessLevel(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    pub grumpiness_level: Option<GrumpinessLevel>,
    pub verbose: Option<bool>,
    pub watch_files: Option<Vec<String>>,
    pub ignore_patterns: Option<Vec<String>>,
    pub max_function_size: Option<u8>,
    pub max_complexity: Option<u8>,
    pub custom_rules: Option<String>,
    pub git_integration: Option<bool>,
    pub rules_file: Option<String>,
}

impl FileConfig {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFoundError(path.to_path_buf()));
        }
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidFile(path.to_path_buf(), e.to_string()))?;
        let parsed_content = toml::from_str(&content)
            .map_err(|e| ConfigError::InvalidFile(path.to_path_buf(), e.to_string()))?;

        Ok(parsed_content)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFoundError(PathBuf),
    InvalidFile(PathBuf, String),
    ValueTooSmall(String, u8, u8),
    MissingWatchFiles,
    InvalidGrumpinessLevel(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFoundError(filepath) => {
                write!(
                    f,
                    "File '{}' could not be found",
                    filepath.display().to_string()
                )
            }
            ConfigError::InvalidFile(filepath, error) => {
                write!(
                    f,
                    "File '{}' seems to be invalid. Error: {}",
                    filepath.display().to_string(),
                    error
                )
            }
            ConfigError::InvalidGrumpinessLevel(value) => {
                write!(
                    f,
                    "GrumpinessLevel must be 'mild', 'sarcastic', or 'rude', but got {}",
                    value
                )
            }
            ConfigError::ValueTooSmall(param, value, min_value) => {
                write!(
                    f,
                    "{} must be greater than {}, but got {}",
                    param, min_value, value
                )
            }
            ConfigError::MissingWatchFiles => {
                write!(f, "Watch files shall not be empty!")
            }
        }
    }
}

/// Unit tests for this class
#[cfg(test)]
mod tests {
    use super::FileConfig;
    use crate::config::{ConfigError, GrumpinessLevel};
    use std::io::Write;
    use std::{fs::File, path::Path, str::FromStr};
    use tempfile::tempdir;

    #[test]
    fn test_grumpiness_level_parsing() {
        assert_eq!(
            GrumpinessLevel::from_str("mild").unwrap(),
            GrumpinessLevel::Mild
        );
        assert_eq!(
            GrumpinessLevel::from_str("SARCASTIC").unwrap(),
            GrumpinessLevel::Sarcastic
        );
        assert_eq!(
            GrumpinessLevel::from_str("ruDE").unwrap(),
            GrumpinessLevel::Rude
        );
        assert!(GrumpinessLevel::from_str("NOT_A_GUMPINESS_LEVEL").is_err())
    }

    #[test]
    fn test_valide_file_config_deserialization() {
        let toml_data = r#"
            grumpiness_level = "sarcastic"
            verbose = true
            watch_files = [".rs", ".toml"]
            max_function_size = 42
            output_format = "json"
            git_integration = "always"
        "#;

        let config: FileConfig = toml::from_str(toml_data).unwrap();
        assert_eq!(config.grumpiness_level.unwrap(), GrumpinessLevel::Sarcastic);
        assert_eq!(config.verbose.unwrap(), true);
        assert_eq!(config.watch_files.unwrap(), vec![".rs", ".toml"]);
        assert_eq!(config.max_function_size.unwrap(), 42);
        assert!(config.rules_file.is_none());
    }

    #[test]
    fn test_invalid_file_path_config_file() {
        let file_path = Path::new("file_does_not_exists.toml");
        assert!(matches!(
            FileConfig::from_file(file_path),
            Err(ConfigError::FileNotFoundError(_))
        ));
    }

    #[test]
    fn test_invalid_file_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tmp.toml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "not = [valid").unwrap();
        assert!(matches!(
            FileConfig::from_file(&file_path),
            Err(ConfigError::InvalidFile(_, _))
        ));
    }
}
