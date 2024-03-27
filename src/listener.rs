// listener.rs
// This module is responsible for listening for and handling incoming TCP connections.
// It supports listening on multiple ports and port ranges, handling each connection asynchronously.
// Dependencies: Tokio for async runtime, log for logging, and custom modules for application configuration and scanning functionality.

use crate::config::get_connection_timeout;
use crate::config::AppConfig;
use crate::scanner::scan_ports;
use log::{error, info, warn};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::time::{timeout, Duration, Instant};

// Starts listening on the specified ports and handles incoming connections.
// Ports can be specified individually or as ranges. Listens until a shutdown signal is received.
pub async fn start_listeners(
    ports: Vec<String>, // Vector of port specifications, either single ports or ranges.
    shutdown_signal: Sender<()>, // Channel to signal listener shutdown.
    app_config: Arc<AppConfig>, // Shared application configuration.
) {
    // Parse ports and start listening
    for port_spec in ports.iter() {
        match port_spec.parse::<u16>() {
            Ok(port) => match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
                Ok(listener) => {
                    info!("Listening on port {}", port);
                    let shutdown_signal_clone = shutdown_signal.clone();
                    let app_config_clone = app_config.clone();
                    tokio::spawn(async move {
                        handle_connections(listener, shutdown_signal_clone, app_config_clone.clone())
                            .await;
                    });
                }
                Err(e) => error!("Failed to listen on port {}: {}", port, e),
            },
            Err(_) => {
                if let Some(range) = port_spec.split_once('-') {
                    let start = range.0.parse::<u16>().expect("Invalid start of range");
                    let end = range.1.parse::<u16>().expect("Invalid end of range");
                    for port in start..=end {
                        match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
                            Ok(listener) => {
                                info!("Listening on port {}", port);
                                let shutdown_signal_clone = shutdown_signal.clone();
                                let app_config_clone = app_config.clone();
                                tokio::spawn(async move {
                                    handle_connections(
                                        listener,
                                        shutdown_signal_clone,
                                        app_config_clone.clone(),
                                    )
                                    .await;
                                });
                            }
                            Err(e) => error!("Failed to listen on port {}: {}", port, e),
                        }
                    }
                } else {
                    error!("Invalid port specification: {}", port_spec);
                }
            }
        }
    }

    // Wait for the shutdown signal
    let _ = shutdown_signal.closed().await;
    info!("Shutdown signal received, stopping all listeners.");
}

// Handles incoming connections for a given TcpListener. Listens for a shutdown signal to terminate.
async fn handle_connections(
    listener: TcpListener, // The TcpListener to accept connections from.
    shutdown_signal: Sender<()>, // Channel to signal handler shutdown.
    app_config: Arc<AppConfig>, // Shared application configuration.
) {
    let timeout_duration = get_connection_timeout(); // Get the timeout duration here
    

    loop {
        let app_config_clone = app_config.clone();
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                info!("Accepted connection from: {}", addr);
                let timeout_duration = timeout_duration; // Copy for the spawned task
                
                tokio::spawn(async move {
                    if let Err(e) = process_connection(socket, timeout_duration, app_config_clone.clone()).await {
                        warn!("Failed to process connection: {}", e);
                    }
                });
            },
            _ = shutdown_signal.closed() => {
                info!("Shutdown signal for listener received.");
                return;
            },
        }
    }
}

// Processes an individual connection, performing logging, optional scanning, and echoing data.
async fn process_connection(
    mut socket: TcpStream, // The TcpStream for the connection to process.
    timeout_duration: Duration, // Duration to consider connection inactive and timeout.
    app_config: Arc<AppConfig>, // Shared application configuration.
) -> tokio::io::Result<()> {
    let peer_addr = match socket.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return Ok(()), // Early return if we can't get the peer address
    };

    let mut log_path = PathBuf::from("logs");
    std::fs::create_dir_all(&log_path).unwrap_or_else(|_| panic!("Failed to create log directory"));
    log_path.push(format!("{}.log", peer_addr.ip()));

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap_or_else(|_| panic!("Failed to open log file"));

    writeln!(
        log_file,
        "Connection from: {}. Timestamp: {:?}",
        peer_addr,
        Instant::now()
    )
    .unwrap();

    if app_config.active {
        let ip = peer_addr.ip().to_string();
        tokio::spawn(scan_ports(ip, app_config.clone()));
    }

    let mut buf = [0u8; 1024];
    loop {
        match timeout(timeout_duration, socket.read(&mut buf)).await {
            Ok(Ok(0)) => {
                writeln!(
                    log_file,
                    "Connection closed by client. Timestamp: {:?}",
                    Instant::now()
                )
                .unwrap();
                return Ok(());
            }
            Ok(Ok(nbytes)) => {
                writeln!(
                    log_file,
                    "Received data at Timestamp: {:?}. Data: {:?}",
                    Instant::now(),
                    &buf[..nbytes]
                )
                .unwrap();
                if nbytes > 0 {
                    socket.write_all(&buf[..nbytes]).await?;
                }
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                writeln!(
                    log_file,
                    "Connection timed out due to inactivity. Timestamp: {:?}",
                    Instant::now()
                )
                .unwrap();
                return Ok(());
            }
        }
    }
}
