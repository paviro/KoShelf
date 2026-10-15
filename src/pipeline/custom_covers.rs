//! Custom cover uploads written for KOReader as `cover.<ext>`.
use crate::source::custom::{MAX_SOURCE_BYTES, cover_extension};
use anyhow::{Context, Result, bail, ensure};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
};

/// Covers written for KOReader fit this width × height box (the KDP cover
/// standard), which still fills the largest e-ink screens.
const DEVICE_BOX: (u32, u32) = (1600, 2560);

/// Validates an upload and returns the extension and bytes for `cover.<ext>`.
/// Covers within [`DEVICE_BOX`] are kept byte-for-byte; larger ones are
/// downscaled to JPEG, or PNG when they have transparency.
pub fn device_bytes(data: &[u8]) -> Result<(String, Vec<u8>)> {
    ensure!(
        data.len() as u64 <= MAX_SOURCE_BYTES,
        "cover exceeds 20 MiB"
    );
    let ext = match image::guess_format(data) {
        Ok(ImageFormat::Jpeg) => "jpg",
        Ok(ImageFormat::Png) => "png",
        Ok(ImageFormat::Gif) => "gif",
        Ok(ImageFormat::WebP) => "webp",
        _ => bail!("unsupported cover format"),
    };
    let mut decoder = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let (width, height) = decoder.dimensions();
    let (width, height) = match orientation {
        image::metadata::Orientation::Rotate90
        | image::metadata::Orientation::Rotate270
        | image::metadata::Orientation::Rotate90FlipH
        | image::metadata::Orientation::Rotate270FlipH => (height, width),
        _ => (width, height),
    };
    let mut image = DynamicImage::from_decoder(decoder).context("invalid cover image")?;
    if width <= DEVICE_BOX.0 && height <= DEVICE_BOX.1 {
        return Ok((ext.to_owned(), data.to_vec()));
    }
    // Re-encoding drops EXIF, so bake the orientation into the pixels.
    image.apply_orientation(orientation);
    let image = image.resize(
        DEVICE_BOX.0,
        DEVICE_BOX.1,
        image::imageops::FilterType::Lanczos3,
    );
    let mut encoded = Cursor::new(Vec::new());
    if image.color().has_alpha() && image.to_rgba8().pixels().any(|p| p[3] < 255) {
        image.write_to(&mut encoded, ImageFormat::Png)?;
        return Ok(("png".to_owned(), encoded.into_inner()));
    }
    DynamicImage::ImageRgb8(image.to_rgb8()).write_with_encoder(
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 90),
    )?;
    Ok(("jpg".to_owned(), encoded.into_inner()))
}

/// Replace every `cover.*` in `folder` with the upload, or remove them all.
/// Call under the folder lock after revision validation.
pub fn replace(folder: &Path, upload: Option<(&str, &[u8])>) -> Result<()> {
    if let Some((ext, bytes)) = upload {
        fs::create_dir_all(folder)?;
        let mut staged = tempfile::NamedTempFile::new_in(folder)?;
        staged.write_all(bytes)?;
        staged.as_file().sync_all()?;
        staged
            .persist(folder.join(format!("cover.{ext}")))
            .map_err(|e| e.error)?;
    }
    let keep = upload.map(|(ext, _)| folder.join(format!("cover.{ext}")));
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    for entry in entries {
        let path = entry?.path();
        if cover_extension(&path).is_none() {
            continue;
        }
        if let Some(keep) = keep.as_deref()
            && is_upload(&path, keep)?
        {
            continue;
        }
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Case-insensitive filesystems keep an existing `cover.PNG` name when
/// `cover.png` replaces it, so match the upload by file identity, not name.
#[cfg(unix)]
fn is_upload(path: &Path, upload: &Path) -> Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let (a, b) = (fs::metadata(path)?, fs::metadata(upload)?);
    Ok(a.dev() == b.dev() && a.ino() == b.ino())
}

#[cfg(not(unix))]
fn is_upload(path: &Path, upload: &Path) -> Result<bool> {
    let name = |p: &Path| p.file_name().and_then(|n| n.to_str()).map(str::to_owned);
    Ok(name(path)
        .zip(name(upload))
        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(&b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgb8(width, height)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn small_covers_are_kept_and_large_ones_downscaled() {
        let small = png(10, 20);
        assert_eq!(device_bytes(&small).unwrap(), ("png".into(), small));
        let (ext, bytes) = device_bytes(&png(3200, 100)).unwrap();
        assert_eq!(ext, "jpg");
        assert_eq!(image::load_from_memory(&bytes).unwrap().width(), 1600);
        assert!(device_bytes(b"<svg/>").is_err());
    }

    #[test]
    fn replacement_removes_siblings_and_reset_removes_all() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("cover.PNG"), b"original").unwrap();
        fs::write(dir.path().join("custom_metadata.lua"), b"kept").unwrap();
        replace(dir.path(), Some(("jpg", b"new"))).unwrap();
        assert!(!dir.path().join("cover.PNG").exists());
        assert_eq!(fs::read(dir.path().join("cover.jpg")).unwrap(), b"new");
        replace(dir.path(), None).unwrap();
        assert!(!dir.path().join("cover.jpg").exists());
        assert!(dir.path().join("custom_metadata.lua").exists());
    }

    #[test]
    fn replacement_differing_only_in_case_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("cover.PNG"), b"original").unwrap();
        replace(dir.path(), Some(("png", b"new"))).unwrap();
        let covers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(covers.len(), 1);
        assert_eq!(fs::read(&covers[0]).unwrap(), b"new");
    }
}
