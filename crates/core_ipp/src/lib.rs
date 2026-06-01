use axum::{
    Router,
    routing::post,
    body::Bytes,
    response::{Response, IntoResponse},
    http::{StatusCode, header},
};
use ipp::prelude::*;
use ipp::parser::IppParser;
use ipp::reader::IppReader;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;
use std::fs;
use std::io::Read;
use core_identity::LdapClient;

#[derive(Clone)]
struct AppState {
    ldap_client: Option<Arc<LdapClient>>,
}

pub async fn start_server() -> Result<(), anyhow::Error> {
    let port = std::env::var("IPP_PORT").unwrap_or_else(|_| "631".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    info!("Starting core_ipp IPP Server on {}...", addr);

    let ldap_client = LdapClient::new().map(Arc::new).ok();
    if ldap_client.is_none() {
        warn!("LDAP client failed to initialize, RBAC AD sync will be bypassed.");
    }

    let state = Arc::new(AppState { ldap_client });

    let app = Router::new()
        .route("/", post(handle_ipp_request))
        .route("/printers/:printer_name", post(handle_ipp_request_named))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            let fallback_addr: SocketAddr = "0.0.0.0:6310".parse()?;
            warn!("Failed to bind to IPP port {}: {}. Attempting fallback port: {}", port, e, fallback_addr);
            tokio::net::TcpListener::bind(fallback_addr).await?
        }
    };

    info!("IPP server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_ipp_request_named(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(printer_name): axum::extract::Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    process_ipp(state, Some(printer_name), body).await
}

async fn handle_ipp_request(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body: Bytes
) -> impl IntoResponse {
    process_ipp(state, None, body).await
}

async fn process_ipp(state: Arc<AppState>, printer_name: Option<String>, body: Bytes) -> Response {
    // 1. Parse IPP Request
    let reader = std::io::Cursor::new(body.to_vec());
    let parser = IppParser::new(IppReader::new(reader));
    let mut request = match parser.parse() {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to parse IPP request: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid IPP Payload").into_response();
        }
    };

    info!("Received IPP Operation: {:?}", request.header().operation_or_status);

    // Extract printer name from URI if not provided in path
    let target_printer = printer_name.unwrap_or_else(|| "default_printer".to_string());
    info!("Target printer: {}", target_printer);

    // Extract requesting user name
    let mut requesting_user = "anonymous".to_string();
    if let Some(op_attrs) = request.attributes().groups_of(ipp::model::DelimiterTag::OperationAttributes).next() {
        if let Some(attr) = op_attrs.attributes().get("requesting-user-name") {
            requesting_user = attr.value().to_string();
        }
    }
    info!("Requesting User: {}", requesting_user);

    // Extract document payload
    let mut document_bytes = Vec::new();
    let _ = request.payload_mut().read_to_end(&mut document_bytes);
    
    if document_bytes.is_empty() {
        warn!("Received empty print payload");
        let ipp_response = IppRequestResponse::new_response(
            request.header().version,
            ipp::model::StatusCode::ClientErrorDocumentFormatNotSupported,
            request.header().request_id,
        ).unwrap();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/ipp")
            .body(axum::body::Body::from(ipp_response.to_bytes().to_vec()))
            .unwrap();
    }

    let job_id = Uuid::new_v4().to_string();
    let temp_file_path = format!("/tmp/zetaprint_{}.pdf", job_id);
    let _ = fs::write(&temp_file_path, &document_bytes);

    // Lookup printer
    let printer = match core_db::get_printer_by_name(&target_printer).await {
        Ok(Some(p)) => p,
        _ => {
            warn!("Printer not found or DB error: {}", target_printer);
            let _ = fs::remove_file(&temp_file_path);
            return (StatusCode::NOT_FOUND, "Printer Not Found").into_response();
        }
    };

    // RBAC & AD Check
    if let Ok(true) = core_db::is_printer_restricted(&printer.id).await {
        info!("Printer {} is restricted. Checking ACL for user '{}'", target_printer, requesting_user);
        let allowed_ous = core_db::get_printer_allowed_ous(&printer.id).await.unwrap_or_default();
        let mut authorized = false;

        if let Some(ldap) = &state.ldap_client {
            for ou in allowed_ous {
                if let Ok(true) = ldap.check_user_in_ou(&requesting_user, &ou).await {
                    authorized = true;
                    break;
                }
            }
        } else {
            // If LDAP is not configured, we just check if the username exactly matches the ACL (basic mode)
            authorized = allowed_ous.contains(&requesting_user);
        }

        if !authorized {
            warn!("User {} is NOT AUTHORIZED to print on {}", requesting_user, target_printer);
            let _ = fs::remove_file(&temp_file_path);
            // IPP Client Error Not Authorized (0x0403)
            let ipp_response = match IppRequestResponse::new_response(
                request.header().version,
                ipp::model::StatusCode::ClientErrorNotAuthorized,
                request.header().request_id,
            ) {
                Ok(r) => r,
                Err(_) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            };
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/ipp")
                .body(axum::body::Body::from(ipp_response.to_bytes().to_vec()))
                .unwrap();
        }
    }

    // Analyze PDF
    let analysis = match core_accounting::analyze_pdf_spool(&temp_file_path, printer.cost_per_page_bw, printer.cost_per_page_color).await {
        Ok(a) => a,
        Err(_) => {
            // Fallback
            core_accounting::PrintJobCost { total_pages: 1, is_color: false, calculated_cost: printer.cost_per_page_bw }
        }
    };

    // Save job
    let user_id = requesting_user.clone();
    if let Err(e) = core_db::insert_print_job(
        &job_id,
        &user_id,
        &printer.id,
        "IPP Print Job",
        analysis.total_pages,
        analysis.is_color,
        analysis.calculated_cost,
        "Processing",
    ).await {
        warn!("Failed to insert IPP print job to DB: {}", e);
    }

    // Dispatch
    let target = core_dispatcher::PrinterTarget {
        name: printer.name.clone(),
        ip_address: printer.ip_address.clone(),
        is_legacy: printer.is_legacy,
    };
    
    let doc_bytes = document_bytes.to_vec();
    tokio::spawn(async move {
        if let Err(e) = core_dispatcher::dispatch_print_job(&target, &doc_bytes).await {
            warn!("IPP Job {} failed to dispatch: {}", job_id, e);
        } else {
            info!("IPP Job {} dispatched successfully", job_id);
        }
        let _ = fs::remove_file(&temp_file_path);
    });

    // Return IPP Success Response
    let ipp_response = match IppRequestResponse::new_response(
        request.header().version,
        ipp::model::StatusCode::SuccessfulOk,
        request.header().request_id,
    ) {
        Ok(resp) => resp,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Serialization Error").into_response();
        }
    };
    
    let out_buffer = ipp_response.to_bytes().to_vec();
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/ipp")
        .body(axum::body::Body::from(out_buffer))
        .unwrap()
}
