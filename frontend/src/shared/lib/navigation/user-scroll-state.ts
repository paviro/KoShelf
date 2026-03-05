import { useSyncExternalStore } from 'react';
import { isScrollRestoringNow } from './scroll-restore-state';

let hasUserScrolled = false;
let isListening = false;
const listeners = new Set<() => void>();

function notifyListeners(): void {
    listeners.forEach((listener) => {
        listener();
    });
}

function stopListening(): void {
    if (!isListening || typeof window === 'undefined') {
        return;
    }

    window.removeEventListener('scroll', handleScroll);
    isListening = false;
}

function handleScroll(): void {
    if (hasUserScrolled || isScrollRestoringNow()) {
        return;
    }

    hasUserScrolled = true;
    stopListening();
    notifyListeners();
}

function ensureListening(): void {
    if (hasUserScrolled || isListening || typeof window === 'undefined') {
        return;
    }

    window.addEventListener('scroll', handleScroll, { passive: true });
    isListening = true;
}

function getSnapshot(): boolean {
    return hasUserScrolled;
}

function subscribe(listener: () => void): () => void {
    listeners.add(listener);
    ensureListening();

    return () => {
        listeners.delete(listener);
    };
}

ensureListening();

export function useHasUserScrolled(): boolean {
    return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}
