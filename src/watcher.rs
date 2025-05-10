use super::error;
use crate::analyzer::actions::handle_file_changes;
use crate::analyzer::custom_rules;
use crate::app_state::SharedAppState;
use crate::cli::MergedConfig;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use regex::Regex;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

pub fn start_watching(
    config: &MergedConfig,
    running: &Arc<AtomicBool>,
    shared_state: SharedAppState,
) -> Result<()> {
    let (tx, rx) = channel::<Event>();
    let watch_extensions = config.watch_files.clone();
    let ignore_list = config.ignore_patterns.clone();
    let grumpiness_level = config.grumpiness_level.clone();
    let max_cyclomatic_complexity = config.max_complexity.clone();
    let max_function_size = config.max_function_size.clone();
    let custom_rules_file = config.custom_rules.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event>| {
            if let Ok(event) = res {
                if tx.send(event).is_err() {
                    error!("Failed to send event to main thread");
                }
            } else {
                error!("Watch error: {:?}", res.err());
            }
        },
        Config::default(),
    )?;

    watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;

    let mut last_triggered = Instant::now() - Duration::from_secs(10);
    let debounce_interval = Duration::from_secs(10);

    while running.load(Ordering::SeqCst) {
        if let Ok(event) = rx.recv_timeout(Duration::from_secs(1)) {
            if let Some(path) = event.paths.first() {
                if shall_be_ignored(path, &ignore_list) {
                    continue;
                }

                if is_relevant(path, &watch_extensions) {
                    let now = Instant::now();
                    if now.duration_since(last_triggered) >= debounce_interval {
                        let message = handle_file_changes(
                            path,
                            &grumpiness_level,
                            &max_cyclomatic_complexity,
                            &max_function_size,
                            Path::new(&custom_rules_file),
                        );

                        // Update UI message
                        {
                            let mut state = shared_state.write().unwrap();
                            state.message = message;
                        }

                        last_triggered = now;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Check if a file has an extension matching one of the allowed watch types.
///
/// # Arguments
/// * `path` - Path to the changed file
/// * `watch_files` - List of file extensions to watch (e.g., `[".rs", ".toml"]`)
fn is_relevant(path: &Path, allowed_extensions: &Vec<String>) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        // Check both `.rs` and `rs` forms
        allowed_extensions
            .iter()
            .any(|e| e.trim_start_matches('.') == ext)
    } else {
        false
    }
}

/// Check if a file is in the ignore patterns
///
/// # Arguments
/// * `path` - Path to the changed file
/// * `ignore_patterns` - patterns to ignore (e.g., `["target/"]`)
fn shall_be_ignored(path: &Path, ignore_pattern: &Vec<String>) -> bool {
    ignore_pattern.iter().any(|pattern| {
        return Regex::new(pattern)
            .unwrap()
            .is_match(path.to_str().unwrap_or(""));
    })
}
