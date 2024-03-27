use serde_derive::Deserialize;
use std::env;
use std::time::Duration;

// Represents the application configuration, sourced from environment variables.
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub ports: Vec<String>, // Ports the application will listen on or interact with
    pub active: bool, // Indicates if the application is active or in a dormant state
    pub scan_ports: Vec<String>, // Ports the application will scan, if any
}

impl AppConfig {
    // Constructs a new AppConfig by loading environment variables.
    pub fn new() -> Result<Self, env::VarError> {
        dotenv::dotenv().ok(); // Attempt to load .env file, if present

        // Extracts ports from the PORTS environment variable, split by comma
        let ports_str = env::var("PORTS")?;
        let ports = ports_str.split(',').map(String::from).collect();

        // Determines if the application is active based on the ACTIVE environment variable
        let active = env::var("ACTIVE").unwrap_or_else(|_| "false".to_string()) == "true";

        // Extracts scan ports from the SCAN_PORTS environment variable, split by comma
        let scan_ports_str = env::var("SCAN_PORTS").unwrap_or_else(|_| "".to_string());
        let scan_ports = scan_ports_str.split(',').map(String::from).collect();

        Ok(AppConfig { ports, active, scan_ports })
    }
}

// Retrieves the connection timeout duration from environment variables.
pub fn get_connection_timeout() -> Duration {
    dotenv::dotenv().ok(); // Attempt to load .env file, if present

    // Parses the CONNECTION_TIMEOUT_SECS environment variable to get timeout duration
    let timeout_secs = env::var("CONNECTION_TIMEOUT_SECS")
        .unwrap_or_else(|_| "30".to_string()) // Defaults to 30 seconds if not set
        .parse::<u64>()
        .expect("CONNECTION_TIMEOUT_SECS must be a positive integer");

    Duration::from_secs(timeout_secs)
}
