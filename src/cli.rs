#![allow(clippy::style)]

/// Represents the command-line arguments for configuring and running GrumpyClippy.
///
/// This structure is used to parse and store CLI arguments provided by the user.
/// It supports options for specifying configuration files, grumpiness levels, verbosity,
/// file watching patterns, and other settings.
///
/// # Fields
///
/// * `config_file` - Optional path to the configuration file.
/// * `grumpiness_level` - Optional level of grumpiness (`mild`, `sarcastic`, or `rude`).
/// * `verbose` - Flag to enable verbose output for suggestions.
/// * `watch_files` - List of file types or patterns to monitor for changes.
/// * `ignore_patterns` - List of file types or patterns to ignore during file monitoring.
/// * `max_function_size` - Optional maximum number of lines allowed in a function.
/// * `max_complexity` - Optional maximum cyclomatic complexity allowed.
/// * `custom_rules` - Optional path to a user-defined rules file.
/// * `git_integration` - Flag to enable Git integration for stale commit checks.
/// * `rules_file` - Optional path to an external rules file.
///
/// # Example
///
/// ```
/// let args = CliArgs {
///     config_file: Some("config.toml".to_string()),
///     grumpiness_level: Some(GrumpinessLevel::Sarcastic),
///     verbose: true,
///     watch_files: vec!["*.rs".to_string()],
///     ignore_patterns: vec!["target/".to_string()],
///     max_function_size: Some(50),
///     max_complexity: Some(10),
///     custom_rules: Some("custom_rules.toml".to_string()),
///     git_integration: true,
///     rules_file: Some("rules.toml".to_string()),
/// };
/// ```
///
/// ---
///
/// Represents the final merged configuration for GrumpyClippy.
///
/// This structure combines CLI arguments and configuration file settings to produce
/// a unified configuration. CLI arguments take precedence over file settings.
///
/// # Fields
///
/// * `grumpiness_level` - The level of grumpiness (`mild`, `sarcastic`, or `rude`).
/// * `verbose` - Whether verbose output is enabled.
/// * `watch_files` - List of file types or patterns to monitor for changes.
/// * `ignore_patterns` - List of file types or patterns to ignore during file monitoring.
/// * `max_function_size` - Maximum number of lines allowed in a function.
/// * `max_complexity` - Maximum cyclomatic complexity allowed.
/// * `custom_rules` - Path to a user-defined rules file.
/// * `git_integration` - Whether Git integration is enabled.
/// * `rules_file` - Path to an external rules file.
///
/// # Methods
///
/// * `from_sources(cli: CliArgs, file: Option<FileConfig>) -> Self`
///   - Merges CLI arguments and configuration file settings into a single configuration.
/// * `validate(&self) -> Result<(), ConfigError>`
///   - Validates the configuration, ensuring all required fields are set and values are within acceptable ranges.
///
/// # Example
///
/// ```
/// let cli_args = CliArgs {
///     grumpiness_level: Some(GrumpinessLevel::Rude),
///     verbose: true,
///     watch_files: vec!["*.rs".to_string()],
///     ignore_patterns: vec!["target/".to_string()],
///     max_function_size: Some(50),
///     max_complexity: Some(10),
///     custom_rules: Some("custom_rules.toml".to_string()),
///     git_integration: true,
///     rules_file: Some("rules.toml".to_string()),
/// };
///
/// let file_config = FileConfig {
///     grumpiness_level: Some(GrumpinessLevel::Sarcastic),
///     verbose: Some(false),
///     watch_files: Some(vec!["*.md".to_string()]),
///     ignore_patterns: None,
///     max_function_size: Some(40),
///     max_complexity: Some(8),
///     custom_rules: Some("file_rules.toml".to_string()),
///     git_integration: Some(false),
///     rules_file: Some("file_rules.toml".to_string()),
/// };
///
/// let merged_config = MergedConfig::from_sources(cli_args, Some(file_config));
/// assert_eq!(merged_config.grumpiness_level, GrumpinessLevel::Rude);
/// ```

use crate::config::{ConfigError, FileConfig, GrumpinessLevel};
use argh::FromArgs;

/// CLI to start and configure GrumpyClippy
#[derive(FromArgs, Debug)]
#[argh(description = "GrumpyClippy: A sarcastic Rust code analyzer")]
pub struct CliArgs {
    /// path to config file
    #[argh(option)]
    pub config_file: Option<String>,

    /// the level of grumpiness: mild, sarcastic, or rude
    #[argh(option)]
    pub grumpiness_level: Option<GrumpinessLevel>,

    #[argh(switch, short = 'v')]
    /// provide additional details for suggestions
    pub verbose: bool,

    /// list of file types or patterns to watch for changes
    #[argh(option)]
    pub watch_files: Vec<String>,

    /// list of file types or patterns that shall be ignored during watch for changes
    #[argh(option)]
    pub ignore_patterns: Vec<String>,

    /// maximum lines for function
    #[argh(option)]
    pub max_function_size: Option<u8>,

    /// maximum cyclomatic complexity
    #[argh(option)]
    pub max_complexity: Option<u8>,

    /// path to user-defined rules file
    #[argh(option)]
    pub custom_rules: Option<String>,

    /// include git integration (stale commit checks)
    #[argh(switch, short = 'g')]
    pub git_integration: bool,

    /// path to external rules file
    #[argh(option)]
    pub rules_file: Option<String>,
}

/// Final merged config: cli args >> config file
#[derive(Debug)]
pub struct MergedConfig {
    pub grumpiness_level: GrumpinessLevel,
    pub verbose: bool,
    pub watch_files: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub max_function_size: u8,
    pub max_complexity: u8,
    pub custom_rules: String,
    pub git_integration: bool,
    pub rules_file: String,
}

impl MergedConfig {
    pub fn from_sources(cli: CliArgs, file: Option<FileConfig>) -> Self {
        MergedConfig {
            grumpiness_level: cli
                .grumpiness_level
                .or_else(|| file.as_ref().and_then(|f| f.grumpiness_level.clone()))
                .unwrap_or(GrumpinessLevel::Mild),

            verbose: cli.verbose || file.as_ref().and_then(|f| f.verbose).unwrap_or(false),

            watch_files: if !cli.watch_files.is_empty() {
                Some(cli.watch_files.clone())
            } else {
                file.as_ref().and_then(|f| f.watch_files.clone())
            }
            .unwrap_or_else(|| vec![".rs".into(), ".toml".into()]),

            ignore_patterns: if !cli.ignore_patterns.is_empty() {
                Some(cli.ignore_patterns.clone())
            } else {
                file.as_ref().and_then(|f| f.ignore_patterns.clone())
            }
            .unwrap_or_else(|| vec!["target/".into()]),

            max_function_size: cli
                .max_function_size
                .or_else(|| file.as_ref().and_then(|f| f.max_function_size))
                .unwrap_or(32),

            max_complexity: cli
                .max_complexity
                .or_else(|| file.as_ref().and_then(|f| f.max_complexity))
                .unwrap_or(32),

            custom_rules: cli
                .custom_rules
                .or_else(|| file.as_ref().and_then(|f| f.custom_rules.clone()))
                .unwrap_or("rules.toml".into()),

            git_integration: cli.git_integration
                || file.as_ref().and_then(|f| f.git_integration).unwrap_or(false),

            rules_file: cli
                .rules_file
                .or_else(|| file.as_ref().and_then(|f| f.rules_file.clone()))
                .unwrap_or_else(|| "my_custom_rules.toml".into()),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_function_size == 0 {
            return Err(ConfigError::ValueTooSmall(
                "max_function_size".to_owned(),
                self.max_function_size,
                0,
            ));
        }
        if self.max_complexity == 0 {
            return Err(ConfigError::ValueTooSmall(
                "max_complexity".to_owned(),
                self.max_complexity,
                0,
            ));
        }
        if self.watch_files.is_empty() {
            return Err(ConfigError::MissingWatchFiles);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argh::FromArgs;

    fn parse_args(args: &[&str]) -> CliArgs {
        // Create a full list of arguments including a fake binary name as the first argument.
        // `argh` expects the first argument to be the binary name (e.g., "grumpy_clippy").
        let full_args: Vec<String> = std::iter::once("grumpy_clippy") // add fake binary name
            .chain(args.iter().cloned()) // clone each &str argument into a new iterator
            .map(String::from) // convert &str into String
            .collect(); // collect into a Vec<String>

        // Convert Vec<String> to Vec<&str> because `argh::from_args` expects `&[&str]`.
        let full_args_ref: Vec<&str> = full_args
            .iter()
            .map(|s| s.as_str()) // get &str from each String
            .collect();

        // Use argh to parse the CLI arguments from the generated &[&str] slice.
        // The first argument is the binary name; the rest are actual CLI arguments.
        CliArgs::from_args(&[full_args_ref[0]], &full_args_ref[1..])
            .expect("Failed to parse CLI arguments")
    }

    #[test]
    fn test_cli_defaults() {
        let args = parse_args(&[]);
        let config = MergedConfig::from_sources(args, None);
        assert_eq!(config.grumpiness_level, GrumpinessLevel::Mild);
        assert_eq!(config.git_integration, false);
        assert_eq!(config.max_complexity, 32)
    }

    #[test]
    fn test_paramters_defined_per_cli() {
        let args = parse_args(&["--grumpiness-level", "rude"]);
        let config = MergedConfig::from_sources(args, None);
        assert_eq!(config.grumpiness_level, GrumpinessLevel::Rude);
    }

    #[test]
    fn test_parameters_defined_per_cli_and_config_file() {
        let args = parse_args(&["--output-format", "txt", "--max-complexity", "8"]);
        let file_config = FileConfig {
            grumpiness_level: Some(GrumpinessLevel::Sarcastic),
            verbose: Some(false),
            watch_files: Some(vec!["*.md".into()]),
            ignore_patterns: None,
            max_function_size: Some(50),
            max_complexity: Some(5),
            custom_rules: Some("custom.toml".into()),
            git_integration: Some(false),
            rules_file: Some("rules.toml".into()),
        };
        let config = MergedConfig::from_sources(args, Some(file_config));
        assert_eq!(config.grumpiness_level, GrumpinessLevel::Sarcastic);
        assert_eq!(config.max_complexity, 8)
    }

    #[test]
    fn test_validation_error_max_function_size() {
        let args = parse_args(&["--max-function-size", "0"]);
        let config = MergedConfig::from_sources(args, None);
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValueTooSmall(_, _, _))
        ))
    }

    #[test]
    fn test_validation_error_max_complexity() {
        let args = parse_args(&["--max-complexity", "0"]);
        let config = MergedConfig::from_sources(args, None);
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ValueTooSmall(_, _, _))
        ))
    }

    #[test]
    fn test_validation_error_empty_watch_files() {
        let args = parse_args(&[]);
        let file_config = FileConfig {
            grumpiness_level: Some(GrumpinessLevel::Sarcastic),
            verbose: Some(false),
            watch_files: Some(vec![]),
            ignore_patterns: None,
            max_function_size: Some(50),
            max_complexity: Some(5),
            custom_rules: Some("custom.toml".into()),
            git_integration: Some(false),
            rules_file: Some("rules.toml".into()),
        };
        let config = MergedConfig::from_sources(args, Some(file_config));
        assert!(matches!(
            config.validate(),
            Err(ConfigError::MissingWatchFiles)
        ))
    }
}
