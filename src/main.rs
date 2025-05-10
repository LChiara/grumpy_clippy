mod analyzer;
mod app_state;
mod cli;
mod config;
mod logger;
mod ui;
mod watcher;

use crate::logger::config::Config as LoggerConfig;
use crate::logger::init_logger;
use app_state::new_shared_state;

use ctrlc;
use eframe::egui;
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn setup_shutdown() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("👋👋👋\nReceived termination signal. Shutting down...\n👋👋👋");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    running
}

fn initialize_logger() {
    let logger_config = LoggerConfig::from_file("logger_config.toml")
        .map(|c| c.finalize())
        .unwrap_or_else(|_| {
            info!("Logger config missing. Using default...");
            LoggerConfig {
                log_type: Some("txt".to_string()),
                file_name: Some("grumpy_clippy_log".to_string()),
                min_level: Some("info".to_string()),
                to_file: Some(false),
            }
            .finalize()
        });

    init_logger(logger_config);

    info!("GrumpyClippy started successfully!");
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logger first
    initialize_logger();

    // Shared app state
    let app_state = new_shared_state();

    // Run the watcher in a background thread
    let running = setup_shutdown();
    let run_flag = running.clone();
    let state_for_watcher = app_state.clone();

    std::thread::spawn(move || {
        let cli = argh::from_env::<cli::CliArgs>();
        let file_config = cli.config_file.as_deref().map(Path::new).and_then(|path| {
            match config::FileConfig::from_file(path) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    eprintln!("Error reading config file: {}", e);
                    None
                }
            }
        });

        let merged_config = cli::MergedConfig::from_sources(cli, file_config);

        if let Err(e) = merged_config.validate() {
            eprintln!("❌ Config error: {}", e);
            run_flag.store(false, Ordering::SeqCst);
            return;
        }

        if let Err(e) = watcher::start_watching(&merged_config, &run_flag, state_for_watcher) {
            eprintln!("❌ Failed to start watcher: {}", e);
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_position([20.0, 20.0])
            .with_inner_size([450.0, 170.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Grumpy Clippy GUI",
        options,
        Box::new(|cc| Ok(Box::new(ui::ClippyApp::new(cc, app_state, running)))),
    )
}
