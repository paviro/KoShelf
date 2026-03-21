function readFromStorage<T>(
    storage: Storage,
    key: string,
    defaultValue: T | null,
): T | null {
    try {
        const item = storage.getItem(key);
        if (item === null) return defaultValue;
        return JSON.parse(item) as T;
    } catch {
        return defaultValue;
    }
}

function writeToStorage(storage: Storage, key: string, value: unknown): void {
    try {
        storage.setItem(key, JSON.stringify(value));
    } catch {
        // Ignore storage failures.
    }
}

export class StorageManager {
    static readonly PREFIX = 'koshelf_';

    static getByKey<T>(key: string, defaultValue: T | null = null): T | null {
        return readFromStorage(localStorage, this.PREFIX + key, defaultValue);
    }

    static setByKey(key: string, value: unknown): void {
        writeToStorage(localStorage, this.PREFIX + key, value);
    }

    static removeByKey(key: string): void {
        try {
            localStorage.removeItem(this.PREFIX + key);
        } catch {
            // Ignore storage failures.
        }
    }

    static getSessionByKey<T>(
        key: string,
        defaultValue: T | null = null,
    ): T | null {
        return readFromStorage(sessionStorage, this.PREFIX + key, defaultValue);
    }

    static setSessionByKey(key: string, value: unknown): void {
        writeToStorage(sessionStorage, this.PREFIX + key, value);
    }

    static removeSessionByKey(key: string): void {
        try {
            sessionStorage.removeItem(this.PREFIX + key);
        } catch {
            // Ignore storage failures.
        }
    }
}
