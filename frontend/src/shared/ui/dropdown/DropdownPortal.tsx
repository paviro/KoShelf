import {
    useEffect,
    useId,
    useRef,
    type ReactNode,
    type RefObject,
} from 'react';
import { createPortal } from 'react-dom';

import type {
    OverlayAlignmentOption,
    OverlayPlacement,
} from '../../overlay/anchored-overlay';
import { useAnchoredPosition } from '../../overlay/useAnchoredPosition';
import { useClickOutside } from '../../lib/dom/useClickOutside';
import { useEscapeKey } from '../../lib/dom/useEscapeKey';

type DropdownPortalProps = {
    triggerRef: RefObject<HTMLElement | null>;
    open: boolean;
    onClose: () => void;
    className?: string;
    children: ReactNode;
    placements?: OverlayPlacement[];
    alignment?: OverlayAlignmentOption;
    gap?: number;
    role?: 'menu' | 'dialog';
    'aria-label'?: string;
};

export function DropdownPortal({
    triggerRef,
    open,
    onClose,
    className = '',
    children,
    placements = ['bottom', 'top'],
    alignment = 'end',
    gap = 6,
    role = 'menu',
    'aria-label': ariaLabel,
}: DropdownPortalProps) {
    const panelRef = useRef<HTMLDivElement>(null);
    const panelId = useId();

    useAnchoredPosition(triggerRef, panelRef, open, onClose, {
        placements,
        alignment,
        arrowSize: 0,
        gap,
    });

    useClickOutside(panelRef, onClose, open, triggerRef);
    useEscapeKey(onClose, open);

    useEffect(() => {
        const trigger = triggerRef.current;
        if (!trigger) return;

        trigger.setAttribute('aria-haspopup', role);
        trigger.setAttribute('aria-expanded', String(open));

        if (open) {
            trigger.setAttribute('aria-controls', panelId);
        } else {
            trigger.removeAttribute('aria-controls');
        }
    }, [open, triggerRef, panelId, role]);

    if (!open) {
        return null;
    }

    return createPortal(
        <div
            ref={panelRef}
            id={panelId}
            role={role}
            aria-label={ariaLabel}
            className={`fixed z-50 ${className}`}
            style={{ visibility: 'hidden' }}
        >
            {children}
        </div>,
        document.body,
    );
}
