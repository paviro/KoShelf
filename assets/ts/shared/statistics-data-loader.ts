export interface DailyActivityEntry {
    date: string;
    read_time: number;
    pages_read: number;
}

export interface ActivityConfig {
    max_scale_seconds: number | null;
}

export interface YearlyActivityResponse {
    data: DailyActivityEntry[];
    config?: ActivityConfig;
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
            return {
                data: Array.isArray(jsonResponse.data) ? jsonResponse.data : [],
                config: jsonResponse.config,
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
