import { useSyncExternalStore } from 'react';

export type ToastVariant = 'success' | 'error' | 'warning' | 'info';

export type ToastEntry = {
    id: number;
    variant: ToastVariant;
    message: string;
    subtitle?: string;
    durationMs: number;
};

type Listener = () => void;

let nextId = 1;
let toasts: ToastEntry[] = [];
const listeners = new Set<Listener>();

function emit() {
    for (const listener of listeners) {
        listener();
    }
}

function subscribe(listener: Listener): () => void {
    listeners.add(listener);
    return () => {
        listeners.delete(listener);
    };
}

function getSnapshot(): ToastEntry[] {
    return toasts;
}

export function addToast(
    variant: ToastVariant,
    message: string,
    options?: { subtitle?: string; durationMs?: number },
): number {
    const id = nextId++;
    toasts = [
        ...toasts,
        {
            id,
            variant,
            message,
            subtitle: options?.subtitle,
            durationMs: options?.durationMs ?? 4000,
        },
    ];
    emit();
    return id;
}

export function dismissToast(id: number): void {
    toasts = toasts.filter((t) => t.id !== id);
    emit();
}

export function useToastStore(): ToastEntry[] {
    return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}
