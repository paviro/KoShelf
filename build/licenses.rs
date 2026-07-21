use flate2::Compression;
use flate2::write::GzEncoder;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use super::shared::{newest_mtime, write_if_changed};

const FRONTEND_LICENSE_SNAPSHOT: &str = "frontend/build-assets/npm-licenses.json";

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

/// Generate third-party license text for embedding in the binary.
/// Combines Rust dependency licenses (via cargo-about) and frontend npm dependency
/// licenses (via the portable frontend artifact or license-checker-rseidelsohn).
pub(crate) fn generate_licenses(out_dir: &str, skip_generation: bool) {
    let output_path = Path::new(out_dir).join("LICENSES.json.gz");

    if skip_generation {
        eprintln!("Skipping license generation (KOSHELF_SKIP_LICENSE_GENERATION=1)");
        write_gzipped(&output_path, b"[]");
        return;
    }

    let snapshot_path = Path::new(FRONTEND_LICENSE_SNAPSHOT);
    let mut input_paths: Vec<&Path> = vec![
        Path::new("Cargo.lock"),
        Path::new("about.toml"),
        Path::new("frontend/package-lock.json"),
    ];
    if snapshot_path.is_file() {
        input_paths.push(snapshot_path);
    }

    if let Ok(output_mtime) = fs::metadata(&output_path).and_then(|m| m.modified())
        && let Some(newest_input) = newest_mtime(&input_paths)
        && output_mtime >= newest_input
    {
        eprintln!("License output is up to date, skipping generation.");
        return;
    }

    let release = std::env::var("PROFILE").as_deref() == Ok("release");
    let mut groups = collect_rust_licenses(out_dir, release);
    groups.extend(collect_frontend_licenses(release));

    let mut merged: Vec<LicenseGroup> = Vec::new();
    for group in groups {
        if let Some(existing) = merged
            .iter_mut()
            .find(|candidate| candidate.license == group.license && candidate.text == group.text)
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

    let json = serde_json::to_vec(&merged).expect("Failed to serialize license data");
    let compressed = gzip(&json);
    eprintln!(
        "License data: {} bytes JSON -> {} bytes gzip ({:.1}%)",
        json.len(),
        compressed.len(),
        (compressed.len() as f64 / json.len() as f64) * 100.0
    );
    fs::write(&output_path, compressed)
        .unwrap_or_else(|error| panic!("Failed to write compressed license file: {error}"));
    eprintln!("License generation completed: {}", output_path.display());
}

fn collect_rust_licenses(out_dir: &str, release: bool) -> Vec<LicenseGroup> {
    let target = std::env::var("TARGET").expect("TARGET env var not set");
    let about_json_path = Path::new(out_dir).join("cargo-about.json");

    eprintln!("Generating Rust dependency licenses (target: {target})...");
    let result = Command::new("cargo")
        .args([
            "about",
            "generate",
            "--format",
            "json",
            "--target",
            &target,
            "--output-file",
        ])
        .arg(&about_json_path)
        .output();

    let json = match result {
        Ok(result) if result.status.success() => fs::read(&about_json_path)
            .unwrap_or_else(|error| panic!("Failed to read cargo-about output: {error}")),
        Ok(result) => {
            let details = String::from_utf8_lossy(&result.stderr);
            let message = format!(
                "cargo-about failed to generate license output: {}",
                details.trim()
            );
            return missing_license_source(release, &message);
        }
        Err(error) => {
            let message = format!(
                "cargo-about could not be started ({error}). Install it with: \
                 cargo install cargo-about --locked"
            );
            return missing_license_source(release, &message);
        }
    };

    let parsed: CargoAboutOutput =
        serde_json::from_slice(&json).expect("Failed to parse cargo-about JSON output");
    parsed
        .licenses
        .into_iter()
        .map(|license| LicenseGroup {
            license: license.id,
            text: license.text,
            dependencies: license
                .used_by
                .into_iter()
                .map(|used| LicenseDep {
                    name: used.krate.name,
                    version: used.krate.version,
                })
                .collect(),
        })
        .collect()
}

fn collect_frontend_licenses(release: bool) -> Vec<LicenseGroup> {
    let snapshot_path = Path::new(FRONTEND_LICENSE_SNAPSHOT);
    if snapshot_path.is_file() {
        eprintln!("Loading frontend dependency licenses from the frontend-build artifact...");
        let json = fs::read(snapshot_path)
            .unwrap_or_else(|error| panic!("Failed to read {}: {error}", snapshot_path.display()));
        return parse_frontend_licenses(&json, true);
    }

    if !Path::new("frontend/node_modules").is_dir() {
        return missing_license_source(
            release,
            "Frontend license data is unavailable: neither the frontend-build artifact nor frontend/node_modules exists",
        );
    }

    eprintln!("Generating frontend dependency licenses from node_modules...");
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
        Ok(result) if result.status.success() => result,
        Ok(result) => {
            let details = String::from_utf8_lossy(&result.stderr);
            let message = format!("Frontend license check failed: {}", details.trim());
            return missing_license_source(release, &message);
        }
        Err(error) => {
            let message = format!("license-checker-rseidelsohn could not be started: {error}");
            return missing_license_source(release, &message);
        }
    };

    parse_frontend_licenses(&result.stdout, false)
}

fn parse_frontend_licenses(json: &[u8], portable_snapshot: bool) -> Vec<LicenseGroup> {
    let json: serde_json::Value =
        serde_json::from_slice(json).expect("Failed to parse frontend license JSON");
    let packages = json
        .as_object()
        .expect("Expected a JSON object from the frontend license checker");
    let mut groups: BTreeMap<String, (String, Option<String>, Vec<LicenseDep>)> = BTreeMap::new();

    for (package_id, info) in packages {
        let license_id = info["licenses"].as_str().unwrap_or("Unknown").to_string();
        let license_file = info["licenseFile"].as_str();
        let is_license_file = license_file.is_some_and(|path| {
            let name = path
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            name.starts_with("licen") || name.starts_with("copying")
        });
        let license_text = if is_license_file && portable_snapshot {
            info["licenseText"]
                .as_str()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_owned)
        } else if is_license_file {
            license_file
                .and_then(|path| fs::read_to_string(path).ok())
                .map(|text| text.trim().to_owned())
        } else {
            None
        };

        let (name, version) = package_id
            .rfind('@')
            .map_or((package_id.as_str(), ""), |position| {
                (&package_id[..position], &package_id[position + 1..])
            });
        let key = license_text.clone().unwrap_or_else(|| license_id.clone());
        let entry = groups
            .entry(key)
            .or_insert_with(|| (license_id, license_text, Vec::new()));
        entry.2.push(LicenseDep {
            name: name.to_owned(),
            version: version.to_owned(),
        });
    }

    groups
        .into_values()
        .map(|(license, text, dependencies)| LicenseGroup {
            text: text
                .unwrap_or_else(|| format!("(No license file found. SPDX identifier: {license})")),
            license,
            dependencies,
        })
        .collect()
}

fn accepted_licenses() -> Vec<String> {
    let content = fs::read_to_string("about.toml").expect("Failed to read about.toml");
    let toml: toml::Value = toml::from_str(&content).expect("Failed to parse about.toml");
    toml["accepted"]
        .as_array()
        .expect("about.toml must contain an accepted array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("Each accepted license must be a string")
                .to_owned()
        })
        .collect()
}

fn missing_license_source(release: bool, message: &str) -> Vec<LicenseGroup> {
    if release {
        panic!(
            "{message}. Release builds must contain complete third-party license data; \
             set KOSHELF_SKIP_LICENSE_GENERATION=1 only to opt out explicitly."
        );
    }
    println!("cargo:warning={message}; the unavailable license source will be omitted");
    Vec::new()
}

fn write_gzipped(path: &Path, bytes: &[u8]) {
    let compressed = gzip(bytes);
    write_if_changed(path, &compressed)
        .unwrap_or_else(|error| panic!("Failed to write {}: {error}", path.display()));
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(bytes)
        .expect("Failed to compress license data");
    encoder
        .finish()
        .expect("Failed to finish license compression")
}
