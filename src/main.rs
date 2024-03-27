// Import necessary modules and traits from local and external crates.
use crate::config::AppConfig;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;

// Include local module definitions.
mod config;
mod listener;
mod logger;
mod scanner;

// Entry point for the async application, powered by Tokio.
#[tokio::main]
async fn main() {
    // Initialize logging. If it fails, log the error and exit the application.
    if let Err(e) = logger::init() {
        println!("Failed to initialize logger: {}", e);
        return;
    }

    // Load application configuration into an Arc for shared access across async tasks.
    // If configuration loading fails, log the error and exit.
    let app_config = Arc::new(AppConfig::new().expect("Failed to load configuration"));

    // Create a channel for sending shutdown signals to other parts of the application.
    let (tx, mut rx) = mpsc::channel::<()>(32);

    // Clone the transmitter to be able to send the shutdown signal from different places.
    let shutdown_tx = tx.clone();

    // Start listening for incoming connections based on the configuration.
    // Pass a clone of the app configuration and the shutdown signal transmitter.
    listener::start_listeners(app_config.ports.clone(), tx, app_config.clone()).await;

    // Setup handling for Unix signals (SIGINT for interrupt, SIGTERM for terminate)
    // to gracefully shutdown the application. If binding signal handlers fails, log the error and exit.
    let mut sigint = match signal(SignalKind::interrupt()) {
        Ok(sig) => sig,
        Err(e) => {
            error!("Failed to bind SIGINT handler: {}", e);
            return;
        }
    };

    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sig) => sig,
        Err(e) => {
            error!("Failed to bind SIGTERM handler: {}", e);
            return;
        }
    };

    // Listen for the first signal received (SIGINT, SIGTERM, or an unexpected message)
    // and initiate the shutdown process accordingly.
    tokio::select! {
        _ = sigint.recv() => {
            info!("Received SIGINT, initiating shutdown...");
        },
        _ = sigterm.recv() => {
            info!("Received SIGTERM, initiating shutdown...");
        },
        _ = rx.recv() => {
            // This case is expected to never occur as it's just for unexpected messages.
            warn!("Unexpected message received, initiating shutdown...");
        }
    }

    // Drop the shutdown transmitter to signal all tasks to start their shutdown process.
    drop(shutdown_tx);

    // Here, one could wait for all tasks to complete their shutdown by coordinating
    // through another channel, ensuring a clean and orderly shutdown.

    // Log the completion of the application's shutdown process.
    info!("Application shutdown complete.");
}
