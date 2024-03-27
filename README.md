OT Early Warning System or
Rust TCP Port Scanner and Listener Application
==============================================

This application is a Rust-based tool that listens on specified ports and can perform port scanning operations based on configurations set through environment variables. It's designed to be highly configurable, supporting dynamic adjustments via a .env file for port listening, connection timeout settings, port scanning, and logging levels.

WARNING
--------
This code should not be used in live production environments. You are welcome to contribute to make it production ready. I take no responsibilities. :P

Why?
--------
In OT environments, network traffic is strictly controlled and highly predictable, making it essential to promptly identify and manage any deviations or unauthorized access attempts. This Rust-based application was developed to provide a highly configurable tool for monitoring and securing such environments, leveraging Rust's performance and reliability to efficiently listen on specified ports and conduct port scans. It ensures operational integrity by allowing real-time adjustments and monitoring, catering to the unique needs of OT security.

Features
--------

-   **Listening on Multiple Ports**: The application can listen on multiple ports specified in the .env file.
-   **Connection Timeout**: Customizable timeout for connections to improve resource management.
-   **Port Scanning**: Ability to enable or disable port scanning and specify target ports for scanning, including support for well-known ports with necessary root privileges (Unix systems). **Please note that it is NOT recommended to run this application as root for security reasons. It is also not recommended to enable scan in OT environments since some sensitive systems can stop functioning correctly and this in turn can be used as a method of attack.***
-   **Logging**: Supports various logging levels to adjust the verbosity of application logs for better debugging and monitoring.

Configuration
-------------

Configuration is managed through a .env file located at the root of the project. Here's a breakdown of the configurable environment variables:

### `PORTS`

-   Specifies the ports the application will listen on.
-   Example: `PORTS=2000,3000-3005`

You need root privileges in order to use ports below 1024. This is not recommended.

### `CONNECTION_TIMEOUT_SECS`

-   Sets the timeout duration (in seconds) for connections.
-   Example: `CONNECTION_TIMEOUT_SECS=3`

### `ACTIVE`

-   Enables or disables port scanning functionality.
-   Example: `ACTIVE=false`

For this option to work you need to start the application as root or with sudo. This is not recommended.

### `SCAN_PORTS`

-   Defines the ports or range of ports to scan. Use commas to separate values and hyphens for ranges.
-   Example: `SCAN_PORTS=1025,1050-1060,2020,7331`

### `LOG_LEVEL`

-   Determines the logging level of the application. Supported levels are TRACE, DEBUG, INFO, WARN, ERROR, and FATAL.
-   Example: `LOG_LEVEL=INFO`

Getting Started
---------------

To get the application running on your system, follow these steps:

1.  **Clone the Repository**

    -   Use `git clone https://github.com/SpiffyFoxOne/OT-early-warning-system.git` and navigate into the project directory with `cd OT-early-warning-system`.
2.  **Setup `.env` File**

    -   Copy the `env.example` to a new `.env` file and adjust the values according to your needs with `cp env.example .env`.
3.  **Install Dependencies**

    -   Ensure you have Rust and Cargo installed on your machine. Then, install the required dependencies with `cargo build`.
4.  **Run the Application**

    -   Start the application with Cargo using `cargo run`.

Contributing
------------

Contributions to improve the application or add new features are welcome. Please follow the standard fork-branch-PR workflow:

1.  Fork the repository.
2.  Create a new branch for your feature.
3.  Implement your feature or fix.
4.  Submit a pull request.

License
-------

This software is released under the MIT License. You are free to use, modify, and distribute this software in any form, for any purpose, including commercial applications. We only ask that you include the original copyright and license notice in any copy of the software or substantial portion of it.

For the full terms of the license, please see the LICENSE file included with this distribution or visit https://opensource.org/licenses/MIT.

For more information or assistance, please open an issue in the GitHub repository.
