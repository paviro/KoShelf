export interface DailyActivityEntry {
    date: string;
    read_time: number;
    pages_read: number;
}

export interface ActivityConfig {
    max_scale_seconds: number | null;
}

export interface YearlyActivitySummary {
    completed_count: number;
}

export interface YearlyActivityResponse {
    data: DailyActivityEntry[];
    config?: ActivityConfig;
    summary: YearlyActivitySummary;
}

const yearlyActivityCache = new Map<string, Promise<YearlyActivityResponse>>();

function makeCacheKey(basePath: string, year: number): string {
    return `${basePath}::${year}`;
}

export async function loadYearlyActivity(
    basePath: string,
    year: number,
): Promise<YearlyActivityResponse> {
    const key = makeCacheKey(basePath, year);
    let request = yearlyActivityCache.get(key);

    if (!request) {
        request = fetch(`${basePath}/daily_activity_${year}.json`).then(async (response) => {
            if (!response.ok) {
                throw new Error(`Failed to load activity data for ${year}`);
            }

            const jsonResponse = (await response.json()) as YearlyActivityResponse;
            const completedCount = jsonResponse.summary?.completed_count;

            return {
                data: Array.isArray(jsonResponse.data) ? jsonResponse.data : [],
                config: jsonResponse.config,
                summary: {
                    completed_count: typeof completedCount === 'number' ? completedCount : 0,
                },
            };
        });

        yearlyActivityCache.set(key, request);
    }

    try {
        return await request;
    } catch (error) {
        yearlyActivityCache.delete(key);
        throw error;
    }
}

export function clearYearlyActivityCache(): void {
    yearlyActivityCache.clear();
}
