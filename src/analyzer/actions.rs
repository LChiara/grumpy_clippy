/// Handles file changes by performing various analyses and checks on the given file.
///
/// This function performs the following tasks:
/// - Runs `cargo fmt` to format the file.
/// - Runs `cargo clippy` to check for linting issues.
/// - Analyzes the file's complexity, including cyclomatic complexity and function size.
/// - Applies custom rules defined in a TOML file.
/// - Checks the file's status in the Git repository, including staleness and most frequent author.
///
/// # Arguments
///
/// * `path` - The path to the file being analyzed.
/// * `grumpiness_level` - The level of grumpiness, which affects the tone of messages.
/// * `max_function_size` - The maximum allowed size of a function in lines of code.
/// * `max_cyclomatic_complexity` - The maximum allowed cyclomatic complexity of a function.
/// * `custom_rules_path` - The path to the TOML file containing custom rules.
///
/// # Returns
///
/// A `String` containing informational, warning, and error messages generated during the analysis.
///
/// # Errors
///
/// This function may return error messages if:
/// - `cargo fmt` or `cargo clippy` fails to execute.
/// - File complexity analysis encounters issues.
/// - Custom rules analysis fails.
/// - Git operations fail.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use crate::config::GrumpinessLevel;
///
/// let path = Path::new("src/main.rs");
/// let grumpiness_level = GrumpinessLevel::High;
/// let max_function_size = 50;
/// let max_cyclomatic_complexity = 10;
/// let custom_rules_path = Path::new("custom_rules.toml");
///
/// let messages = handle_file_changes(
///     path,
///     &grumpiness_level,
///     &max_function_size,
///     &max_cyclomatic_complexity,
///     custom_rules_path,
/// );
/// println!("{}", messages);
/// ```

use std::fs;
use std::io::{self};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use crate::analyzer::complexity_inspector;
use crate::analyzer::custom_rules::*;
use crate::analyzer::git;
use crate::analyzer::messages::*;
use crate::config::GrumpinessLevel;
use crate::{error, info, warning};

pub fn handle_file_changes(
    path: &Path,
    grumpiness_level: &GrumpinessLevel,
    max_function_size: &u8,
    max_cyclomatic_complexity: &u8,
    custom_rules_path: &Path,
) -> String {
    let mut info_messages = format!(
        "Detected changes in '{:?}'\n",
        extract_path_from_src(path).unwrap_or("".to_string())
    );
    let mut warning_messages = String::new();
    let mut error_messages = String::new();

    match run_fmt(path) {
        Ok((_, _)) => {
            info_messages.push_str("✅ cargo fmt successful!\n");
        }
        Err(e) => {
            error_messages.push_str(&format!("❌ Failed to run 'cargo fmt': {}\n", e));
        }
    };
    match run_clippy() {
        Ok((status, stderr_bytes)) => {
            let stderr = String::from_utf8_lossy(&stderr_bytes);

            if status.success() {
                info_messages.push_str(clippy::success(grumpiness_level));
            } else if match_path(path, &stderr) {
                warning_messages.push_str(clippy::failure(grumpiness_level));
                warning!(
                    "{:?}\n",
                    extract_clippy_error_for_path(
                        &stderr,
                        &extract_path_from_src(path).unwrap_or("".to_string())
                    )
                    .unwrap_or_default()
                );
            }
        }
        Err(err) => {
            error_messages.push_str(&format!("❌ Failed to run 'clippy': {}\n", err));
        }
    };
    match analyze_file_complexity(
        path,
        grumpiness_level,
        max_function_size,
        max_cyclomatic_complexity,
    ) {
        Ok((status, messages)) => {
            if !status {
                warning_messages.push_str(&messages);
            }
        }
        Err(err) => {
            error_messages.push_str(&format!(
                "❌ Failed to analyse file with custom rules: {}\n",
                err
            ));
        }
    };
    match analyze_file_with_custom_rules(path, custom_rules_path) {
        Ok((status, messages)) => {
            if !status {
                warning_messages.push_str(&messages.join("\n"));
            }
        }
        Err(err) => {
            error_messages.push_str(&format!("❌ Failed to analyse file: {}\n", err));
        }
    };
    match git::GitInspector::new(path) {
        Ok(tgit_inspector) => {
            match tgit_inspector.is_file_stale(path, 7) {
                Ok(true) => {
                    info_messages.push_str(&git_is_stale::info(
                        grumpiness_level,
                    ));
                }
                Ok(false) => (),
                Err(e) => {
                    error_messages
                        .push_str(&format!("❌ Failed to check if file is stale: {}\n", e));
                }
            }
            match tgit_inspector.most_frequent_author(path) {
                Ok(author) => {
                    info_messages.push_str(&git_most_frequent_author::info(
                        grumpiness_level,
                        author.as_deref().unwrap_or(""),
                    ));
                }
                Err(e) => {
                    error_messages
                        .push_str(&format!("❌ Failed to get most frequent author: {}\n", e));
                }
            }
        }
        Err(e) => {
            error_messages.push_str(&format!("❌ Failed to create GitInspector: {}\n", e));
        }
    }

    if !info_messages.is_empty() {
        for line in info_messages.lines() {
            info!("{}", line);
        }
    }
    if !warning_messages.is_empty() {
        for line in warning_messages.lines() {
            warning!("{}", line);
        }
    }
    if !error_messages.is_empty() {
        for line in error_messages.lines() {
            error!("{}", line);
        }
    }

    println!("Generated messages: {}", info_messages);
    println!("Generated messages: {}", warning_messages);
    println!("Generated messages: {}", error_messages);

    info_messages + &warning_messages + &error_messages
}

fn run_cmd(mut cmd: Command) -> io::Result<(ExitStatus, Vec<u8>)> {
    let process = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

    let output = process.wait_with_output()?;

    let status = output.status;
    let stderr = output.stderr;

    Ok((status, stderr))
}

fn run_fmt(path: &Path) -> io::Result<(ExitStatus, Vec<u8>)> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");
    cmd.arg("--");
    cmd.arg(path.to_str().unwrap_or(""));
    run_cmd(cmd)
}

fn run_clippy() -> io::Result<(ExitStatus, Vec<u8>)> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    cmd.arg("--all-targets");
    cmd.arg("--all-features");
    cmd.arg("--");
    cmd.arg("-Dwarnings");
    run_cmd(cmd)
}

fn extract_path_from_src(path: &Path) -> Option<String> {
    let delimiter = "src/".to_string();
    path.to_str()
        .unwrap_or("")
        .split_once(&delimiter)
        .map(|(_, rest)| format!("src/{}", rest)) // return owned String
}

fn match_path(path: &Path, std_err: &str) -> bool {
    if let Some(relative_path) = extract_path_from_src(&path) {
        return std_err.contains(&format!("--> {}", relative_path))
            || std_err.contains(&format!("--> {}", path.display()));
    } else {
        false
    }
}

fn extract_clippy_error_for_path<'a>(stderr: &'a str, path: &str) -> Option<&'a str> {
    let mut lines: std::iter::Peekable<std::str::Lines<'_>> = stderr.lines().peekable();
    let mut collecting = false;
    let mut start = 0;
    let mut end = 0;
    let mut current_index = 0;

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with("--> ") {
            if line.contains(path) {
                collecting = true;
                start = current_index;
            } else if collecting {
                end = current_index;
                break;
            }
        }

        if collecting {
            end = current_index + 1;
        }

        current_index += line.len() + 1; // +1 for the newline
    }

    if collecting {
        Some(&stderr[start..end])
    } else {
        None
    }
}

fn analyze_file_complexity(
    path: &Path,
    grumpiness_level: &GrumpinessLevel,
    max_function_size: &u8,
    max_cyclomatic_complexity: &u8,
) -> Result<(bool, String), String> {
    let mut successful = true;
    let mut messages = String::new();

    let code = fs::read_to_string(path).expect("Failed to read file");
    let syntax = syn::parse_file(&code).expect("Syntax error");

    let metrics = complexity_inspector::analyze_file(&syntax);
    for m in metrics {
        if m.cyclomatic_complexity as u8 > *max_cyclomatic_complexity {
            messages.push_str(&complexity::warning(
                &grumpiness_level,
                &m.name,
                m.cyclomatic_complexity,
                *max_cyclomatic_complexity,
            ));
            messages.push('\n');
            successful = false;
        }
        if m.lines_of_code as u8 > *max_function_size {
            messages.push_str(&function_size::warning(
                &grumpiness_level,
                &m.name,
                m.lines_of_code,
                *max_function_size,
            ));
            messages.push('\n');
            successful = false;
        }
    }
    Ok((successful, messages))
}

fn analyze_file_with_custom_rules(
    path: &Path,
    custom_rules_path: &Path,
) -> Result<(bool, Vec<String>), String> {
    let code = fs::read_to_string(path).expect("Failed to read file");
    match load_custom_rules_from_toml(custom_rules_path.to_str().unwrap()) {
        Ok(Some(rules)) => apply_rules(rules, &code),
        Ok(None) => {
            info!("No custom rules found, skipping custom rules analysis.");
            Ok((true, vec![])) // No rules means no issues
        },
        Err(e) => return Err(format!("Failed to load custom rules: {}", e)),
    }
}
