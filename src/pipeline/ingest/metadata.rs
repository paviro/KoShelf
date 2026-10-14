pub(crate) use crate::source::metadata::MetadataIndices;
#[cfg(test)]
use crate::source::metadata::locate_metadata_path;
#[cfg(test)]
mod tests {
    use super::{MetadataIndices, locate_metadata_path};
    use crate::shelf::models::LibraryItemFormat;
    use crate::source::scanner::MetadataLocation;
    use std::path::Path;

    fn write_metadata(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("metadata parent"))
            .expect("metadata parent dir");
        std::fs::write(path, "return {}\n").expect("metadata file");
    }

    #[test]
    fn docsettings_lookup_matches_regular_epub_by_stem_and_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        let metadata_path = dir
            .path()
            .join("mnt")
            .join("onboard")
            .join("books")
            .join("Book Title.sdr")
            .join("metadata.epub.lua");
        write_metadata(&metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");
        let book_path = dir.path().join("library").join("Book Title.epub");

        assert_eq!(
            locate_metadata_path(&indices, &book_path, LibraryItemFormat::Epub),
            Some(metadata_path)
        );
    }

    #[test]
    fn docsettings_lookup_matches_extensionless_kobo_epub_by_stem_and_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        let kobo_id = "428047c7-8b1e-4d15-80d1-4d12829cd185";
        let metadata_path = dir
            .path()
            .join("mnt")
            .join("onboard")
            .join(".kobo")
            .join("kepub")
            .join(format!("{}.sdr", kobo_id))
            .join("metadata.epub.lua");
        write_metadata(&metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");
        let book_path = dir.path().join("books").join(kobo_id);

        assert_eq!(
            locate_metadata_path(&indices, &book_path, LibraryItemFormat::Epub),
            Some(metadata_path)
        );
    }

    #[test]
    fn docsettings_lookup_keeps_same_stem_different_formats_distinct() {
        let dir = tempfile::tempdir().expect("temp dir");
        let sidecar_dir = dir.path().join("mnt").join("onboard").join("Book.sdr");
        let epub_metadata_path = sidecar_dir.join("metadata.epub.lua");
        let fb2_metadata_path = sidecar_dir.join("metadata.fb2.lua");
        write_metadata(&epub_metadata_path);
        write_metadata(&fb2_metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");

        assert_eq!(
            locate_metadata_path(
                &indices,
                &dir.path().join("Book.epub"),
                LibraryItemFormat::Epub
            ),
            Some(epub_metadata_path)
        );
        assert_eq!(
            locate_metadata_path(
                &indices,
                &dir.path().join("Book.fb2"),
                LibraryItemFormat::Fb2
            ),
            Some(fb2_metadata_path)
        );
    }

    #[test]
    fn in_book_lookup_matches_fb2_zip_metadata_zip_sidecar() {
        let dir = tempfile::tempdir().expect("temp dir");
        let book_path = dir.path().join("Book.fb2.zip");
        let metadata_path = dir.path().join("Book.fb2.sdr").join("metadata.zip.lua");
        write_metadata(&metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::InBookFolder).expect("metadata indices");

        assert_eq!(
            locate_metadata_path(&indices, &book_path, LibraryItemFormat::Fb2),
            Some(metadata_path)
        );
    }

    #[test]
    fn docsettings_lookup_matches_fb2_zip_metadata_zip_by_sidecar_stem() {
        let dir = tempfile::tempdir().expect("temp dir");
        let metadata_path = dir
            .path()
            .join("mnt")
            .join("onboard")
            .join("books")
            .join("Book.fb2.sdr")
            .join("metadata.zip.lua");
        write_metadata(&metadata_path);

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf()))
                .expect("metadata indices");
        let book_path = dir.path().join("library").join("Book.fb2.zip");

        assert_eq!(
            locate_metadata_path(&indices, &book_path, LibraryItemFormat::Fb2),
            Some(metadata_path)
        );
    }

    #[test]
    fn docsettings_index_rejects_duplicate_same_stem_and_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        write_metadata(
            &dir.path()
                .join("mnt")
                .join("onboard")
                .join("books")
                .join("Book.sdr")
                .join("metadata.epub.lua"),
        );
        write_metadata(
            &dir.path()
                .join("storage")
                .join("emulated")
                .join("0")
                .join("books")
                .join("Book.sdr")
                .join("metadata.epub.lua"),
        );

        let indices =
            MetadataIndices::new(&MetadataLocation::DocSettings(dir.path().to_path_buf())).unwrap();
        let resolution = indices.resolve(
            std::path::Path::new("Book.epub"),
            LibraryItemFormat::Epub,
            "",
            None,
        );
        assert!(resolution.blocked);
        assert!(
            resolution.issues.iter().any(|issue| matches!(
                issue,
                crate::source::metadata::DiscoveryIssue::Ambiguous(_)
            ))
        );
    }
}
