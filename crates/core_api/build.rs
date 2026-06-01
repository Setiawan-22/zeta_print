use std::process::Command;
use std::path::Path;

fn main() {
    let frontend_dir = Path::new("../../frontend");
    
    // Only run if the frontend directory exists
    if frontend_dir.exists() {
        // Tell Cargo to re-run this script if any of these frontend files change
        println!("cargo:rerun-if-changed=../../frontend/src");
        println!("cargo:rerun-if-changed=../../frontend/static");
        println!("cargo:rerun-if-changed=../../frontend/package.json");
        println!("cargo:rerun-if-changed=../../frontend/svelte.config.js");
        println!("cargo:rerun-if-changed=../../frontend/vite.config.js");

        let npm_cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };

        // 1. Run npm install
        let install_status = Command::new(npm_cmd)
            .current_dir(frontend_dir)
            .args(["install"])
            .status();

        match install_status {
            Ok(status) if status.success() => {
                // 2. Run npm run build
                let build_status = Command::new(npm_cmd)
                    .current_dir(frontend_dir)
                    .args(["run", "build"])
                    .status();

                if let Ok(build_s) = build_status {
                    if !build_s.success() {
                        println!("cargo:warning=npm run build failed. Frontend SPA assets might be missing or broken.");
                    }
                } else {
                    println!("cargo:warning=Failed to execute npm run build.");
                }
            }
            Ok(_) => {
                println!("cargo:warning=npm install failed. Skipping frontend build.");
            }
            Err(e) => {
                println!("cargo:warning=npm command not found ({}). Ensure NodeJS is installed. Skipping frontend build.", e);
            }
        }
    } else {
        println!("cargo:warning=Frontend directory not found at {:?}. Skipping build.", frontend_dir);
    }
}
