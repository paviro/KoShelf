import { StorageManager } from './storage-manager';

export type ThemePreference = 'auto' | 'light' | 'dark';

const THEME_STORAGE_KEY = 'theme_preference';
const DARK_MEDIA_QUERY = '(prefers-color-scheme: dark)';

export const THEME_PREFERENCE_CHANGE_EVENT = 'koshelf:theme-preference-changed';

let currentPreference: ThemePreference | null = null;
let mediaQueryList: MediaQueryList | null = null;
let removeMediaQueryListener: (() => void) | null = null;

function normalizeThemePreference(value: unknown): ThemePreference {
    if (value === 'light' || value === 'dark' || value === 'auto') {
        return value;
    }
    return 'auto';
}

function readStoredThemePreference(): ThemePreference {
    return normalizeThemePreference(
        StorageManager.getByKey<string>(THEME_STORAGE_KEY, 'auto'),
    );
}

function emitThemePreferenceChanged(): void {
    if (typeof window === 'undefined') {
        return;
    }
    window.dispatchEvent(new Event(THEME_PREFERENCE_CHANGE_EVENT));
}

function resolveShouldUseDark(preference: ThemePreference): boolean {
    if (preference === 'dark') {
        return true;
    }
    if (preference === 'light') {
        return false;
    }
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
        return false;
    }
    return window.matchMedia(DARK_MEDIA_QUERY).matches;
}

function applyThemePreference(preference: ThemePreference): void {
    if (typeof document === 'undefined') {
        return;
    }

    const shouldUseDark = resolveShouldUseDark(preference);
    document.documentElement.classList.toggle('dark', shouldUseDark);
    document.documentElement.style.colorScheme = shouldUseDark ? 'dark' : 'light';
}

function detachSystemThemeListener(): void {
    removeMediaQueryListener?.();
    removeMediaQueryListener = null;
    mediaQueryList = null;
}

function attachSystemThemeListener(): void {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
        return;
    }

    detachSystemThemeListener();

    mediaQueryList = window.matchMedia(DARK_MEDIA_QUERY);
    const handleSystemThemeChange = () => {
        if (currentPreference === 'auto') {
            applyThemePreference('auto');
        }
    };

    if (typeof mediaQueryList.addEventListener === 'function') {
        mediaQueryList.addEventListener('change', handleSystemThemeChange);
        removeMediaQueryListener = () => {
            mediaQueryList?.removeEventListener('change', handleSystemThemeChange);
        };
        return;
    }

    mediaQueryList.addListener(handleSystemThemeChange);
    removeMediaQueryListener = () => {
        mediaQueryList?.removeListener(handleSystemThemeChange);
    };
}

function syncSystemThemeListener(preference: ThemePreference): void {
    if (preference === 'auto') {
        attachSystemThemeListener();
        return;
    }
    detachSystemThemeListener();
}

export function initThemePreference(): ThemePreference {
    const preference = readStoredThemePreference();
    currentPreference = preference;
    applyThemePreference(preference);
    syncSystemThemeListener(preference);
    return preference;
}

export function getThemePreference(): ThemePreference {
    if (currentPreference) {
        return currentPreference;
    }
    return initThemePreference();
}

export function setThemePreference(preference: ThemePreference): void {
    const normalizedPreference = normalizeThemePreference(preference);
    currentPreference = normalizedPreference;
    StorageManager.setByKey(THEME_STORAGE_KEY, normalizedPreference);
    applyThemePreference(normalizedPreference);
    syncSystemThemeListener(normalizedPreference);
    emitThemePreferenceChanged();
}
