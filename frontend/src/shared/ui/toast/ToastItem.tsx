import { useEffect, useRef, useState } from 'react';
import {
    LuCheck,
    LuCircleAlert,
    LuInfo,
    LuTriangleAlert,
    LuX,
} from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../button/Button';
import {
    dismissToast,
    type ToastEntry,
    type ToastVariant,
} from './toast-store';

const VARIANT_STYLES: Record<
    ToastVariant,
    {
        icon: typeof LuCheck;
        containerClass: string;
        iconContainerClass: string;
        progressClass: string;
    }
> = {
    success: {
        icon: LuCheck,
        containerClass: 'border-emerald-200/60 dark:border-emerald-500/20',
        iconContainerClass:
            'bg-emerald-100 dark:bg-emerald-500/15 text-emerald-600 dark:text-emerald-400',
        progressClass: 'bg-emerald-500 dark:bg-emerald-400',
    },
    error: {
        icon: LuCircleAlert,
        containerClass: 'border-red-200/60 dark:border-red-500/20',
        iconContainerClass:
            'bg-red-100 dark:bg-red-500/15 text-red-600 dark:text-red-400',
        progressClass: 'bg-red-500 dark:bg-red-400',
    },
    warning: {
        icon: LuTriangleAlert,
        containerClass: 'border-amber-200/60 dark:border-amber-500/20',
        iconContainerClass:
            'bg-amber-100 dark:bg-amber-500/15 text-amber-600 dark:text-amber-400',
        progressClass: 'bg-amber-500 dark:bg-amber-400',
    },
    info: {
        icon: LuInfo,
        containerClass: 'border-primary-200/60 dark:border-primary-500/20',
        iconContainerClass:
            'bg-primary-100 dark:bg-primary-500/15 text-primary-600 dark:text-primary-400',
        progressClass: 'bg-primary-500 dark:bg-primary-400',
    },
};

const EXIT_DURATION_MS = 300;

export function ToastItem({ toast }: { toast: ToastEntry }) {
    const [phase, setPhase] = useState<'enter' | 'visible' | 'exit'>('enter');
    const timerRef = useRef<number | null>(null);
    const wrapperRef = useRef<HTMLDivElement | null>(null);
    const [measuredHeight, setMeasuredHeight] = useState<number | null>(null);

    // Measure natural height on mount for animated collapse
    useEffect(() => {
        if (wrapperRef.current) {
            setMeasuredHeight(wrapperRef.current.scrollHeight);
        }
    }, []);

    // Enter animation: mount as 'enter', flip to 'visible' next frame
    useEffect(() => {
        const frameId = requestAnimationFrame(() => {
            setPhase('visible');
        });
        return () => cancelAnimationFrame(frameId);
    }, []);

    // Auto-dismiss timer
    useEffect(() => {
        timerRef.current = window.setTimeout(() => {
            setPhase('exit');
        }, toast.durationMs);
        return () => {
            if (timerRef.current !== null) {
                window.clearTimeout(timerRef.current);
            }
        };
    }, [toast.durationMs]);

    // Remove from store after exit animation completes
    useEffect(() => {
        if (phase !== 'exit') return;
        const timeoutId = window.setTimeout(() => {
            dismissToast(toast.id);
        }, EXIT_DURATION_MS);
        return () => window.clearTimeout(timeoutId);
    }, [phase, toast.id]);

    const handleDismiss = () => {
        if (timerRef.current !== null) {
            window.clearTimeout(timerRef.current);
        }
        setPhase('exit');
    };

    const style = VARIANT_STYLES[toast.variant];
    const Icon = style.icon;

    const isVisible = phase === 'visible';
    const isExit = phase === 'exit';

    // Mobile: top-right, slides down; Desktop: bottom-right, slides up
    const enterClass = '-translate-y-3 sm:translate-y-3 opacity-0';
    const exitClass = '-translate-y-2 sm:translate-y-2 opacity-0 scale-95';

    return (
        <div
            ref={wrapperRef}
            style={{
                maxHeight: isExit ? 0 : (measuredHeight ?? 200),
                transition: `max-height ${EXIT_DURATION_MS}ms ease-out, margin ${EXIT_DURATION_MS}ms ease-out`,
                marginBottom: isExit ? -8 : 0, // collapse the flex gap
            }}
            className="overflow-hidden"
        >
            <div
                role="alert"
                className={`
                    relative overflow-hidden
                    w-full max-w-sm
                    bg-white/95 dark:bg-dark-900/90
                    backdrop-blur-sm
                    border ${style.containerClass}
                    rounded-xl shadow-lg dark:shadow-dark-950/40
                    transition-all duration-300 ease-out
                    ${isVisible ? 'translate-y-0 opacity-100' : ''}
                    ${!isVisible && !isExit ? enterClass : ''}
                    ${isExit ? exitClass : ''}
                `}
            >
                <div className="flex items-start gap-3 px-4 py-3">
                    <div
                        className={`shrink-0 mt-0.5 w-7 h-7 rounded-lg flex items-center justify-center ${style.iconContainerClass}`}
                    >
                        <Icon className="w-4 h-4" aria-hidden />
                    </div>
                    <p className="flex-1 text-sm font-medium text-gray-800 dark:text-dark-100 leading-snug pt-1">
                        {toast.message}
                    </p>
                    <Button
                        variant="ghost"
                        size="xs"
                        icon={LuX}
                        label={translation.get('toast-dismiss-label')}
                        onClick={handleDismiss}
                        className="shrink-0 mt-0.5 p-1 h-auto"
                    />
                </div>

                {/* Progress bar */}
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-gray-100 dark:bg-dark-700/50">
                    <div
                        className={`h-full ${style.progressClass} rounded-full`}
                        style={{
                            transition: isVisible
                                ? `width ${toast.durationMs}ms linear`
                                : undefined,
                            width: isVisible ? '0%' : '100%',
                        }}
                    />
                </div>
            </div>
        </div>
    );
}
