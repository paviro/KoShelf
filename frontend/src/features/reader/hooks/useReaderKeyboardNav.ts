import { useEffect } from 'react';

function shouldIgnoreKeyboardNavigation(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) {
        return false;
    }

    const tagName = target.tagName.toLowerCase();
    if (tagName === 'input' || tagName === 'textarea' || tagName === 'select') {
        return true;
    }

    return target.isContentEditable;
}

export function useReaderKeyboardNav(
    onPrev: () => void,
    onNext: () => void,
): void {
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (
                e.defaultPrevented ||
                shouldIgnoreKeyboardNavigation(e.target)
            ) {
                return;
            }

            if (e.key === 'ArrowLeft') {
                e.preventDefault();
                onPrev();
            } else if (e.key === 'ArrowRight') {
                e.preventDefault();
                onNext();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onPrev, onNext]);
}
