import {
    useEffect,
    useLayoutEffect,
    useRef,
    useState,
    type CSSProperties,
    type ReactNode,
    type RefObject,
} from 'react';
import { createPortal } from 'react-dom';

type DropdownPortalProps = {
    triggerRef: RefObject<HTMLElement | null>;
    open: boolean;
    onClose: () => void;
    closeOnScroll?: boolean;
    className?: string;
    children: ReactNode;
};

export function DropdownPortal({
    triggerRef,
    open,
    onClose,
    closeOnScroll = false,
    className = '',
    children,
}: DropdownPortalProps) {
    const panelRef = useRef<HTMLDivElement>(null);
    const [style, setStyle] = useState<CSSProperties>({});

    useLayoutEffect(() => {
        if (!open || !triggerRef.current) {
            return;
        }

        const update = () => {
            const rect = triggerRef.current!.getBoundingClientRect();
            setStyle({
                position: 'fixed',
                top: rect.bottom + 6,
                right: window.innerWidth - rect.right,
            });
        };

        update();
        window.addEventListener('resize', update);
        return () => {
            window.removeEventListener('resize', update);
        };
    }, [open, triggerRef]);

    useEffect(() => {
        if (!open) {
            return;
        }

        const onMouseDown = (event: MouseEvent) => {
            const target = event.target;
            if (!(target instanceof Node)) {
                return;
            }
            if (triggerRef.current?.contains(target)) {
                return;
            }
            if (panelRef.current?.contains(target)) {
                return;
            }
            onClose();
        };

        const onScroll = closeOnScroll
            ? (event: Event) => {
                  if (panelRef.current?.contains(event.target as Node)) {
                      return;
                  }
                  onClose();
              }
            : null;

        document.addEventListener('mousedown', onMouseDown);
        if (onScroll) {
            document.addEventListener('scroll', onScroll, true);
        }
        return () => {
            document.removeEventListener('mousedown', onMouseDown);
            if (onScroll) {
                document.removeEventListener('scroll', onScroll, true);
            }
        };
    }, [open, onClose, closeOnScroll, triggerRef]);

    if (!open) {
        return null;
    }

    return createPortal(
        <div ref={panelRef} className={`z-50 ${className}`} style={style}>
            {children}
        </div>,
        document.body,
    );
}
