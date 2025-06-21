use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=assets/input.css");
    println!("cargo:rerun-if-changed=assets/script.js");
    println!("cargo:rerun-if-changed=assets/lazy-loading.js");
    println!("cargo:rerun-if-changed=assets/statistics.js");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=src/");

    // Check if we have the node_modules and package.json for Tailwind
    if !Path::new("package.json").exists() {
        panic!("package.json not found. Please ensure Tailwind CSS dependencies are configured.");
    }

    // Install dependencies if node_modules doesn't exist
    if !Path::new("node_modules").exists() {
        eprintln!("Installing Tailwind CSS dependencies...");
        let install_output = Command::new("npm")
            .arg("install")
            .output()
            .expect("Failed to run npm install. Make sure Node.js and npm are installed.");
        
        if !install_output.status.success() {
            panic!(
                "npm install failed: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
        }
    }

    // Generate the CSS using Tailwind
    eprintln!("Compiling Tailwind CSS...");
    
    // Create a temporary output file for the CSS
    let output_path = std::env::temp_dir().join("style.css");
    
    let tailwind_output = Command::new("npx")
        .args(&[
            "tailwindcss",
            "-i", "./assets/input.css",
            "-o", &output_path.to_string_lossy(),
            "--minify"
        ])
        .output()
        .expect("Failed to run Tailwind CSS compilation. Make sure Node.js and npm are installed.");
    
    if !tailwind_output.status.success() {
        panic!(
            "Tailwind CSS compilation failed: {}",
            String::from_utf8_lossy(&tailwind_output.stderr)
        );
    }

    // Read the generated CSS and write it to a file that can be included at compile time
    let css_content = fs::read_to_string(&output_path)
        .expect("Failed to read generated CSS file");
    
    // Write the CSS to a file in the target directory for inclusion
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compiled_style.css");
    fs::write(&dest_path, css_content)
        .expect("Failed to write CSS to output directory");
    
    // Clean up the temporary file
    let _ = fs::remove_file(&output_path);
    
    eprintln!("Tailwind CSS compilation completed");
} 