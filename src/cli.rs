#![allow(clippy::style)]

use crate::config::{ConfigError, FileConfig, GitIntegrationMode, GrumpinessLevel, OutputFormat};
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

    /// whether to print output with color
    #[argh(switch, short = 'c')]
    pub print_color: bool,

    /// path to user-defined rules file
    #[argh(option)]
    pub custom_rules: Option<String>,

    /// output format: text, json, or fancy
    #[argh(option)]
    pub output_format: Option<OutputFormat>,

    /// include git integration (stale commit checks)
    #[argh(option)]
    pub git_integration: Option<GitIntegrationMode>,

    /// maximum number of warnings before final complaint
    #[argh(option)]
    pub max_warnings: Option<u32>,

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
    pub print_color: bool,
    pub custom_rules: String,
    pub output_format: OutputFormat,
    pub git_integration: GitIntegrationMode,
    pub max_warnings: u32,
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

            print_color: cli.print_color
                || file.as_ref().and_then(|f| f.print_color).unwrap_or(false),

            custom_rules: cli
                .custom_rules
                .or_else(|| file.as_ref().and_then(|f| f.custom_rules.clone()))
                .unwrap_or("rules.toml".into()),

            output_format: cli
                .output_format
                .or_else(|| file.as_ref().and_then(|f| f.output_format.clone()))
                .unwrap_or(OutputFormat::Fancy),

            git_integration: cli
                .git_integration
                .or_else(|| file.as_ref().and_then(|f| f.git_integration.clone()))
                .unwrap_or(GitIntegrationMode::Never),

            max_warnings: cli
                .max_warnings
                .or_else(|| file.as_ref().and_then(|f| f.max_warnings))
                .unwrap_or(10),

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
        assert_eq!(config.git_integration, GitIntegrationMode::Never);
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
            print_color: Some(true),
            custom_rules: Some("custom.toml".into()),
            output_format: Some(OutputFormat::Json),
            git_integration: Some(GitIntegrationMode::Always),
            max_warnings: Some(7),
            rules_file: Some("rules.toml".into()),
        };
        let config = MergedConfig::from_sources(args, Some(file_config));
        assert_eq!(config.output_format, OutputFormat::Txt);
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
            print_color: Some(true),
            custom_rules: Some("custom.toml".into()),
            output_format: Some(OutputFormat::Json),
            git_integration: Some(GitIntegrationMode::Always),
            max_warnings: Some(7),
            rules_file: Some("rules.toml".into()),
        };
        let config = MergedConfig::from_sources(args, Some(file_config));
        assert!(matches!(
            config.validate(),
            Err(ConfigError::MissingWatchFiles)
        ))
    }
}
