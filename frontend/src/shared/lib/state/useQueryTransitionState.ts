import { useEffect, useState } from 'react';

export const DEFAULT_QUERY_TRANSITION_SPINNER_DELAY_MS = 120;

type QueryTransitionStateOptions<TData> = {
    data: TData | undefined;
    enabled?: boolean;
    isLoading: boolean;
    isFetching: boolean;
    isPlaceholderData?: boolean;
    spinnerDelayMs?: number;
    clearOnDisabled?: boolean;
};

type QueryTransitionState<TData> = {
    displayData: TData | null;
    hasDisplayData: boolean;
    hasFreshData: boolean;
    isTransitioning: boolean;
    showBlockingSpinner: boolean;
    showOverlaySpinner: boolean;
};

export function useQueryTransitionState<TData>({
    data,
    enabled = true,
    isLoading,
    isFetching,
    isPlaceholderData = false,
    spinnerDelayMs = DEFAULT_QUERY_TRANSITION_SPINNER_DELAY_MS,
    clearOnDisabled = true,
}: QueryTransitionStateOptions<TData>): QueryTransitionState<TData> {
    const [delayedOverlayVisible, setDelayedOverlayVisible] = useState(false);

    const hasFreshData = enabled && data !== undefined && !isPlaceholderData;
    const displayData =
        !enabled && clearOnDisabled ? null : data === undefined ? null : data;

    const hasDisplayData = displayData !== null;
    const isTransitioning =
        enabled && hasDisplayData && isFetching && isPlaceholderData;

    useEffect(() => {
        if (!isTransitioning) {
            const resetTimeoutId = window.setTimeout(() => {
                setDelayedOverlayVisible(false);
            }, 0);
            return () => {
                window.clearTimeout(resetTimeoutId);
            };
        }

        if (delayedOverlayVisible) {
            return;
        }

        const timeoutId = window.setTimeout(() => {
            setDelayedOverlayVisible(true);
        }, spinnerDelayMs);

        return () => {
            window.clearTimeout(timeoutId);
        };
    }, [delayedOverlayVisible, isTransitioning, spinnerDelayMs]);

    const showBlockingSpinner =
        enabled &&
        !hasDisplayData &&
        (isLoading || isFetching) &&
        !hasFreshData;

    return {
        displayData,
        hasDisplayData,
        hasFreshData,
        isTransitioning,
        showBlockingSpinner,
        showOverlaySpinner: delayedOverlayVisible && isTransitioning,
    };
}
