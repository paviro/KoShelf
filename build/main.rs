mod compress;
mod licenses;
mod locale;
mod react;
mod shared;

use std::path::Path;

use shared::{env_flag, rerun_if_changed_recursive};

fn main() {
    rerun_if_changed_recursive(Path::new("assets"));
    rerun_if_changed_recursive(Path::new("src"));
    rerun_if_changed_recursive(Path::new("frontend/locales"));
    rerun_if_changed_recursive(Path::new("frontend/src"));
    rerun_if_changed_recursive(Path::new("frontend/public"));
    println!("cargo:rerun-if-changed=frontend/index.html");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/package-lock.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");
    println!("cargo:rerun-if-changed=frontend/tsconfig.json");
    println!("cargo:rerun-if-changed=frontend/tsconfig.node.json");
    println!("cargo:rerun-if-changed=frontend/postcss.config.cjs");
    println!("cargo:rerun-if-changed=frontend/tailwind.config.cjs");
    println!("cargo:rerun-if-changed=about.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-env-changed=KOSHELF_SKIP_NPM_INSTALL");
    println!("cargo:rerun-if-env-changed=KOSHELF_SKIP_REACT_BUILD");
    println!("cargo:rerun-if-env-changed=KOSHELF_SKIP_LICENSE_GENERATION");

    let skip_npm_install = env_flag("KOSHELF_SKIP_NPM_INSTALL");
    let skip_react_build = env_flag("KOSHELF_SKIP_REACT_BUILD");
    let skip_license_generation = env_flag("KOSHELF_SKIP_LICENSE_GENERATION");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Generate a compact locale manifest for CLI language listing.
    locale::generate_locale_manifest(&out_dir);

    // Build React + Vite frontend, then pre-compress text assets for embedding.
    react::compile_react_frontend(skip_npm_install, skip_react_build);
    compress::compress_frontend_dist(&out_dir);
    compress::compress_fonts(&out_dir);

    // Generate third-party license text for embedding
    licenses::generate_licenses(&out_dir, skip_license_generation);
}
