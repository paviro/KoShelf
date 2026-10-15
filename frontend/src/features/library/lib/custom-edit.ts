import type { MetadataField, MetadataPatch } from '../../../shared/api-client';
import type { CustomEdit } from '../../../shared/contracts';

/** Whether the custom metadata/cover editor can be opened for an item. */
export function canEditCustom(edit: CustomEdit | null | undefined): boolean {
    return Boolean(edit && (edit.metadata.available || edit.cover.available));
}

const NUMBER = /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/;

/** KOReader stores series indices as numbers; blank clears the override. */
export function isValidSeriesIndex(value: string): boolean {
    const text = value.trim();
    return text === '' || (NUMBER.test(text) && Number.isFinite(Number(text)));
}

/**
 * Normalize a metadata value the way the backend compares it: surrounding
 * whitespace, empty list entries, and number formatting carry no meaning.
 */
export function normalizeMetadataValue(
    field: MetadataField,
    value: string,
): string {
    if (field === 'series_index' && value.trim() && isValidSeriesIndex(value)) {
        return String(Number(value.trim()));
    }
    if (field === 'authors' || field === 'keywords') {
        return value
            .split('\n')
            .map((line) => line.trim())
            .filter(Boolean)
            .join('\n');
    }
    return value.trim();
}

/** Whether `text` differs from the ebook value enough to be an override. */
export function isMetadataOverride(
    field: MetadataField,
    text: string,
    base: string,
): boolean {
    return (
        text.trim() !== '' &&
        normalizeMetadataValue(field, text) !==
            normalizeMetadataValue(field, base)
    );
}

/**
 * Turn typed drafts into the minimal patch: values that differ from the ebook
 * become overrides, everything else clears a stored override.
 */
export function buildMetadataPatch(
    edit: Pick<CustomEdit, 'base' | 'overrides'>,
    drafts: Partial<Record<MetadataField, string>>,
): MetadataPatch {
    const patch: MetadataPatch = {};
    for (const [field, text] of Object.entries(drafts) as [
        MetadataField,
        string,
    ][]) {
        const base = edit.base[field] ?? '';
        const next = isMetadataOverride(field, text, base) ? text : null;
        if (next !== (edit.overrides[field] ?? null)) patch[field] = next;
    }
    return patch;
}
