use crate::config::AppConfig;
use log::{error, info, warn};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::time::{timeout, Duration};
#[cfg(unix)]
use nix::unistd;

// Begins an asynchronous port scanning operation for a specified IP address.
pub async fn scan_ports(ip: String, app_config: Arc<AppConfig>) {
    // Skip scanning if it's disabled in the application configuration.
    if !app_config.active {
        info!("Port scanning is disabled.");
        return;
    }

    // Unix-specific: Check if any specified ports are well-known ports which require root privileges to scan.
    #[cfg(unix)]
    let needs_root = app_config.scan_ports.iter().any(|p| {
        p.parse::<u16>().map(|port| port <= 1024).unwrap_or_else(|_| {
            p.split_once('-').map_or(false, |(start, end)| {
                start.parse::<u16>().unwrap_or(0) <= 1024 || end.parse::<u16>().unwrap_or(0) <= 1024
            })
        })
    });

    // If root privileges are needed but not present, log an error and return.
    #[cfg(unix)]
    if needs_root && !check_root_privileges() {
        error!("Root privileges are required for scanning well-known ports.");
        return;
    }

    // Log the start of the scanning process.
    info!("Initiating port scan for: {}", ip);
    let mut log_path = PathBuf::from("logs");
    std::fs::create_dir_all(&log_path).expect("Failed to create log directory");
    log_path.push(format!("{}-scan.log", ip));

    // Prepare the log file for recording scan results.
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(log_path)
        .expect("Failed to open scan log file");

    // Iterate through each port or range of ports specified in the configuration.
    for port_spec in &app_config.scan_ports {
        // Handle port ranges specified with a dash (e.g., "1000-2000").
        if let Some(range) = port_spec.split_once('-') {
            let start = range.0.parse::<u16>().unwrap_or(0);
            let end = range.1.parse::<u16>().unwrap_or(0);
            for port in start..=end {
                scan_single_port(&ip, port, &mut log_file).await;
            }
        } else {
            // Handle individual ports.
            let port = port_spec.parse::<u16>().unwrap_or(0);
            if port != 0 {
                scan_single_port(&ip, port, &mut log_file).await;
            }
        }
    }
}

// Unix-specific function to check if the current process has root privileges.
#[cfg(unix)]
fn check_root_privileges() -> bool {
    unistd::geteuid().is_root()
}

// Scans an individual port on the given IP address and logs the outcome.
async fn scan_single_port(ip: &str, port: u16, log_file: &mut std::fs::File) {
    let addr = format!("{}:{}", ip, port);
    match TcpStream::connect(addr).await {
        Ok(mut stream) => {
            // If the port is open, log the success.
            let msg = format!("Port {} is open", port);
            info!("{}", &msg);
            writeln!(log_file, "{}", &msg).expect("Failed to write to scan log file");

            // Attempt to read data from the open port, logging any received data.
            let mut buffer = [0; 1024]; // Buffer for received data.
            match timeout(Duration::from_secs(5), stream.read(&mut buffer)).await {
                Ok(Ok(n)) if n > 0 => {
                    // If data is received, log the data.
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    let msg = format!("Received data from port {}: {}", port, data);
                    info!("{}", &msg);
                    writeln!(log_file, "{}", &msg).expect("Failed to write to scan log file");
                }
                _ => {
                    // If no data is received or the read times out, log the event.
                    let msg = "No immediate data received or read timed out";
                    writeln!(log_file, "[INFO] {}: {}", port, msg).expect("Failed to write to scan log file");
                }
            }
        },
        Err(e) => {
            // If the connection fails, log the failure.
            let msg = format!("Failed to connect to port {}: {}", port, e);
            writeln!(log_file, "[WARN] {}", &msg).expect("Failed to write to scan log file");
        }
    }
}
