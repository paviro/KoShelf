use std::process::Command;
use std::path::Path;
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=assets/input.css");
    println!("cargo:rerun-if-changed=assets/book_list.js");
    println!("cargo:rerun-if-changed=assets/lazy-loading.js");
    println!("cargo:rerun-if-changed=assets/statistics.js");
    println!("cargo:rerun-if-changed=assets/heatmap.js");
    println!("cargo:rerun-if-changed=assets/section-toggle.js");
    println!("cargo:rerun-if-changed=assets/recap.js");
    println!("cargo:rerun-if-changed=assets/share_story.svg");
    println!("cargo:rerun-if-changed=assets/share_square.svg");
    println!("cargo:rerun-if-changed=assets/share_banner.svg");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");
    println!("cargo:rerun-if-changed=assets/calendar.js");

    // Check if we have the node_modules and package.json for Tailwind
    if !Path::new("package.json").exists() {
        panic!("package.json not found. Please ensure Tailwind CSS dependencies are configured.");
    }

    // Install dependencies if node_modules doesn't exist or if package-lock.json is newer than node_modules
    let should_install = !Path::new("node_modules").exists() || 
        (Path::new("package-lock.json").exists() && 
         Path::new("node_modules").metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH) <
         Path::new("package-lock.json").metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));
    
    if should_install {
        eprintln!("Installing npm dependencies...");
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
        eprintln!("npm install completed successfully");
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

    // Copy calendar library files
    eprintln!("Copying event calendar library files...");
    
    let calendar_js_path = Path::new("node_modules/@event-calendar/build/dist/event-calendar.min.js");
    let calendar_css_path = Path::new("node_modules/@event-calendar/build/dist/event-calendar.min.css");
    let calendar_map_path = Path::new("node_modules/@event-calendar/build/dist/event-calendar.min.js.map");
    
    // Ensure calendar JS file exists
    if !calendar_js_path.exists() {
        panic!("Event calendar JS file not found at {:?}. Make sure @event-calendar/build is properly installed.", calendar_js_path);
    }
    
    let calendar_js_content = fs::read_to_string(calendar_js_path)
        .expect("Failed to read event calendar JS file");
    let calendar_js_dest = Path::new(&out_dir).join("event-calendar.min.js");
    fs::write(&calendar_js_dest, calendar_js_content)
        .expect("Failed to write event calendar JS to output directory");
    
    // Ensure calendar CSS file exists  
    if !calendar_css_path.exists() {
        panic!("Event calendar CSS file not found at {:?}. Make sure @event-calendar/build is properly installed.", calendar_css_path);
    }
    
    let calendar_css_content = fs::read_to_string(calendar_css_path)
        .expect("Failed to read event calendar CSS file");
    let calendar_css_dest = Path::new(&out_dir).join("event-calendar.min.css");
    fs::write(&calendar_css_dest, calendar_css_content)
        .expect("Failed to write event calendar CSS to output directory");
    
    // Copy calendar JS map file if it exists
    if calendar_map_path.exists() {
        let calendar_map_content = fs::read_to_string(calendar_map_path)
            .expect("Failed to read event calendar map file");
        let calendar_map_dest = Path::new(&out_dir).join("event-calendar.min.js.map");
        fs::write(&calendar_map_dest, calendar_map_content)
            .expect("Failed to write event calendar map to output directory");
    }
    
    eprintln!("Event calendar library files copied successfully");
} 