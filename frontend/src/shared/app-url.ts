/** Resolve application resources beneath the server's validated base URL. */
export function appUrl(path: string): string {
    if (!path.startsWith('/') || path.startsWith('//')) return path;
    const base = document.querySelector('base')?.getAttribute('href') ?? '/';
    return `${base}${path.slice(1)}`;
}

/** API media fields are root-relative in both live and exported contracts. */
export function resolveMediaUrls(value: unknown): unknown {
    if (Array.isArray(value)) return value.map(resolveMediaUrls);
    if (value && typeof value === 'object') {
        return Object.fromEntries(
            Object.entries(value).map(([key, item]) => [
                key,
                key.endsWith('_url') &&
                typeof item === 'string' &&
                item.startsWith('/assets/')
                    ? appUrl(item)
                    : resolveMediaUrls(item),
            ]),
        );
    }
    return value;
}
