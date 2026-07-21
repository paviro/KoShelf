use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::shared::write_if_changed;

/// Copy `frontend/dist` to `$OUT_DIR/frontend_dist`, gzip-compressing text-based
/// assets to reduce the embedded binary size.
pub(crate) fn compress_frontend_dist(out_dir: &str) {
    let src = Path::new("frontend").join("dist");
    let dst = Path::new(out_dir).join("frontend_dist");

    if !src.exists() {
        panic!(
            "frontend/dist not found. Build the frontend or download the frontend-build artifact."
        );
    }

    let mut total_original: u64 = 0;
    let mut total_compressed: u64 = 0;

    copy_dir_compressed(&src, &dst, &mut total_original, &mut total_compressed);

    if total_original > 0 {
        eprintln!(
            "Frontend assets: {} bytes -> {} bytes ({:.1}%)",
            total_original,
            total_compressed,
            (total_compressed as f64 / total_original as f64) * 100.0
        );
    }
}

fn copy_dir_compressed(
    src: &Path,
    dst: &Path,
    total_original: &mut u64,
    total_compressed: &mut u64,
) {
    fs::create_dir_all(dst).unwrap_or_else(|e| panic!("Failed to create {}: {}", dst.display(), e));

    for entry in fs::read_dir(src)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", src.display(), e))
        .flatten()
    {
        let path = entry.path();
        let dest = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_compressed(&path, &dest, total_original, total_compressed);
        } else if path.is_file() {
            let data = fs::read(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
            let original_len = data.len() as u64;

            if is_compressible_extension(&path) {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
                encoder.write_all(&data).unwrap();
                let compressed = encoder.finish().unwrap();
                let compressed_len = compressed.len() as u64;

                *total_original += original_len;
                *total_compressed += compressed_len;

                let _ = write_if_changed(&dest, &compressed);
            } else {
                *total_original += original_len;
                *total_compressed += original_len;

                let _ = write_if_changed(&dest, &data);
            }
        }
    }
}

fn is_compressible_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("js" | "mjs" | "css" | "html" | "json" | "map" | "svg")
    )
}

/// Gzip-compress the embedded Gelasio font files to `$OUT_DIR`.
pub(crate) fn compress_fonts(out_dir: &str) {
    let fonts = [
        (
            "frontend/build-assets/fonts/Gelasio_400Regular.ttf",
            "frontend/node_modules/@expo-google-fonts/gelasio/400Regular/Gelasio_400Regular.ttf",
            "Gelasio_400Regular.ttf.gz",
        ),
        (
            "frontend/build-assets/fonts/Gelasio_400Regular_Italic.ttf",
            "frontend/node_modules/@expo-google-fonts/gelasio/400Regular_Italic/Gelasio_400Regular_Italic.ttf",
            "Gelasio_400Regular_Italic.ttf.gz",
        ),
    ];

    for (bundled_src, node_modules_src, dst_name) in &fonts {
        let bundled_path = Path::new(bundled_src);
        let node_modules_path = Path::new(node_modules_src);
        let src_path = if bundled_path.is_file() {
            bundled_path
        } else if node_modules_path.is_file() {
            node_modules_path
        } else {
            panic!(
                "Font {dst_name} is missing from both the frontend-build artifact and node_modules"
            );
        };

        let data = fs::read(src_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", src_path.display(), e));

        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        eprintln!(
            "Font {}: {} bytes -> {} bytes ({:.1}%)",
            dst_name,
            data.len(),
            compressed.len(),
            (compressed.len() as f64 / data.len() as f64) * 100.0
        );

        let dest = Path::new(out_dir).join(dst_name);
        let _ = write_if_changed(&dest, &compressed);
    }
}
