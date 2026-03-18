const READER_SUPPORTED_FORMATS = new Set(['epub', 'fb2', 'mobi', 'cbz']);

export function normalizeLibraryFormat(
    format: string | null | undefined,
): string | null {
    const normalized = format?.trim().toLowerCase();
    if (!normalized) {
        return null;
    }

    return normalized.startsWith('.') ? normalized.slice(1) : normalized;
}

export function isReaderFormatSupported(
    format: string | null | undefined,
): boolean {
    const normalized = normalizeLibraryFormat(format);
    return normalized ? READER_SUPPORTED_FORMATS.has(normalized) : false;
}
