# KNOWLEDGE INSTRUCTION (KI) - ZETAPRINT
# DOCUMENT VERSION: 1.1
# SYSTEM TYPE: PRINT MANAGEMENT SERVER (MODULAR MONOLITH)

## 1. SYSTEM OVERVIEW & DEPLOYMENT MANDATE
ZetaPrint adalah sistem manajemen dan pelacakan mesin cetak (Print Management System) berkinerja tinggi, dirancang khusus untuk lingkungan jaringan terisolasi dan manufaktur. 

**MANDAT DEPLOYMENT:** ZetaPrint DIWAJIBKAN untuk di-deploy sebagai **Single Binary Systemd Service**. Tidak diizinkan memecah komponen ke dalam microservices. Seluruh engine utama, database (SQLite), web server, frontend UI, dan jembatan perangkat keras legacy (PAPPL) harus dibungkus dalam satu binary (executable) demi kemudahan instalasi via `cargo install` dan efisiensi resource. Setiap melakukan perubahan atau penambahan fitur update di dokumen task.md

## 2. ARCHITECTURE PATTERN
- **Type:** Modular Monolith (Single Binary).
- **Frontend Delivery:** Embedded SPA (Single Page Application). SvelteKit di-compile secara statis (adapter-static) dan di-inject ke dalam Rust binary menggunakan `rust-embed`.
- **Process Manager:** `systemd` mengelola service ZetaPrint tunggal. Aplikasi ZetaPrint memiliki subcommand CLI (contoh: `zetaprint install` & `zetaprint start`) untuk mempermudah setup systemd, database SQLite, dan konfigurasi lainnya secara otomatis.

## 3. CORE TECHNOLOGY STACK
- **Engine / Backend:** Rust (Edition 2021)
- **Web Framework:** `axum` (Routing, REST API, Static File Serving, SSE)
- **Async Runtime:** `tokio` (Multithreading & I/O)
- **Print Protocol:** `ipp` crate (IPP Server & Client Implementation)
- **Document Parser:** `lopdf` (Evaluasi halaman dan deteksi warna pada Spool PDF)
- **Database:** SQLite (via `sqlx` untuk compile-time query checking, embedded)
- **Frontend:** Svelte 5 + SvelteKit (SPA Mode) + TailwindCSS
- **Legacy Hardware Bridge:** PAPPL (Printer Application Framework)

## 4. WORKSPACE & DIRECTORY STRUCTURE
Sistem menggunakan `Cargo Workspace` untuk memisahkan domain logika secara ketat namun tetap dikompilasi menjadi satu binary.

zetaprint-workspace/
├── Cargo.toml                  # Workspace root configuration
├── build.rs                    # Script untuk auto-build frontend (jika ada)
│
├── frontend/                   # SvelteKit SPA Project
│   ├── src/                    # UI Components, Pages, Stores
│   ├── svelte.config.js        # Configured with @sveltejs/adapter-static
│   └── package.json
│
├── crates/
│   ├── core_api/               # Modul Axum, HTTP Handlers, dan rust-embed
│   ├── core_ipp/               # Modul implementasi IPP Server (Listener Port 631)
│   ├── core_accounting/        # Modul lopdf untuk parsing PDF & Kalkulasi Biaya
│   ├── core_dispatcher/        # Modul routing file ke Printer IPP atau PAPPL
│   ├── core_db/                # Modul SQLx, Migrations, dan Entities
│   └── core_auth/              # Modul LDAP/Active Directory Sync
│
└── src/
    └── main.rs                 # Entry point: Menggabungkan semua crates & inisiasi Tokio

## 5. CORE WORKFLOWS

### A. Print Interception Flow (The Engine)
1. Klien (Windows/Mac) mengirim Print Job (PDF) via port 631 ke `core_ipp`.
2. `core_ipp` melempar file PDF ke dalam memori/temp-file.
3. `core_accounting` mengambil alih via `tokio::task::spawn_blocking` untuk membedah halaman PDF.
4. `core_db` memvalidasi saldo/kuota user.
5. Jika ditolak: Transaksi dibatalkan, status `Denied` dikirim via SSE ke UI.
6. Jika disetujui: Transaksi di-log sebagai `Processing`, file dilempar ke `core_dispatcher`.

### B. Hardware Dispatch Flow (The Muscles)
1. `core_dispatcher` membaca identitas printer tujuan dari database.
2. Jika tipe = MODERN: ZetaPrint membuka TCP/Raw socket langsung ke IP Printer fisik.
3. Jika tipe = LEGACY: ZetaPrint meneruskan request IPP ke port internal PAPPL yang berjalan di dalam container yang sama.
4. Setelah file terkirim, status DB diupdate menjadi `Completed`, saldo departemen dipotong.

### C. UI Delivery Flow (The Face)
1. User mengakses IP ZetaPrint dari browser (Port 80/8080).
2. Axum mendeteksi request GET ke rute `/`.
3. `rust-embed` memanggil file `index.html` dari dalam binary Rust secara instan.
4. Browser merender UI (Svelte SPA) dan membuka koneksi `EventSource` (SSE) untuk menerima pembaruan status.

## 6. EXECUTION & INFRASTRUCTURE RULES
Karena menggunakan Systemd dan berjalan langsung di Host (Linux), eksekusi HARUS mematuhi parameter berikut:
1. **Network:** Berjalan native di host, mDNS (Bonjour) dari IPP Server akan bekerja secara langsung di jaringan lokal tanpa intervensi NAT.
2. **Hardware Access:** Akses hardware diberikan ke daemon melalui *user* dan *group* Linux (seperti grup `lp` atau `usb`) agar ZetaPrint dan PAPPL memiliki akses langsung ke `/dev/bus/usb` untuk printer legacy.
3. **Data Persistence:** Semua penyimpanan data (SQLite) dan konfigurasi berada langsung di sistem file lokal, umumnya di folder terpusat seperti `/var/lib/zetaprint/`.

## 7. SECURITY & AUTHENTICATION MECHANISM
- **SPA to API Communication:** Menggunakan JSON Web Tokens (JWT). Saat user atau admin login melalui UI statis, Rust memvalidasi kredensial (via LDAP/Database) dan mengembalikan `HttpOnly Secure Cookie` berisi JWT untuk menjaga keamanan sesi dari serangan XSS.
- **Service-to-Service API Key:** Rute khusus (misal: `/api/v1/print/system`) diamankan menggunakan static API Key. Ini memungkinkan sistem internal lain menembak *endpoint* ini secara langsung.

## 8. EXTERNAL SYSTEM INTEGRATION (BMS READY)
Selain mendengarkan protokol cetak IPP (Port 631), `core_api` menyediakan REST API Endpoint bagi sistem eksternal.
- **Workflow Integrasi Otomatis:** Sistem manajemen bisnis (misal: modul *Warehouse/Purchasing*) dapat mengirimkan *HTTP POST* berisi file PDF murni atau instruksi teks mentah ke ZetaPrint.
- ZetaPrint akan menerima instruksi tersebut, mencatatnya ke dalam `core_db` sebagai transaksi sistem, dan langsung mendistribusikannya via `core_dispatcher` ke printer tujuan (misal: printer dotmatrix gudang) tanpa memerlukan intervensi user.

## 9. LOGGING & TELEMETRY STRATEGY (SYSTEMD)
Karena menggunakan `systemd` sebagai process manager, strategi logging diatur sebagai berikut:
- **Rust Engine:** Menggunakan crate `tracing` dan `tracing-subscriber`. Log diatur untuk output ke `stdout` dalam format terstruktur.
- **Journald:** Output standar (*stdout/stderr*) akan ditangkap secara native oleh systemd `journald`. Administrator dapat membaca log menggunakan perintah standar Linux: `journalctl -u zetaprint -f`.

## 10. DATABASE ENTITIES (CORE SCHEMA)
Tabel utama dalam skema PostgreSQL:
1. `departments` (id, code, name, monthly_quota)
2. `users` (id, ad_sid, username, department_id, role)
3. `printers` (id, name, ip_address, is_legacy, cost_per_page_bw, cost_per_page_color)
4. `print_jobs` (id, user_id, printer_id, document_name, total_pages, is_color, total_cost, status, created_at)