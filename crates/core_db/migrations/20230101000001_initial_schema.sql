-- Departments Table
CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    monthly_quota REAL DEFAULT 0.0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Users Table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    ad_sid TEXT UNIQUE,
    username TEXT UNIQUE NOT NULL,
    department_id TEXT REFERENCES departments(id) ON DELETE SET NULL,
    role TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Printers Table
CREATE TABLE IF NOT EXISTS printers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    is_legacy BOOLEAN NOT NULL DEFAULT 0,
    cost_per_page_bw REAL NOT NULL DEFAULT 0.0,
    cost_per_page_color REAL NOT NULL DEFAULT 0.0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Print Jobs Table
CREATE TABLE IF NOT EXISTS print_jobs (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    printer_id TEXT NOT NULL REFERENCES printers(id),
    document_name TEXT NOT NULL,
    total_pages INTEGER NOT NULL DEFAULT 1,
    is_color BOOLEAN NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
