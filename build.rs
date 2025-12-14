use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    rerun_if_changed_recursive(Path::new("assets"));
    rerun_if_changed_recursive(Path::new("templates"));
    rerun_if_changed_recursive(Path::new("src"));
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");

    // Check if we have the node_modules and package.json for Tailwind
    if !Path::new("package.json").exists() {
        panic!("package.json not found. Please ensure Tailwind CSS dependencies are configured.");
    }

    // Install dependencies if node_modules doesn't exist or if package-lock.json is newer than node_modules
    let should_install = !Path::new("node_modules").exists()
        || (Path::new("package-lock.json").exists()
            && Path::new("node_modules")
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                < Path::new("package-lock.json")
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH));

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

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Generate the CSS using Tailwind
    eprintln!("Compiling Tailwind CSS...");

    // Create a temporary output file for the CSS
    let output_path = std::env::temp_dir().join("style.css");

    let tailwind_output = Command::new("npx")
        .args([
            "tailwindcss",
            "-i",
            "./assets/input.css",
            "-o",
            &output_path.to_string_lossy(),
            "--minify",
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
    let css_content = fs::read_to_string(&output_path).expect("Failed to read generated CSS file");

    // Write the CSS to a file in the target directory for inclusion
    let dest_path = Path::new(&out_dir).join("compiled_style.css");
    fs::write(&dest_path, css_content).expect("Failed to write CSS to output directory");

    // Clean up the temporary file
    let _ = fs::remove_file(&output_path);

    eprintln!("Tailwind CSS compilation completed");

    // Compile TypeScript with esbuild
    compile_typescript(&out_dir);

    let calendar_css_path =
        Path::new("node_modules/@event-calendar/build/dist/event-calendar.min.css");
    // Copy calendar CSS (JS is bundled into calendar.js via esbuild)
    eprintln!("Copying event calendar CSS...");

    // Ensure calendar CSS file exists
    if !calendar_css_path.exists() {
        panic!(
            "Event calendar CSS file not found at {:?}. Make sure @event-calendar/build is properly installed.",
            calendar_css_path
        );
    }

    let calendar_css_content =
        fs::read_to_string(calendar_css_path).expect("Failed to read event calendar CSS file");
    let calendar_css_dest = Path::new(&out_dir).join("event-calendar.min.css");
    fs::write(&calendar_css_dest, calendar_css_content)
        .expect("Failed to write event calendar CSS to output directory");
    eprintln!("Event calendar CSS copied successfully");

    // Download and embed fonts for SVG rendering
    download_fonts(&out_dir);
}

fn rerun_if_changed_recursive(dir: &Path) {
    if !dir.exists() {
        return;
    }

    // Watch the directory entry too, so add/remove/rename can trigger rebuilds.
    if let Some(p) = dir.to_str() {
        println!("cargo:rerun-if-changed={}", p);
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rerun_if_changed_recursive(&path);
        } else if path.is_file() {
            // Best-effort: only print paths Cargo can parse nicely.
            if let Some(p) = path.to_str() {
                println!("cargo:rerun-if-changed={}", p);
            }
        }
    }
}

/// Compile TypeScript files with esbuild
/// Outputs JavaScript files to the OUT_DIR for embedding via include_str!
fn compile_typescript(out_dir: &str) {
    let ts_dir = Path::new("assets/ts");

    // Skip if no TypeScript directory exists yet (allows gradual migration)
    if !ts_dir.exists() {
        eprintln!("No assets/ts directory found, skipping TypeScript compilation");
        return;
    }

    // Explicit entrypoints: we want a small shared base bundle + a few page bundles.
    // Helper modules are imported by these entrypoints and should not be emitted as standalone files.
    let ts_files: Vec<String> = vec![
        "assets/ts/app/base.ts",
        "assets/ts/pages/library_list.ts",
        "assets/ts/pages/item_detail.ts",
        "assets/ts/pages/statistics.ts",
        "assets/ts/pages/recap.ts",
        "assets/ts/pages/calendar.ts",
        // Service worker must remain its own top-level script.
        "assets/ts/app/service-worker.ts",
    ]
    .into_iter()
    .map(|p| p.to_string())
    .collect();

    // Ensure all entrypoints exist to keep build errors actionable.
    for entry in &ts_files {
        if !Path::new(entry).exists() {
            panic!("TypeScript entrypoint not found: {}", entry);
        }
    }

    if ts_files.is_empty() {
        eprintln!("No TypeScript files found in assets/ts/, skipping compilation");
        return;
    }

    eprintln!("Compiling {} TypeScript files...", ts_files.len());

    let mut args = vec![
        "esbuild".to_string(),
        "--bundle".to_string(),
        "--format=esm".to_string(),
        "--target=es2020".to_string(),
        "--minify".to_string(),
        // Flatten output names so Rust can embed OUT_DIR/<name>.js.
        // Without this, esbuild preserves folders (e.g. pages/calendar.ts -> OUT_DIR/pages/calendar.js),
        // and stale OUT_DIR/calendar.js can remain and accidentally get embedded/served.
        "--entry-names=[name]".to_string(),
        format!("--outdir={}", out_dir),
    ];
    args.extend(ts_files);

    let esbuild_output = Command::new("npx")
        .args(&args)
        .output()
        .expect("Failed to run esbuild. Make sure Node.js and npm are installed.");

    if !esbuild_output.status.success() {
        panic!(
            "TypeScript compilation failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&esbuild_output.stdout),
            String::from_utf8_lossy(&esbuild_output.stderr)
        );
    }

    eprintln!("TypeScript compilation completed successfully");
}

/// Download Gelasio font files for SVG rendering
/// Uses a shared cache directory so fonts are only downloaded once across all targets
fn download_fonts(out_dir: &str) {
    let fonts = [
        (
            "Gelasio-Regular.ttf",
            "https://fonts.gstatic.com/s/gelasio/v14/cIfiMaFfvUQxTTqS3iKJkLGbI41wQL8Ilycs.ttf",
        ),
        (
            "Gelasio-Italic.ttf",
            "https://fonts.gstatic.com/s/gelasio/v14/cIfsMaFfvUQxTTqS9Cu7b2nySBfeR6rA1M9v8zQ.ttf",
        ),
    ];

    // Use a shared cache directory in target/ so fonts are only downloaded once
    // across all target architectures during cross-compilation
    let cache_dir = Path::new("target").join(".font-cache");
    fs::create_dir_all(&cache_dir).expect("Failed to create font cache directory");

    for (filename, url) in fonts {
        let cache_path = cache_dir.join(filename);
        let dest_path = Path::new(out_dir).join(filename);

        // Check if font is already in shared cache
        if cache_path.exists() {
            eprintln!("Font {} found in cache, copying to build dir", filename);
            fs::copy(&cache_path, &dest_path)
                .unwrap_or_else(|e| panic!("Failed to copy cached font {}: {}", filename, e));
            continue;
        }

        eprintln!("Downloading font: {}...", filename);

        match ureq::get(url).call() {
            Ok(response) => {
                let bytes = match response.into_body().read_to_vec() {
                    Ok(b) => b,
                    Err(e) => panic!("Failed to read font data for {}: {}", filename, e),
                };

                // Save to shared cache
                fs::write(&cache_path, &bytes)
                    .unwrap_or_else(|e| panic!("Failed to cache font {}: {}", filename, e));

                // Copy to build output directory
                fs::write(&dest_path, &bytes)
                    .unwrap_or_else(|e| panic!("Failed to write font {}: {}", filename, e));

                eprintln!(
                    "Font {} downloaded and cached ({} bytes)",
                    filename,
                    bytes.len()
                );
            }
            Err(e) => {
                panic!("Failed to download font {}: {}", filename, e);
            }
        }
    }

    eprintln!("Font files downloaded/verified successfully");
}
