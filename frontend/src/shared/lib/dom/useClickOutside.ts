import { useEffect, type RefObject } from 'react';

export function useClickOutside<T extends HTMLElement>(
    ref: RefObject<T | null>,
    handler: () => void,
    enabled = true,
    excludeRef?: RefObject<HTMLElement | null>,
): void {
    useEffect(() => {
        if (!enabled) {
            return;
        }

        const isOutside = (target: EventTarget | null): boolean => {
            if (!(target instanceof Node) || !ref.current) {
                return false;
            }

            if (ref.current.contains(target)) {
                return false;
            }

            if (excludeRef?.current?.contains(target)) {
                return false;
            }

            return true;
        };

        const listener = (event: MouseEvent): void => {
            if (!isOutside(event.target)) {
                return;
            }

            handler();
        };

        document.addEventListener('mousedown', listener);
        return () => {
            document.removeEventListener('mousedown', listener);
        };
    }, [enabled, excludeRef, handler, ref]);
}
