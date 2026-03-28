import { useEffect } from 'react';

export function useEscapeKey(handler: () => void, enabled = true): void {
    useEffect(() => {
        if (!enabled) return;

        const onKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                handler();
            }
        };

        document.addEventListener('keydown', onKeyDown);
        return () => document.removeEventListener('keydown', onKeyDown);
    }, [handler, enabled]);
}
