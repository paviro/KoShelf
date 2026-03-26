import type { PageActivityAnnotation } from '../../../shared/contracts';

export interface AggregatedPage {
    page: number;
    totalDuration: number;
    readCount: number;
}

/**
 * Build annotation lookup: page number → list of annotations on that page.
 */
export function buildAnnotationMap(
    annotations: PageActivityAnnotation[],
): Map<number, PageActivityAnnotation[]> {
    const map = new Map<number, PageActivityAnnotation[]>();
    for (const a of annotations) {
        const list = map.get(a.page);
        if (list) {
            list.push(a);
        } else {
            map.set(a.page, [a]);
        }
    }
    return map;
}

/**
 * Compute a percentile value from page durations.
 * Reduces the impact of outlier pages on the color scale.
 */
export function percentileDuration(
    pageData: Map<number, AggregatedPage>,
    percentile: number,
): number {
    const durations = [...pageData.values()]
        .map((p) => p.totalDuration)
        .filter((d) => d > 0)
        .sort((a, b) => a - b);
    if (durations.length === 0) return 0;
    const index = Math.ceil((percentile / 100) * durations.length) - 1;
    return durations[Math.max(0, index)];
}
