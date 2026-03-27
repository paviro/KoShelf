import { type MouseEvent, type ReactNode } from 'react';

import { useOverlayAnimation } from '../../lib/dom/useOverlayAnimation';
import { CloseButton } from '../button/CloseButton';

type ModalShellProps = {
    open: boolean;
    onClose: () => void;
    children: ReactNode;
    containerClassName?: string;
    cardClassName?: string;
    showCloseButton?: boolean;
};

export function ModalShell({
    open,
    onClose,
    children,
    containerClassName = '',
    cardClassName = '',
    showCloseButton = true,
}: ModalShellProps) {
    const { isMounted, isVisible, backdropRef } = useOverlayAnimation(
        open,
        onClose,
    );

    if (!isMounted) {
        return null;
    }

    const handleBackdropMouseDown = (event: MouseEvent<HTMLDivElement>) => {
        if (event.target === event.currentTarget) {
            onClose();
        }
    };

    return (
        <div
            ref={backdropRef}
            className={`fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 transition-opacity duration-300 ${
                isVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'
            } ${containerClassName}`}
            onMouseDown={handleBackdropMouseDown}
        >
            <div
                className={`relative w-full transform transition-all duration-300 ${
                    isVisible ? 'opacity-100 scale-100' : 'opacity-0 scale-95'
                } ${
                    cardClassName
                        ? ''
                        : 'bg-white/95 dark:bg-dark-900/90 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl'
                } ${cardClassName}`}
                role="dialog"
                aria-modal="true"
                onMouseDown={(event) => event.stopPropagation()}
            >
                {showCloseButton && (
                    <CloseButton
                        onClick={onClose}
                        className="absolute top-4 right-4 rounded-full p-2"
                    />
                )}
                {children}
            </div>
        </div>
    );
}
