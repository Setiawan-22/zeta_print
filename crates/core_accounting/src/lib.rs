use std::path::Path;
use tracing::{info, warn};

pub async fn init() -> Result<(), anyhow::Error> {
    info!("Initializing core_accounting module...");
    Ok(())
}

#[derive(Debug)]
pub struct PrintJobCost {
    pub total_pages: u32,
    pub is_color: bool,
    pub calculated_cost: f64,
}

/// Analyze a PDF file to determine page count, color usage, and cost
pub async fn analyze_pdf_spool<P: AsRef<Path>>(
    pdf_path: P,
    cost_bw: f64,
    cost_color: f64,
) -> Result<PrintJobCost, anyhow::Error> {
    let path = pdf_path.as_ref();

    // Perform file parsing in spawn_blocking to avoid blocking the async executor (per KI instructions)
    let path_clone = path.to_path_buf();
    let result = tokio::task::spawn_blocking(move || -> Result<PrintJobCost, anyhow::Error> {
        let doc = lopdf::Document::load(path_clone)?;
        
        let total_pages = doc.get_pages().len() as u32;
        let mut is_color = false;

        // Simple color detection: look for device color spaces in page resources
        for page_id in doc.page_iter() {
            let (resources_opt, _) = doc.get_page_resources(page_id);
            if let Some(resources) = resources_opt {
                if let Ok(color_space) = resources.get(b"ColorSpace") {
                    let cs_str = format!("{:?}", color_space);
                    if cs_str.contains("DeviceCMYK") || cs_str.contains("DeviceRGB") {
                        is_color = true;
                        break;
                    }
                }
            }
        }

        let calculated_cost = if is_color {
            total_pages as f64 * cost_color
        } else {
            total_pages as f64 * cost_bw
        };

        Ok(PrintJobCost {
            total_pages,
            is_color,
            calculated_cost,
        })
    })
    .await?;

    match &result {
        Ok(analysis) => {
            info!(
                "PDF Analysis completed for {:?}: {} pages, Color: {}, Total Cost: {}",
                path, analysis.total_pages, analysis.is_color, analysis.calculated_cost
            );
        }
        Err(e) => {
            warn!("Failed to analyze PDF spool: {:?}", e);
        }
    }

    result
}
