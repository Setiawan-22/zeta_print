# ZetaPrint 🖨️
**Print Management System for Isolated Networks & Manufacturing**

ZetaPrint is a high-performance, modular monolith print management system built with Rust and Svelte. It acts as an IPP Print Server that intercepts, analyzes (cost & quota), and dispatches print jobs to physical printers (both modern IPP and legacy PAPPL-based printers). 

## ✨ Features
- **Single Binary Deploy**: Entire system (Database, Web Server, Print Engine, SPA Frontend) compiled into a single static binary.
- **Embedded SvelteKit SPA**: Ultra-fast UI delivered directly from RAM via `rust-embed` without an external web server like NGINX.
- **Real-Time Dashboard**: Monitor print queues, system logs, and printer health dynamically using Server-Sent Events (SSE).
- **Advanced PDF Accounting**: Automatically parses print jobs (spool files) to calculate BW/Color pages and enforce department quotas.
- **Systemd Native**: Fully integrates with Linux `journalctl` for telemetry and runs natively on host networks for mDNS discovery.
- **Legacy Hardware Bridge**: Built-in support for PAPPL (Printer Application Framework) for dot-matrix and legacy USB printers.

## 🛠️ Technology Stack
- **Engine**: Rust (Edition 2021), Tokio (Async Runtime)
- **Web/API**: Axum, JWT (HttpOnly Cookies), SSE
- **Database**: Embedded SQLite (via SQLx)
- **Frontend**: Svelte 5, SvelteKit, Vanilla CSS (No Tailwind)
- **Protocols**: IPP (Internet Printing Protocol), PAPPL

## 📋 Prerequisites
Before installing, testing, or contributing to ZetaPrint, ensure your system has the following requirements:
- **Rust Toolchain**: `rustup`, `cargo`, and `rustc` (Edition 2021) installed.
- **Node.js & npm**: Required by the `build.rs` script to automatically compile the SvelteKit frontend during the Rust build process.
- **Linux Environment**: Recommended for deploying and testing the `systemd` service and `journalctl` logging natively.
- **C Compiler**: A basic C compiler (like `gcc` or `clang`) may be required for compiling SQLite and other C dependencies natively.

## 🚀 Installation & Setup
ZetaPrint is designed to be installed easily on Linux machines via `systemd`.

### 1. Install via Cargo
You can install ZetaPrint directly from the GitHub repository using a single `cargo install` command.

```bash
cargo install --git https://github.com/Setiawan-22/zeta_print.git
```
*(Make sure you have Rust installed on your system before running this command).*

### 2. Systemd Installation
ZetaPrint has a built-in command to automatically install itself as a systemd service, configure the `/var/lib/zetaprint/` data directory, and start the daemon.

```bash
# Run the built-in installer (requires root privileges)
sudo ./target/release/zetaprint install
```

This command will:
- Create the necessary `/var/lib/zetaprint` directories.
- Generate and place the `/etc/systemd/system/zetaprint.service` file.
- Enable and start the service on boot.

### 3. Verification
Check the status of the server:
```bash
sudo systemctl status zetaprint
```
View real-time system logs natively:
```bash
sudo journalctl -u zetaprint -f
```

Access the Web UI by navigating to the server's IP address (Port `8080` by default):
`http://<SERVER_IP>:8080/`

**Default Credentials**:
- **Username**: `sysadmin`
- **Password**: `admin123`
