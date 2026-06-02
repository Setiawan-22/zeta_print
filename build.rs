use std::process::Command;
use std::path::Path;

fn main() {
    // Only trigger rebuild if frontend source files change
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/svelte.config.js");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");

    let frontend_dir = Path::new("frontend");
    
    // Check if npm is installed (Optional but good for error messages)
    if Command::new("npm").arg("--version").output().is_err() {
        println!("cargo:warning=npm is not installed. Skipping frontend build. Make sure frontend/build exists.");
        return;
    }

    // Run npm install
    let npm_install = Command::new("npm")
        .args(["install"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm install");

    if !npm_install.success() {
        panic!("npm install failed. Make sure you have an active internet connection and Node.js is properly installed.");
    }

    // Run npm run build
    let npm_build = Command::new("npm")
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm run build");

    if !npm_build.success() {
        panic!("npm run build failed. Check your SvelteKit configuration.");
    }
}
