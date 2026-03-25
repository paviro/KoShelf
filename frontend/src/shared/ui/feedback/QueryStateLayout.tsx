import type { ReactNode } from 'react';

import { LoadingSpinner } from './LoadingSpinner';
import { PageErrorState } from './PageErrorState';

type QueryStateLayoutProps = {
    isError: boolean;
    error?: unknown;
    onRetry?: () => void;
    showBlockingSpinner: boolean;
    showOverlaySpinner: boolean;
    hasData: boolean;
    srLabel: string;
    errorChildren?: ReactNode;
    renderContent: () => ReactNode;
    blockingSpinnerClassName?: string;
    overlayClassName?: string;
    wrapperClassName?: string;
};

export function QueryStateLayout({
    isError,
    error,
    onRetry,
    showBlockingSpinner,
    showOverlaySpinner,
    hasData,
    srLabel,
    errorChildren,
    renderContent,
    blockingSpinnerClassName = 'page-centered-state',
    overlayClassName = 'absolute inset-0 z-20 flex items-center justify-center rounded-lg bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]',
    wrapperClassName = 'relative space-y-6 md:space-y-8',
}: QueryStateLayoutProps) {
    return (
        <>
            {!isError && showBlockingSpinner && (
                <section className={blockingSpinnerClassName}>
                    <LoadingSpinner size="lg" srLabel={srLabel} />
                </section>
            )}

            {isError && (
                <PageErrorState error={error} onRetry={onRetry}>
                    {errorChildren}
                </PageErrorState>
            )}

            {!isError && hasData && (
                <div className={wrapperClassName}>
                    {showOverlaySpinner && (
                        <div className={overlayClassName}>
                            <LoadingSpinner size="md" srLabel={srLabel} />
                        </div>
                    )}
                    {renderContent()}
                </div>
            )}
        </>
    );
}
