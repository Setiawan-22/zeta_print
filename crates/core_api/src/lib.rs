use axum::{
    Router,
    routing::{get, post, put, delete},
    response::{IntoResponse, Response, sse::{Event, Sse, KeepAlive}},
    http::{StatusCode, Uri, header, Request},
    Json,
    middleware::{self, Next},
    extract::{Extension, State},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::fs;
use uuid::Uuid;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tracing::{info, warn};
use tower_http::cors::CorsLayer;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use futures_util::StreamExt;
use std::convert::Infallible;
use std::time::Duration;

#[derive(Clone, Serialize, Debug)]
pub struct NotificationEvent {
    pub event_type: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<NotificationEvent>,
}

#[derive(RustEmbed)]
#[folder = "../../frontend/build/"] // SPA build static output path
struct Assets;

pub async fn start_web_server() -> Result<(), anyhow::Error> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    info!("Initializing core_api Web Server on {}...", addr);

    // Setup channel (100 capacity)
    let (tx, _) = broadcast::channel(100);
    let app_state = AppState { tx: tx.clone() };

    // Setup router
    let app = Router::new()
        // API v1 Namespace
        .route("/api/v1/auth/login", post(handle_login))
        .route("/api/v1/print/system", post(handle_system_print))
        // Protected APIs
        .nest("/api/v1/dashboard", Router::new()
            .route("/stats", get(handle_dashboard_stats))
            .route("/printers", get(handle_get_printers).post(handle_create_printer))
            .route("/printers/:id", put(handle_update_printer).delete(handle_delete_printer))
            .route("/queue", get(handle_get_queue))
            .route("/ad/ous", get(handle_ad_ous))
            .route("/printers/:id/acl", get(handle_get_printer_acl).post(handle_add_printer_acl))
            .route("/events", get(handle_sse))
            .route_layer(middleware::from_fn(auth_middleware))
        )
        .route("/health", get(handle_health))
        // Fallback: serves embedded Svelte SPA UI
        .fallback(static_handler)
        // CORS & Middlewares
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    // Spawn a dummy task that emits events every 10 seconds for testing
    let tx_clone = app_state.tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let event = NotificationEvent {
                event_type: "ping".to_string(),
                message: "Heartbeat from ZetaPrint".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = tx_clone.send(event);
        }
    });

    // Bind and start listening
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Axum Web server listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

// --- Handler Fallback to serve static SvelteKit files embedded ---
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // If the path is empty (root '/'), serve index.html
    let asset_path = if path.is_empty() {
        "index.html"
    } else {
        path
    };

    match Assets::get(asset_path) {
        Some(file) => {
            let mime_type = mime_guess::from_path(asset_path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .body(axum::body::Body::from(file.data))
                .unwrap()
        }
        None => {
            // For Single Page Application (SPA) routing, fallback to index.html for unknown routes
            match Assets::get("index.html") {
                Some(index_file) => Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(axum::body::Body::from(index_file.data))
                    .unwrap(),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(axum::body::Body::from("404 Not Found"))
                    .unwrap(),
            }
        }
    }
}

// --- API Request / Response structures ---
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    user: UserProfile,
    permissions: Vec<String>,
}

#[derive(Serialize)]
struct UserProfile {
    id: String,
    username: String,
    full_name: String,
    role_code: String,
    role_name: String,
}

// --- Dummy Handlers ---
async fn handle_login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    info!("Login request received for user: {}", payload.username);

    // Exact replica of BMS admin/admin logic
    if payload.username == "admin" && payload.password == "admin" {
        // Generate real JWT
        let token = match core_auth::create_jwt("admin-id", "admin", "System Administrator") {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to generate JWT: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to generate token" }))).into_response();
            }
        };

        let response = LoginResponse {
            user: UserProfile {
                id: "admin-id".to_string(),
                username: "admin".to_string(),
                full_name: "System Administrator".to_string(),
                role_code: "admin".to_string(),
                role_name: "System Administrator".to_string(),
            },
            permissions: vec!["*".to_string()], // Super Admin wildcard bypass
        };
        
        let cookie_str = format!("auth_token={}; HttpOnly; Path=/; SameSite=Lax", token);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::SET_COOKIE,
            cookie_str.parse().unwrap(),
        );

        (StatusCode::OK, headers, Json(response)).into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid username or password" })),
        )
            .into_response()
    }
}

// --- JWT Auth Middleware ---
async fn auth_middleware(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut token_opt = req.headers().get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    // Fallback to cookie if Authorization header is missing
    if token_opt.is_none() {
        if let Some(cookie_header) = req.headers().get(header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(t) = cookie.strip_prefix("auth_token=") {
                        token_opt = Some(t);
                        break;
                    }
                }
            }
        }
    }

    let token = match token_opt {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match core_auth::validate_token(token).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn handle_dashboard_stats(Extension(user): Extension<core_auth::AuthUser>) -> impl IntoResponse {
    info!("Stats requested by user: {}", user.username);
    match core_db::get_dashboard_stats().await {
        Ok(stats) => Json(serde_json::json!({ "status": "success", "data": stats })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

async fn handle_get_printers(Extension(user): Extension<core_auth::AuthUser>) -> impl IntoResponse {
    info!("Printers list requested by user: {}", user.username);
    match core_db::get_all_printers().await {
        Ok(printers) => Json(serde_json::json!({ "status": "success", "data": printers })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
struct PrinterRequest {
    name: String,
    ip_address: String,
    is_legacy: bool,
    cost_per_page_bw: f64,
    cost_per_page_color: f64,
    is_restricted: bool,
}

async fn handle_create_printer(
    Extension(user): Extension<core_auth::AuthUser>,
    Json(payload): Json<PrinterRequest>,
) -> impl IntoResponse {
    info!("Create printer requested by user: {}", user.username);
    let new_id = Uuid::new_v4().to_string();
    
    match core_db::create_printer(
        &new_id,
        &payload.name,
        &payload.ip_address,
        payload.is_legacy,
        payload.cost_per_page_bw,
        payload.cost_per_page_color,
        payload.is_restricted,
    ).await {
        Ok(_) => Json(serde_json::json!({ "status": "success", "id": new_id })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

async fn handle_update_printer(
    axum::extract::Path(printer_id): axum::extract::Path<String>,
    Extension(user): Extension<core_auth::AuthUser>,
    Json(payload): Json<PrinterRequest>,
) -> impl IntoResponse {
    info!("Update printer requested by user: {}", user.username);
    
    match core_db::update_printer(
        &printer_id,
        &payload.name,
        &payload.ip_address,
        payload.is_legacy,
        payload.cost_per_page_bw,
        payload.cost_per_page_color,
        payload.is_restricted,
    ).await {
        Ok(_) => Json(serde_json::json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

async fn handle_delete_printer(
    axum::extract::Path(printer_id): axum::extract::Path<String>,
    Extension(user): Extension<core_auth::AuthUser>,
) -> impl IntoResponse {
    info!("Delete printer requested by user: {}", user.username);
    match core_db::delete_printer(&printer_id).await {
        Ok(_) => Json(serde_json::json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

async fn handle_get_queue(Extension(user): Extension<core_auth::AuthUser>) -> impl IntoResponse {
    info!("Print queue requested by user: {}", user.username);
    match core_db::get_print_queue().await {
        Ok(queue) => Json(serde_json::json!({ "status": "success", "data": queue })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

async fn handle_ad_ous() -> impl IntoResponse {
    let client = match core_identity::LdapClient::new() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to init LDAP client").into_response(),
    };
    match client.fetch_ous().await {
        Ok(ous) => Json(serde_json::json!({ "status": "success", "data": ous })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response()
    }
}

async fn handle_get_printer_acl(axum::extract::Path(printer_id): axum::extract::Path<String>) -> impl IntoResponse {
    match core_db::get_printer_allowed_ous(&printer_id).await {
        Ok(ous) => Json(serde_json::json!({ "status": "success", "data": ous })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response()
    }
}

#[derive(Deserialize)]
struct AddAclRequest {
    entity_type: String,
    entity_name: String,
}

async fn handle_add_printer_acl(
    axum::extract::Path(printer_id): axum::extract::Path<String>,
    Json(payload): Json<AddAclRequest>
) -> impl IntoResponse {
    match core_db::add_printer_acl(&printer_id, &payload.entity_type, &payload.entity_name).await {
        Ok(_) => Json(serde_json::json!({ "status": "success" })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "status": "error", "message": e.to_string() }))).into_response()
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SystemPrintRequest {
    printer_id: String,
    document_name: String,
    payload_base64: String,
}

async fn handle_system_print(Json(payload): Json<SystemPrintRequest>) -> impl IntoResponse {
    info!("BMS System Print request received for printer: {}", payload.printer_id);
    
    // 1. Decode base64 payload
    let pdf_bytes = match STANDARD.decode(&payload.payload_base64) {
        Ok(b) => b,
        Err(e) => {
            warn!("Failed to decode base64 payload: {}", e);
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid base64 payload" }))).into_response();
        }
    };

    let job_id = Uuid::new_v4().to_string();
    let temp_file_path = format!("/tmp/zetaprint_job_{}.pdf", job_id);

    // Save to temp file
    if let Err(e) = fs::write(&temp_file_path, &pdf_bytes) {
        warn!("Failed to write spool file: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Failed to spool file" }))).into_response();
    }

    // 2. Lookup printer in core_db
    let printer = match core_db::get_printer_by_name(&payload.printer_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            warn!("Printer not found: {}", payload.printer_id);
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Printer not found" }))).into_response();
        }
        Err(e) => {
            warn!("DB error looking up printer: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Database error" }))).into_response();
        }
    };

    // 3. Analyze PDF (core_accounting)
    let analysis = match core_accounting::analyze_pdf_spool(&temp_file_path, printer.cost_per_page_bw, printer.cost_per_page_color).await {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to analyze PDF for job {}: {}", job_id, e);
            // Default to 1 page BW if analysis fails
            core_accounting::PrintJobCost { total_pages: 1, is_color: false, calculated_cost: printer.cost_per_page_bw }
        }
    };

    // 4. Save job to DB (assume user is 'system' for now)
    let user_id = "system"; // Mock system user
    if let Err(e) = core_db::insert_print_job(
        &job_id,
        user_id,
        &printer.id,
        &payload.document_name,
        analysis.total_pages,
        analysis.is_color,
        analysis.calculated_cost,
        "Processing",
    ).await {
        warn!("Failed to insert print job to DB: {}", e);
    }

    // 5. Dispatch job (core_dispatcher)
    let target = core_dispatcher::PrinterTarget {
        name: printer.name.clone(),
        ip_address: printer.ip_address.clone(),
        is_legacy: printer.is_legacy,
    };

    let dispatch_job_id = job_id.clone();
    tokio::spawn(async move {
        if let Err(e) = core_dispatcher::dispatch_print_job(&target, &pdf_bytes).await {
            warn!("Job {} failed to dispatch: {}", dispatch_job_id, e);
            // Ideally update job status to Failed in DB here
        } else {
            info!("Job {} dispatched successfully", dispatch_job_id);
            // Update job status to Completed in DB here
        }
        // Cleanup temp file
        let _ = fs::remove_file(&temp_file_path);
    });
    
    (StatusCode::OK, Json(serde_json::json!({ "status": "processing", "job_id": job_id, "message": "Job sent to dispatcher" }))).into_response()
}

async fn handle_health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "online" })))
}

async fn handle_sse(
    State(state): State<AppState>,
    Extension(user): Extension<core_auth::AuthUser>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    info!("Client connected to SSE: {}", user.username);
    
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| async move {
        match res {
            Ok(event) => {
                let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                Some(Ok(Event::default().data(json)))
            }
            Err(_) => {
                // Ignore Lagged errors (dropped messages)
                None
            }
        }
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive-text"),
    )
}
