use flate2::Compression;
use flate2::write::GzEncoder;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use super::shared::{newest_mtime, write_if_changed};

#[derive(serde::Serialize)]
struct LicenseGroup {
    license: String,
    text: String,
    dependencies: Vec<LicenseDep>,
}

#[derive(serde::Serialize)]
struct LicenseDep {
    name: String,
    version: String,
}

/// Generate third-party license text for embedding in the binary.
/// Combines Rust dependency licenses (via cargo-about) and frontend npm dependency
/// licenses (via license-checker-rseidelsohn).
pub(crate) fn generate_licenses(out_dir: &str, skip_generation: bool) {
    let output_path = Path::new(out_dir).join("LICENSES.json.gz");

    if skip_generation {
        eprintln!("Skipping license generation (KOSHELF_SKIP_LICENSE_GENERATION=1)");
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(b"[]").unwrap();
        let compressed = encoder.finish().unwrap();
        let _wrote = write_if_changed(&output_path, &compressed)
            .unwrap_or_else(|e| panic!("Failed to write license placeholder: {}", e));
        return;
    }

    // Skip regeneration when the output is already newer than all inputs.
    let input_paths: [&Path; 3] = [
        Path::new("Cargo.lock"),
        Path::new("about.toml"),
        Path::new("frontend/package-lock.json"),
    ];

    if let Ok(output_mtime) = fs::metadata(&output_path).and_then(|m| m.modified())
        && let Some(newest_input) = newest_mtime(&input_paths)
        && output_mtime >= newest_input
    {
        eprintln!("License output is up to date, skipping generation.");
        return;
    }

    let mut groups: Vec<LicenseGroup> = Vec::new();

    // ── Rust dependency licenses (cargo-about) ──────────────────────────
    // Filter by current build target so only actual dependencies are included
    // (e.g. no windows-* crates on macOS builds and vice versa).
    let target = std::env::var("TARGET").expect("TARGET env var not set");

    eprintln!("Generating Rust dependency licenses (target: {target})...");
    let cargo_about = Command::new("cargo")
        .args(["about", "generate", "--format", "json", "--target", &target])
        .output();

    let cargo_about_output = match cargo_about {
        Ok(result) if result.status.success() => result.stdout,
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = &result.stdout;
            if !stdout.is_empty() {
                eprintln!(
                    "cargo-about produced warnings but generated output:\n{}",
                    stderr
                );
                stdout.clone()
            } else {
                panic!("cargo-about failed to generate license output:\n{}", stderr);
            }
        }
        Err(e) => {
            panic!(
                "Failed to run cargo-about. Install it with: cargo install cargo-about\nError: {}",
                e
            );
        }
    };

    #[derive(serde::Deserialize)]
    struct CargoAboutOutput {
        licenses: Vec<CargoAboutLicense>,
    }
    #[derive(serde::Deserialize)]
    struct CargoAboutLicense {
        id: String,
        text: String,
        used_by: Vec<CargoAboutUsedBy>,
    }
    #[derive(serde::Deserialize)]
    struct CargoAboutUsedBy {
        #[serde(rename = "crate")]
        krate: CargoAboutCrate,
    }
    #[derive(serde::Deserialize)]
    struct CargoAboutCrate {
        name: String,
        version: String,
    }

    let parsed: CargoAboutOutput = serde_json::from_slice(&cargo_about_output)
        .expect("Failed to parse cargo-about JSON output");

    groups.extend(parsed.licenses.into_iter().map(|lic| {
        LicenseGroup {
            license: lic.id,
            text: lic.text,
            dependencies: lic
                .used_by
                .into_iter()
                .map(|u| LicenseDep {
                    name: u.krate.name,
                    version: u.krate.version,
                })
                .collect(),
        }
    }));

    // ── Frontend dependency licenses (license-checker-rseidelsohn) ─────
    let frontend_dir = Path::new("frontend");
    if frontend_dir.join("node_modules").exists() {
        eprintln!("Generating frontend dependency licenses...");
        groups.extend(collect_frontend_licenses());
    }

    // Merge groups that share identical (license, text) pairs.
    let mut merged: Vec<LicenseGroup> = Vec::new();
    for group in groups {
        if let Some(existing) = merged
            .iter_mut()
            .find(|g| g.license == group.license && g.text == group.text)
        {
            existing.dependencies.extend(group.dependencies);
        } else {
            merged.push(group);
        }
    }
    for group in &mut merged {
        group
            .dependencies
            .sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    }

    let json = serde_json::to_string(&merged).expect("Failed to serialize license data");

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(json.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    eprintln!(
        "License data: {} bytes JSON -> {} bytes gzip ({:.1}%)",
        json.len(),
        compressed.len(),
        (compressed.len() as f64 / json.len() as f64) * 100.0
    );

    // Always write (not write_if_changed) to update mtime for the cache check.
    fs::write(&output_path, &compressed)
        .unwrap_or_else(|e| panic!("Failed to write compressed license file: {}", e));

    eprintln!("License generation completed: {}", output_path.display());
}

/// Parse the `accepted` list from about.toml (shared with cargo-about).
fn accepted_licenses() -> Vec<String> {
    let content = fs::read_to_string("about.toml").expect("Failed to read about.toml");
    let toml: toml::Value = toml::from_str(&content).expect("Failed to parse about.toml");

    toml["accepted"]
        .as_array()
        .expect("about.toml must contain an 'accepted' array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("Each accepted license must be a string")
                .to_string()
        })
        .collect()
}

fn collect_frontend_licenses() -> Vec<LicenseGroup> {
    let only_allow = accepted_licenses().join(";");

    let result = Command::new("npx")
        .args([
            "license-checker-rseidelsohn",
            "--production",
            "--excludePackages",
            "koshelf-frontend",
            "--onlyAllow",
            &only_allow,
            "--json",
        ])
        .current_dir("frontend")
        .output();

    let result = match result {
        Ok(r) => r,
        Err(e) => panic!(
            "Failed to run license-checker-rseidelsohn. \
             Install it with: npm --prefix frontend install --save-dev license-checker-rseidelsohn\n\
             Error: {}",
            e
        ),
    };

    if !result.status.success() {
        panic!(
            "Frontend license check failed (a dependency may use a disallowed license):\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    let json: serde_json::Value =
        serde_json::from_slice(&result.stdout).expect("Failed to parse license-checker JSON");

    let packages = json
        .as_object()
        .expect("Expected JSON object from license-checker");

    // Group packages by their license text to avoid repetition.
    let mut groups: BTreeMap<String, (String, Option<String>, Vec<LicenseDep>)> = BTreeMap::new();

    for (pkg_id, info) in packages {
        let license_id = info["licenses"].as_str().unwrap_or("Unknown").to_string();

        // Read the actual license file if it exists and is not a README fallback.
        let license_text = info["licenseFile"]
            .as_str()
            .filter(|p| {
                let name = p.rsplit('/').next().unwrap_or("").to_lowercase();
                name.starts_with("licen") || name.starts_with("copying")
            })
            .and_then(|p| fs::read_to_string(p).ok())
            .map(|t| t.trim().to_string());

        // Split "pkg@version" into name and version.
        let (name, version) = match pkg_id.rfind('@') {
            Some(pos) => (&pkg_id[..pos], &pkg_id[pos + 1..]),
            None => (pkg_id.as_str(), ""),
        };

        let key = license_text.clone().unwrap_or_else(|| license_id.clone());
        let entry = groups
            .entry(key)
            .or_insert_with(|| (license_id, license_text, Vec::new()));
        entry.2.push(LicenseDep {
            name: name.to_string(),
            version: version.to_string(),
        });
    }

    groups
        .into_values()
        .map(|(license_id, text, deps)| {
            let text = text.unwrap_or_else(|| {
                format!("(No license file found. SPDX identifier: {})", license_id)
            });
            LicenseGroup {
                license: license_id,
                text,
                dependencies: deps,
            }
        })
        .collect()
}
