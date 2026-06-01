use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::OnceLock;
use std::env;
use std::time::Duration;
use tracing::{info, warn};

pub struct DbState {
    pub pool: SqlitePool,
}

static DB_STATE: OnceLock<DbState> = OnceLock::new();

pub async fn init() -> Result<(), anyhow::Error> {
    info!("Initializing core_db SQLite connection pool...");

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://zetaprint.db".to_string());

    // Setup connection options
    let pool_result = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await;

    match pool_result {
        Ok(pool) => {
            info!("Successfully connected to SQLite at {}", database_url);
            
            // Run database migrations
            info!("Running database migrations...");
            if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
                warn!("Failed to run database migrations: {}", e);
            } else {
                info!("Database migrations applied successfully.");
            }

            let _ = DB_STATE.set(DbState { pool });
        }
        Err(e) => {
            warn!("Could not connect to SQLite ({}): {}. System will run with in-memory fallback.", database_url, e);
        }
    }

    Ok(())
}

pub fn get_pool() -> Option<&'static SqlitePool> {
    DB_STATE.get().map(|state| &state.pool)
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct PrinterRow {
    pub id: String,
    pub name: String,
    pub ip_address: String,
    pub is_legacy: bool,
    pub cost_per_page_bw: f64,
    pub cost_per_page_color: f64,
    pub is_restricted: bool,
}

pub async fn get_printer_by_name(name: &str) -> Result<Option<PrinterRow>, anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    let printer = sqlx::query_as::<_, PrinterRow>("SELECT * FROM printers WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(printer)
}

pub async fn insert_print_job(
    id: &str,
    user_id: &str,
    printer_id: &str,
    document_name: &str,
    total_pages: u32,
    is_color: bool,
    total_cost: f64,
    status: &str,
) -> Result<(), anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    
    sqlx::query(
        r#"
        INSERT INTO print_jobs (id, user_id, printer_id, document_name, total_pages, is_color, total_cost, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(id)
    .bind(user_id)
    .bind(printer_id)
    .bind(document_name)
    .bind(total_pages)
    .bind(is_color)
    .bind(total_cost)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn is_printer_restricted(printer_id: &str) -> Result<bool, anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    let restricted: (bool,) = sqlx::query_as("SELECT is_restricted FROM printers WHERE id = ?")
        .bind(printer_id)
        .fetch_one(pool)
        .await?;
    Ok(restricted.0)
}

pub async fn get_printer_allowed_ous(printer_id: &str) -> Result<Vec<String>, anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    let records: Vec<(String,)> = sqlx::query_as(
        "SELECT entity_name FROM printer_acl WHERE printer_id = ? AND entity_type = 'ou'"
    )
    .bind(printer_id)
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|(name,)| name).collect())
}

pub async fn add_printer_acl(printer_id: &str, entity_type: &str, entity_name: &str) -> Result<(), anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    sqlx::query("INSERT INTO printer_acl (printer_id, entity_type, entity_name) VALUES (?, ?, ?)")
        .bind(printer_id)
        .bind(entity_type)
        .bind(entity_name)
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct DashboardStats {
    pub total_printers: i64,
    pub total_jobs: i64,
    pub total_revenue: f64,
}

pub async fn get_dashboard_stats() -> Result<DashboardStats, anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    
    let printers_row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM printers").fetch_one(pool).await?;
    let jobs_row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM print_jobs").fetch_one(pool).await?;
    let revenue_row: (f64,) = sqlx::query_as("SELECT COALESCE(SUM(total_cost), 0.0) FROM print_jobs").fetch_one(pool).await?;

    Ok(DashboardStats {
        total_printers: printers_row.0,
        total_jobs: jobs_row.0,
        total_revenue: revenue_row.0,
    })
}

pub async fn get_all_printers() -> Result<Vec<PrinterRow>, anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    let printers = sqlx::query_as::<_, PrinterRow>("SELECT * FROM printers ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(printers)
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct PrintJobRow {
    pub id: String,
    pub user_id: String,
    pub printer_id: String,
    pub document_name: String,
    pub total_pages: i64,
    pub is_color: bool,
    pub total_cost: f64,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn get_print_queue() -> Result<Vec<PrintJobRow>, anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    let jobs = sqlx::query_as::<_, PrintJobRow>("SELECT * FROM print_jobs ORDER BY created_at DESC LIMIT 50")
        .fetch_all(pool)
        .await?;
    Ok(jobs)
}

pub async fn create_printer(
    id: &str,
    name: &str,
    ip_address: &str,
    is_legacy: bool,
    cost_per_page_bw: f64,
    cost_per_page_color: f64,
    is_restricted: bool,
) -> Result<(), anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    sqlx::query(
        r#"
        INSERT INTO printers (id, name, ip_address, is_legacy, cost_per_page_bw, cost_per_page_color, is_restricted)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(id)
    .bind(name)
    .bind(ip_address)
    .bind(is_legacy)
    .bind(cost_per_page_bw)
    .bind(cost_per_page_color)
    .bind(is_restricted)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_printer(
    id: &str,
    name: &str,
    ip_address: &str,
    is_legacy: bool,
    cost_per_page_bw: f64,
    cost_per_page_color: f64,
    is_restricted: bool,
) -> Result<(), anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    sqlx::query(
        r#"
        UPDATE printers 
        SET name = ?, ip_address = ?, is_legacy = ?, cost_per_page_bw = ?, cost_per_page_color = ?, is_restricted = ?
        WHERE id = ?
        "#
    )
    .bind(name)
    .bind(ip_address)
    .bind(is_legacy)
    .bind(cost_per_page_bw)
    .bind(cost_per_page_color)
    .bind(is_restricted)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_printer(id: &str) -> Result<(), anyhow::Error> {
    let pool = get_pool().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    sqlx::query("DELETE FROM printers WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
