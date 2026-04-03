use anyhow::Result;
use std::collections::BTreeMap;
use std::io::Read as _;

#[derive(serde::Deserialize)]
struct LicenseGroup {
    license: String,
    text: String,
    dependencies: Vec<LicenseDep>,
}

#[derive(serde::Deserialize)]
struct LicenseDep {
    name: String,
    version: String,
}

pub(crate) fn print_licenses(dependency: Option<String>) -> Result<()> {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/LICENSES.json.gz"));
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut json = String::new();
    decoder
        .read_to_string(&mut json)
        .expect("Failed to decompress license data");
    let data: Vec<LicenseGroup> =
        serde_json::from_str(&json).expect("Failed to parse embedded license data");

    match dependency {
        None => {
            let mut by_spdx: BTreeMap<&str, Vec<&LicenseDep>> = BTreeMap::new();
            for group in &data {
                by_spdx
                    .entry(&group.license)
                    .or_default()
                    .extend(&group.dependencies);
            }

            for (spdx, deps) in &by_spdx {
                let count = deps.len();
                println!(
                    "{} ({} {})",
                    spdx,
                    count,
                    if count == 1 {
                        "dependency"
                    } else {
                        "dependencies"
                    }
                );
                for dep in deps {
                    if dep.version.is_empty() {
                        println!("  {}", dep.name);
                    } else {
                        println!("  {} {}", dep.name, dep.version);
                    }
                }
                println!();
            }
            println!("Use 'koshelf licenses <dependency>' to view the full license text.");
        }
        Some(query) => {
            let query_lower = query.to_lowercase();

            // Try exact match first, then substring
            let mut matches: Vec<(&LicenseDep, &LicenseGroup)> = Vec::new();
            for group in &data {
                for dep in &group.dependencies {
                    if dep.name.to_lowercase() == query_lower {
                        matches.push((dep, group));
                    }
                }
            }
            if matches.is_empty() {
                for group in &data {
                    for dep in &group.dependencies {
                        if dep.name.to_lowercase().contains(&query_lower) {
                            matches.push((dep, group));
                        }
                    }
                }
            }

            match matches.len() {
                0 => {
                    anyhow::bail!("No dependency found matching '{query}'");
                }
                1 => {
                    let (dep, group) = matches[0];
                    if dep.version.is_empty() {
                        println!("{} — {}\n", dep.name, group.license);
                    } else {
                        println!("{} {} — {}\n", dep.name, dep.version, group.license);
                    }
                    println!("{}", group.text);
                }
                _ => {
                    // Multiple matches — check if they all share the same license group
                    let first_license = &matches[0].1.license;
                    let first_text = &matches[0].1.text;
                    let all_same = matches
                        .iter()
                        .all(|(_, g)| g.license == *first_license && g.text == *first_text);

                    if all_same {
                        for (dep, _) in &matches {
                            if dep.version.is_empty() {
                                println!("{} — {}", dep.name, first_license);
                            } else {
                                println!("{} {} — {}", dep.name, dep.version, first_license);
                            }
                        }
                        println!("\n{}", first_text);
                    } else {
                        println!("Multiple dependencies match '{query}':\n");
                        for (dep, group) in &matches {
                            if dep.version.is_empty() {
                                println!("  {} ({})", dep.name, group.license);
                            } else {
                                println!("  {} {} ({})", dep.name, dep.version, group.license);
                            }
                        }
                        println!("\nPlease specify the exact dependency name.");
                    }
                }
            }
        }
    }
    Ok(())
}
