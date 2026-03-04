import { useEffect, type RefObject } from 'react';

export function useClickOutside<T extends HTMLElement>(
    ref: RefObject<T | null>,
    handler: () => void,
    enabled = true,
): void {
    useEffect(() => {
        if (!enabled) {
            return;
        }

        const listener = (event: MouseEvent): void => {
            const target = event.target instanceof Node ? event.target : null;
            if (!target || !ref.current) {
                return;
            }

            if (!ref.current.contains(target)) {
                handler();
            }
        };

        document.addEventListener('mousedown', listener);
        return () => {
            document.removeEventListener('mousedown', listener);
        };
    }, [enabled, handler, ref]);
}
