use std::fs;
use std::io::{self};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use super::{error, info, warning};
use crate::config::GrumpinessLevel;
use crate::inspector;

/// Do some actions basing on the grumpiness level when file change detected
///
/// # Arguments:
/// * `path` - Path to the changed filed
/// * `grumpiness_level` - level of grumpiness to use to generate messages
pub fn handle_file_changes(
    path: &Path,
    grumpiness_level: &GrumpinessLevel,
    max_function_size: &u8,
    max_cyclomatic_complexity: &u8,
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
                info_messages.push_str(&get_clippy_message_successful(grumpiness_level));
            } else if match_path(path, &stderr) {
                warning_messages.push_str(&get_clippy_message_failed(grumpiness_level));
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
    match analyze_file(path, max_function_size, max_cyclomatic_complexity) {
        Ok((status, messages)) => {
            if !status {
                warning_messages.push_str(&messages);
            }
        }
        Err(err) => {
            error_messages.push_str(&format!("❌ Failed to analyse file: {}\n", err));
        }
    };

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

    //todo!("Fix the logic!");

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

fn get_clippy_message_successful(level: &GrumpinessLevel) -> String {
    let message = match level {
        GrumpinessLevel::Mild => "✅ cargo clippy succcessful",
        GrumpinessLevel::Sarcastic => "✅🙈 Oh, you did not break anything. Strange!",
        GrumpinessLevel::Rude => {
            "✅🙄 Oh, you managed not to break anything? Well, there is a first time for everything."
        }
    };
    message.to_string() + "\n"
}

fn get_clippy_message_failed(level: &GrumpinessLevel) -> String {
    let message = match level {
        GrumpinessLevel::Mild => "❌ Clippy failed (see terminal for details)",
        GrumpinessLevel::Sarcastic => "❌🙄 Oh, you did break something (as usual):",
        GrumpinessLevel::Rude => "❌💣 Of course you broke something—how utterly predictable.",
    };
    message.to_string() + "\n"
}

fn analyze_file(
    path: &Path,
    max_function_size: &u8,
    max_cyclomatic_complexity: &u8,
) -> Result<(bool, String), String> {
    let mut successful = true;
    let mut messages = String::new();

    let code = fs::read_to_string(path).expect("Failed to read file");
    let syntax = syn::parse_file(&code).expect("Syntax error");

    let metrics = inspector::analyze_file(&syntax);
    for m in metrics {
        if m.cyclomatic_complexity as u8 > *max_cyclomatic_complexity {
            messages.push_str(&format!(
                "Function '{}': cyclomatic complexity exceeding ({}>{})\n",
                m.name, max_cyclomatic_complexity, m.cyclomatic_complexity
            ));
            successful = false;
        }
        if m.lines_of_code as u8 > *max_function_size {
            messages.push_str(&format!(
                "Function '{}': max lines of code exceeding ({}>{})\n",
                m.name, max_function_size, m.lines_of_code
            ));
            successful = false;
        }
    }
    Ok((successful, messages))
}
