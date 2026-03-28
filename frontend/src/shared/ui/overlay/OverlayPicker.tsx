import {
    useCallback,
    useEffect,
    useLayoutEffect,
    useRef,
    useState,
} from 'react';
import { createPortal } from 'react-dom';

import { computeOverlayPosition } from '../../overlay/anchored-overlay';

type OverlayPickerProps = {
    anchorRef: React.RefObject<HTMLElement | null>;
    onClose: () => void;
    children: React.ReactNode;
    className?: string;
};

export function OverlayPicker({
    anchorRef,
    onClose,
    children,
    className = '',
}: OverlayPickerProps) {
    const ref = useRef<HTMLDivElement>(null);
    const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

    const updatePosition = useCallback(() => {
        const anchor = anchorRef.current;
        const overlay = ref.current;
        if (!anchor || !overlay) return;

        const anchorRect = anchor.getBoundingClientRect();
        const overlayRect = overlay.getBoundingClientRect();
        const result = computeOverlayPosition(
            anchorRect,
            overlayRect,
            window.innerWidth,
            window.innerHeight,
            { placementOrder: ['bottom', 'top'], arrowSize: 0, gap: 4 },
        );
        setPos({ top: result.top, left: result.left });
    }, [anchorRef]);

    useLayoutEffect(() => {
        updatePosition();
    }, [updatePosition]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (
                ref.current &&
                !ref.current.contains(event.target as Node) &&
                !anchorRef.current?.contains(event.target as Node)
            ) {
                onClose();
            }
        };

        const handleEscape = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                onClose();
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleEscape);
        window.addEventListener('scroll', onClose, true);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleEscape);
            window.removeEventListener('scroll', onClose, true);
        };
    }, [onClose, anchorRef]);

    return createPortal(
        <div
            ref={ref}
            className={`fixed z-50 bg-white/95 dark:bg-dark-900/88 backdrop-blur-xs border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl p-2 ${className}`}
            style={{
                top: pos?.top ?? 0,
                left: pos?.left ?? 0,
                visibility: pos ? 'visible' : 'hidden',
            }}
        >
            {children}
        </div>,
        document.body,
    );
}
