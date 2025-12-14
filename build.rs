use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn main() {
    rerun_if_changed_recursive(Path::new("assets"));
    rerun_if_changed_recursive(Path::new("templates"));
    rerun_if_changed_recursive(Path::new("src"));
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");
    println!("cargo:rerun-if-env-changed=KOSHELF_SKIP_NODE_BUILD");
    println!("cargo:rerun-if-env-changed=KOSHELF_SKIP_NPM_INSTALL");
    println!("cargo:rerun-if-env-changed=KOSHELF_SKIP_FONT_DOWNLOAD");
    println!("cargo:rerun-if-env-changed=KOSHELF_FONT_CACHE_DIR");

    let skip_node_build = env_flag("KOSHELF_SKIP_NODE_BUILD");
    let skip_npm_install = env_flag("KOSHELF_SKIP_NPM_INSTALL");
    let skip_font_download = env_flag("KOSHELF_SKIP_FONT_DOWNLOAD");

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

    if should_install && !skip_npm_install && !skip_node_build {
        eprintln!("Installing npm dependencies...");
        let mut cmd = Command::new("npm");
        if Path::new("package-lock.json").exists() {
            // Deterministic install based on lockfile (better for CI / reproducibility).
            cmd.arg("ci");
        } else {
            cmd.arg("install");
        }
        let install_output = cmd
            .output()
            .expect("Failed to run npm install. Make sure Node.js and npm are installed.");

        if !install_output.status.success() {
            panic!(
                "npm install failed: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
        }
        eprintln!("npm install completed successfully");
    } else if should_install && (skip_npm_install || skip_node_build) {
        panic!(
            "node_modules missing/outdated but npm install is disabled (KOSHELF_SKIP_NODE_BUILD or KOSHELF_SKIP_NPM_INSTALL). \
             Run `npm ci`/`npm install` manually, or unset the env var(s)."
        );
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();

    if !skip_node_build {
        // Compile CSS bundles
        compile_tailwind_css(
            &out_dir,
            "Tailwind",
            Path::new("assets/css/input.css"),
            "compiled_style.css",
        );

        let compiled_calendar = compile_tailwind_css(
            &out_dir,
            "calendar Tailwind",
            Path::new("assets/css/calendar.css"),
            "compiled_calendar.css",
        );
        bundle_css_with_esbuild("calendar", &compiled_calendar, &out_dir);

        // Compile TypeScript with esbuild
        compile_typescript(&out_dir);
    } else {
        eprintln!("Skipping Tailwind/CSS/TypeScript build (KOSHELF_SKIP_NODE_BUILD=1)");
    }

    // Download and embed fonts for SVG rendering
    download_fonts(&out_dir, skip_font_download);
}

fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES") | Ok("on") | Ok("ON")
    )
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

fn write_if_changed(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    match fs::read(path) {
        Ok(existing) if existing == bytes => Ok(false),
        _ => {
            fs::write(path, bytes)?;
            Ok(true)
        }
    }
}

/// Compile a Tailwind CSS entrypoint into OUT_DIR.
/// Returns the output file path in OUT_DIR.
fn compile_tailwind_css(
    out_dir: &str,
    display_name: &str,
    input: &Path,
    out_filename: &str,
) -> std::path::PathBuf {
    if !input.exists() {
        panic!("{} not found (required for {} styling)", input.display(), display_name);
    }

    eprintln!("Compiling {} CSS...", display_name);
    // Use OUT_DIR for intermediates to avoid collisions across parallel builds.
    let tmp_path = Path::new(out_dir).join(format!("{}.tmp", out_filename));
    let dest_path = Path::new(out_dir).join(out_filename);

    let output = Command::new("npx")
        .args([
            "tailwindcss",
            "-i",
            &input.to_string_lossy(),
            "-o",
            &tmp_path.to_string_lossy(),
            "--minify",
        ])
        .output()
        .expect("Failed to run Tailwind CSS. Make sure Node.js and npm are installed.");

    if !output.status.success() {
        panic!(
            "{} CSS compilation failed:\nstderr: {}",
            display_name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let css_bytes = fs::read(&tmp_path).expect("Failed to read generated CSS");
    let _wrote = write_if_changed(&dest_path, &css_bytes)
        .expect("Failed to write generated CSS to output directory");
    let _ = fs::remove_file(&tmp_path);

    eprintln!("{} CSS compilation completed", display_name);
    dest_path
}

/// Bundle and minify a CSS file using esbuild
/// - `name`: Display name for logging (e.g., "calendar")
/// - `input_path`: Path to the source CSS file
/// - `out_dir`: Output directory for the bundled file
fn bundle_css_with_esbuild(name: &str, input_path: &Path, out_dir: &str) {
    if !input_path.exists() {
        panic!(
            "{} not found (required for {} styling)",
            input_path.display(),
            name
        );
    }

    eprintln!("Bundling {} CSS...", name);
    let output_name = format!("{}.css", name);
    let outfile = Path::new(out_dir).join(&output_name);
    let tmpfile = Path::new(out_dir).join(format!("{}.css.tmp", name));

    let output = Command::new("npx")
        .args([
            "esbuild",
            &input_path.to_string_lossy(),
            "--bundle",
            "--minify",
            "--loader:.css=css",
            &format!("--outfile={}", tmpfile.to_string_lossy()),
        ])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to run esbuild for {} CSS. Make sure Node.js and npm are installed. Error: {}",
                name, e
            )
        });

    if !output.status.success() {
        panic!(
            "{} CSS bundling failed:\nstdout: {}\nstderr: {}",
            name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let css_bytes = fs::read(&tmpfile).expect("Failed to read bundled CSS");
    let _wrote = write_if_changed(&outfile, &css_bytes).expect("Failed to write bundled CSS");
    let _ = fs::remove_file(&tmpfile);

    eprintln!("{} CSS bundling completed", name);
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
fn download_fonts(out_dir: &str, skip_download: bool) {
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
    let cache_dir = std::env::var("KOSHELF_FONT_CACHE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| Path::new("target").join(".font-cache"));
    fs::create_dir_all(&cache_dir).expect("Failed to create font cache directory");

    for (filename, url) in fonts {
        let cache_path = cache_dir.join(filename);
        let dest_path = Path::new(out_dir).join(filename);

        // Check if font is already in shared cache
        if cache_path.exists() {
            eprintln!("Font {} found in cache, copying to build dir", filename);
            let bytes = fs::read(&cache_path)
                .unwrap_or_else(|e| panic!("Failed to read cached font {}: {}", filename, e));
            let _wrote = write_if_changed(&dest_path, &bytes)
                .unwrap_or_else(|e| panic!("Failed to write font {}: {}", filename, e));
            continue;
        }

        if skip_download {
            panic!(
                "Font {} not found in cache and downloading is disabled (KOSHELF_SKIP_FONT_DOWNLOAD=1). \
                 Either unset that env var, or pre-populate the cache at {:?}.",
                filename, cache_dir
            );
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

                // Copy to build output directory (but avoid rewriting identical output).
                let _wrote = write_if_changed(&dest_path, &bytes)
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
