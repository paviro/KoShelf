import { api, type ScopeValue } from '../../../shared/api';
import type {
    CompletionGroup,
    CompletionItem,
    CompletionsShareAssets,
    CompletionsSummary,
} from '../../../shared/contracts';

export type RecapScope = ScopeValue;

export type {
    CompletionGroup,
    CompletionItem,
    CompletionsShareAssets,
    CompletionsSummary,
};

export interface RecapIndexResponse {
    available_years: number[];
    latest_year?: number | null;
}

export interface RecapYearResponse {
    year: number;
    summary: CompletionsSummary | null;
    months: CompletionGroup[];
    items: CompletionItem[];
    share_assets: CompletionsShareAssets | null;
}

export async function loadRecapIndex(
    scope: RecapScope,
): Promise<RecapIndexResponse> {
    const data = await api.getAvailablePeriods('completions', 'year', scope);

    return {
        available_years: data.periods
            .map((p) => Number(p.key))
            .filter(Number.isFinite),
        latest_year: data.latest_key ? Number(data.latest_key) : null,
    };
}

export async function loadRecapYear(
    scope: RecapScope,
    year: number,
): Promise<RecapYearResponse> {
    const data = await api.getReadingCompletions(scope, {
        year,
        groupBy: 'month',
        include: 'summary,share_assets',
    });

    return {
        year,
        summary: data.summary ?? null,
        months: data.groups ?? [],
        items: data.items ?? [],
        share_assets: data.share_assets ?? null,
    };
}
