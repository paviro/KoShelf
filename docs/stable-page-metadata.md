# Stable Page Metadata & Scaling

KoShelf can use KOReader stable page metadata to improve page totals and page-based stats.

## How It Works

- **Stable total pages for display** are used when KOReader metadata contains:
  - `pagemap_doc_pages > 0`
- This display behavior works for both publisher labels and synthetic mode.
- **Synthetic page scaling for statistics** is applied only when synthetic metadata is also present:
  - `pagemap_doc_pages > 0`
  - `pagemap_chars_per_synthetic_page`

## Publisher Labels vs. Synthetic Mode

- If you use publisher labels without synthetic override, KoShelf still shows stable total pages, but page-based statistics stay unscaled.
- Why publisher-label mode stays unscaled: KoShelf rescales stats using one linear factor (`stable_total / rendered_total`) across page events. That works for KOReader synthetic pagination (uniform char-based pages), but publisher labels are often non-linear (front matter, skipped/duplicate labels, appendix jumps). Applying one factor there would distort pages/day and pages/hour.
- If these `pagemap_*` fields are missing, KoShelf uses KOReader's normal `doc_pages`/statistics values and does not apply synthetic scaling.

## Recommendation

For consistent page-based comparisons between books, enable KOReader's `Override publisher page numbers` setting. This makes KOReader persist synthetic metadata, which lets KoShelf rescale page metrics across books.

## Compatibility

- This feature requires KOReader nightly builds or a future stable release after `2025.10 "Ghost"`.
- KOReader `2025.10 "Ghost"` does not write the required `pagemap_*` metadata fields, so KoShelf uses its standard unscaled page behavior.
- After updating to a KOReader build newer than `2025.10 "Ghost"`, you can use [KoReader-PopulateStablePageKOReader](https://github.com/paviro/KoReader-PopulateStablePage) to backfill stable page metadata for already finished books.

## Disabling

Use the `--ignore-stable-page-metadata` flag to disable this feature entirely. See [Configuration](configuration.md#common-options) for details.
