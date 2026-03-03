import type { CSSProperties } from 'react';

type SpinnerSize = 'sm' | 'md' | 'lg';

const SIZE_CLASSNAME: Record<SpinnerSize, string> = {
    sm: 'h-5 w-5 border-2',
    md: 'h-8 w-8 border-2',
    lg: 'h-12 w-12 border-[3px]',
};

type LoadingSpinnerProps = {
    size?: SpinnerSize;
    containerClassName?: string;
    spinnerClassName?: string;
    srLabel?: string;
    delayed?: boolean;
    delayMs?: number;
};

export function LoadingSpinner({
    size = 'md',
    containerClassName = '',
    spinnerClassName = '',
    srLabel = 'Loading',
    delayed = true,
    delayMs = 100,
}: LoadingSpinnerProps) {
    const delayedStyle: CSSProperties | undefined = delayed
        ? { opacity: 0, animationDelay: `${Math.max(0, delayMs)}ms` }
        : undefined;

    return (
        <div
            className={`flex items-center justify-center ${delayed ? 'loading-reveal-delayed' : ''} ${containerClassName}`}
            style={delayedStyle}
            role="status"
            aria-live="polite"
            aria-label={srLabel}
        >
            <div
                className={`rounded-full animate-spin border-primary-200 dark:border-primary-900 border-t-primary-500 dark:border-t-primary-300 ${SIZE_CLASSNAME[size]} ${spinnerClassName}`}
                aria-hidden="true"
            ></div>
        </div>
    );
}
