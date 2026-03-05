import { useSyncExternalStore } from 'react';

let activeRestoreCount = 0;
const listeners = new Set<() => void>();

function notifyListeners(): void {
    listeners.forEach((listener) => {
        listener();
    });
}

function getSnapshot(): boolean {
    return activeRestoreCount > 0;
}

function subscribe(listener: () => void): () => void {
    listeners.add(listener);
    return () => {
        listeners.delete(listener);
    };
}

export function beginScrollRestore(): void {
    activeRestoreCount += 1;
    notifyListeners();
}

export function endScrollRestore(): void {
    if (activeRestoreCount === 0) {
        return;
    }

    activeRestoreCount -= 1;
    notifyListeners();
}

export function useIsScrollRestoring(): boolean {
    return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

export function isScrollRestoringNow(): boolean {
    return getSnapshot();
}
