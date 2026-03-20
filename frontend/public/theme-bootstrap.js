(() => {
    const STORAGE_KEY = 'koshelf_theme_preference';
    const DARK_QUERY = '(prefers-color-scheme: dark)';

    try {
        const rawValue = localStorage.getItem(STORAGE_KEY);
        const parsedValue = rawValue === null ? null : JSON.parse(rawValue);
        const preference =
            parsedValue === 'light' ||
            parsedValue === 'dark' ||
            parsedValue === 'auto'
                ? parsedValue
                : 'auto';
        const useDark =
            preference === 'dark' ||
            (preference === 'auto' &&
                window.matchMedia &&
                window.matchMedia(DARK_QUERY).matches);

        document.documentElement.classList.toggle('dark', useDark);
        document.documentElement.style.colorScheme = useDark ? 'dark' : 'light';
    } catch {}
})();
