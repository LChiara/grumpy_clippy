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

/// Enum representing different output format
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Txt,
    Json,
    Fancy,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for OutputFormat {
    type Err = ConfigError; // Custom Error Type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "txt" => Ok(OutputFormat::Txt),
            "json" => Ok(OutputFormat::Json),
            "fancy" => Ok(OutputFormat::Fancy),
            _ => Err(ConfigError::InvalidOutputFormat(s.to_string())),
        }
    }
}

/// Enum representing different modes of Git integration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GitIntegrationMode {
    Always,
    OnCommit,
    OnPush,
    Never,
}

impl fmt::Display for GitIntegrationMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for GitIntegrationMode {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(GitIntegrationMode::Always),
            "on_commit" => Ok(GitIntegrationMode::OnCommit),
            "on_push" => Ok(GitIntegrationMode::OnPush),
            "never" => Ok(GitIntegrationMode::Never),
            _ => Err(ConfigError::InvalidGitIntegrationMode(s.to_string())),
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
    pub print_color: Option<bool>,
    pub custom_rules: Option<String>,
    pub output_format: Option<OutputFormat>,
    pub git_integration: Option<GitIntegrationMode>,
    pub max_warnings: Option<u32>,
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
    InvalidGitIntegrationMode(String),
    InvalidOutputFormat(String),
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
            ConfigError::InvalidOutputFormat(value) => {
                write!(
                    f,
                    "OutputFormat must be 'txt', 'json', or 'fancy', but got {}",
                    value
                )
            }
            ConfigError::InvalidGitIntegrationMode(value) => {
                write!(
                    f,
                    "GitIntegration must be 'never', 'always', 'onpush' or 'oncommit', but got {}",
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
    use crate::config::{ConfigError, GitIntegrationMode, GrumpinessLevel, OutputFormat};
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
    fn test_output_format_parsing() {
        assert_eq!(OutputFormat::from_str("jSoN").unwrap(), OutputFormat::Json);
        assert_eq!(
            OutputFormat::from_str("fancy").unwrap(),
            OutputFormat::Fancy
        );
        assert_eq!(OutputFormat::from_str("TXT").unwrap(), OutputFormat::Txt);
        assert!(OutputFormat::from_str("_json").is_err())
    }

    #[test]
    fn test_git_integration_mode_parsing() {
        assert_eq!(
            GitIntegrationMode::from_str("always").unwrap(),
            GitIntegrationMode::Always
        );
        assert_eq!(
            GitIntegrationMode::from_str("NeVeR").unwrap(),
            GitIntegrationMode::Never
        );
        assert_eq!(
            GitIntegrationMode::from_str("ON_PUSH").unwrap(),
            GitIntegrationMode::OnPush
        );
        assert_eq!(
            GitIntegrationMode::from_str("on_Commit").unwrap(),
            GitIntegrationMode::OnCommit
        );
        assert!(GitIntegrationMode::from_str("oncommit").is_err());
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
        assert_eq!(config.output_format.unwrap(), OutputFormat::Json);
        assert_eq!(config.git_integration.unwrap(), GitIntegrationMode::Always);
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
