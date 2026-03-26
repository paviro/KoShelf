import { useMemo, useState } from 'react';
import { LuInfo } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';
import {
    FilterDropdown,
    type FilterDropdownOption,
} from '../../../shared/ui/selectors/FilterDropdown';
import type {
    ChapterEntry,
    LibraryCompletions,
} from '../../../shared/contracts';
import { PageActivityGrid } from '../components/PageActivityGrid';
import { usePageActivityQuery } from '../hooks/usePageActivityQuery';
import type { AggregatedPage } from '../lib/page-activity-data';
import { formatCompletionDateRange } from '../lib/library-detail-formatters';

type LibraryPageActivitySectionProps = {
    itemId: string;
    chapters: ChapterEntry[];
    completions: LibraryCompletions | null;
    visible: boolean;
    onToggle: () => void;
};

const ALL_READINGS_KEY = '__all__';

export function LibraryPageActivitySection({
    itemId,
    chapters,
    completions,
    visible,
    onToggle,
}: LibraryPageActivitySectionProps) {
    const [selectedCompletion, setSelectedCompletion] =
        useState(ALL_READINGS_KEY);

    const completionParam =
        selectedCompletion === ALL_READINGS_KEY
            ? undefined
            : selectedCompletion;
    const { data, isLoading } = usePageActivityQuery(
        itemId,
        visible,
        completionParam,
    );

    // Build completion selector options.
    const completionOptions = useMemo((): FilterDropdownOption<string>[] => {
        const options: FilterDropdownOption<string>[] = [
            {
                value: ALL_READINGS_KEY,
                label: translation.get('filter.all'),
            },
        ];
        if (completions?.entries) {
            for (const [i, c] of completions.entries.entries()) {
                options.push({
                    value: String(i),
                    label: (
                        <span>
                            {translation.get('page-activity.reading-number', {
                                number: i + 1,
                            })}{' '}
                            <span className="text-[0.8em] font-normal opacity-60">
                                (
                                {formatCompletionDateRange(
                                    c.start_date,
                                    c.end_date,
                                )}
                                )
                            </span>
                        </span>
                    ),
                });
            }
        }
        return options;
    }, [completions]);

    // Convert server-aggregated pages to the Map the grid expects.
    const pageData = useMemo(() => {
        if (!data) return new Map<number, AggregatedPage>();
        const map = new Map<number, AggregatedPage>();
        for (const p of data.pages) {
            map.set(p.page, {
                page: p.page,
                totalDuration: p.total_duration,
                readCount: p.read_count,
            });
        }
        return map;
    }, [data]);

    const hasData = data && data.total_pages > 0 && data.pages.length > 0;
    const hasCompletions =
        completions !== null && completions.entries.length > 0;

    // Animation seed changes when completion selection changes to re-trigger animation.
    const animationSeed = `${itemId}-${selectedCompletion}`;

    return (
        <CollapsibleSection
            sectionKey="page-activity"
            defaultVisible={false}
            accentClass="bg-linear-to-b from-indigo-400 to-indigo-600"
            title={translation.get('page-activity')}
            visible={visible}
            onToggle={onToggle}
            contentClassName="mb-8"
            controls={
                visible && hasCompletions ? (
                    <FilterDropdown
                        value={selectedCompletion}
                        options={completionOptions}
                        onChange={setSelectedCompletion}
                        ariaLabel={translation.get(
                            'page-activity.select-reading',
                        )}
                        panelClassName="w-64"
                    />
                ) : undefined
            }
        >
            <div className="relative">
                {isLoading && (
                    <div className="flex items-center justify-center py-12">
                        <LoadingSpinner
                            size="md"
                            srLabel="Loading page activity"
                        />
                    </div>
                )}

                {!isLoading && !hasData && (
                    <p className="text-sm text-gray-500 dark:text-dark-400 py-4">
                        {translation.get('page-activity.no-data')}
                    </p>
                )}

                {!isLoading && hasData && (
                    <PageActivityGrid
                        totalPages={data.total_pages}
                        pageData={pageData}
                        annotations={data.annotations}
                        chapters={chapters}
                        animationSeed={animationSeed}
                    />
                )}
            </div>

            {!isLoading && hasData && (
                <div className="mt-6 p-4 bg-primary-50 dark:bg-dark-850/30 rounded-lg border border-primary-200 dark:border-dark-700/50">
                    <div className="flex items-center text-sm font-medium text-gray-500 dark:text-dark-400">
                        <LuInfo
                            className="w-4 h-4 mr-2 shrink-0 text-primary-400"
                            aria-hidden="true"
                        />
                        <span>{translation.get('page-activity.info')}</span>
                    </div>
                </div>
            )}
        </CollapsibleSection>
    );
}
