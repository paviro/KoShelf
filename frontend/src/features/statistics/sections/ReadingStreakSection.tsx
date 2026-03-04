import type { StatisticsIndexResponse, StatisticsYearResponse } from '../api/statistics-data';
import { translation } from '../../../shared/i18n';
import { formatStreakDateRange, type SectionName } from '../model/statistics-model';
import { HeatmapSection } from './HeatmapSection';
import { YearSelector } from '../../../shared/ui/selectors/YearSelector';
import { CollapsibleSection } from '../../../shared/ui/sections/CollapsibleSection';

type ReadingStreakSectionProps = {
    visible: boolean;
    onToggle: (sectionName: SectionName) => void;
    availableYears: number[];
    selectedYear: number | null;
    onSelectYear: (year: number) => void;
    yearData: StatisticsYearResponse | undefined;
    animationSeed: string;
    currentStreak: StatisticsIndexResponse['streaks']['current'];
    longestStreak: StatisticsIndexResponse['streaks']['longest'];
};

export function ReadingStreakSection({
    visible,
    onToggle,
    availableYears,
    selectedYear,
    onSelectYear,
    yearData,
    animationSeed,
    currentStreak,
    longestStreak,
}: ReadingStreakSectionProps) {
    return (
        <CollapsibleSection
            sectionKey="reading-streak"
            accentClass="bg-gradient-to-b from-green-400 to-green-600"
            title={translation.get('reading-streak')}
            visible={visible}
            onToggle={() => onToggle('reading-streak')}
            controls={
                <YearSelector
                    idPrefix="Year"
                    years={availableYears}
                    selectedYear={selectedYear}
                    onSelect={onSelectYear}
                    iconColorClass="text-gray-600 dark:text-gray-300 sm:text-green-400 sm:dark:text-green-400"
                    optionActiveClass="bg-dark-700 text-white"
                    mobileFallback="No data"
                />
            }
        >
            <div className="mb-8">
                <HeatmapSection
                    selectedYear={selectedYear}
                    yearData={yearData}
                    animationSeed={animationSeed}
                />

                <div className="w-full mt-3 sm:mt-3 md:mt-4">
                    <div className="flex flex-row w-full rounded-lg shadow-lg overflow-hidden gap-0">
                        <div className="flex-1 flex flex-col lg:flex-row items-center lg:items-center justify-center lg:justify-start bg-gradient-to-br from-primary-50 to-primary-100/80 dark:from-primary-900/40 dark:to-dark-900 border border-primary-200/80 dark:border-primary-900/50 rounded-l-lg border-r-0 p-3 md:p-4 lg:p-6 xl:p-6">
                            <div className="flex flex-col lg:flex-row items-center lg:items-center w-full lg:w-auto">
                                <div className="flex items-baseline justify-center lg:justify-start mb-2 lg:mb-0 lg:mr-2">
                                    <span className="text-2xl md:text-3xl lg:text-4xl xl:text-5xl font-extrabold text-primary-600 dark:text-white drop-shadow-sm leading-none">
                                        {currentStreak.days}
                                    </span>
                                    <span className="text-sm md:text-lg lg:text-xl xl:text-2xl font-bold text-primary-500 dark:text-primary-200 ml-1.5 md:ml-2 tracking-wider xl:tracking-widest uppercase">
                                        {translation.get('days_label', currentStreak.days)}
                                    </span>
                                </div>
                                <div className="flex items-center lg:ml-6">
                                    <div>
                                        <div className="text-xs md:text-sm lg:text-base xl:text-base font-bold text-center lg:text-left text-primary-600 dark:text-primary-300 tracking-wider xl:tracking-widest uppercase mb-0.5 md:mb-0.5">
                                            {translation.get('streak.current')}
                                        </div>
                                        <div className="text-xs md:text-xs lg:text-sm xl:text-sm text-center lg:text-left text-primary-500/90 dark:text-primary-200/70">
                                            {formatStreakDateRange(
                                                currentStreak.start_date,
                                                currentStreak.end_date,
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="flex-1 flex flex-col lg:flex-row items-center lg:items-center justify-center lg:justify-end bg-gradient-to-br from-gray-50 to-gray-100/80 dark:from-dark-800 dark:to-dark-900 border border-gray-200/80 dark:border-dark-700/50 rounded-r-lg border-l-0 p-3 md:p-4 lg:p-6 xl:p-6">
                            <div className="flex flex-col lg:flex-row items-center lg:items-center w-full lg:w-auto">
                                <div className="flex items-baseline justify-center lg:justify-end mb-2 lg:mb-0 lg:mr-2">
                                    <span className="text-2xl md:text-3xl lg:text-4xl xl:text-5xl font-extrabold text-gray-700 dark:text-white drop-shadow-sm leading-none">
                                        {longestStreak.days}
                                    </span>
                                    <span className="text-sm md:text-lg lg:text-xl xl:text-2xl font-bold text-gray-600 dark:text-gray-400 ml-1.5 md:ml-2 tracking-wider xl:tracking-widest uppercase">
                                        {translation.get('days_label', longestStreak.days)}
                                    </span>
                                </div>
                                <div className="flex items-center w-full lg:flex-1 lg:justify-end lg:ml-6">
                                    <div className="w-full text-center lg:text-right">
                                        <div className="text-xs md:text-sm lg:text-base xl:text-base font-bold text-center lg:text-right text-gray-700 dark:text-dark-200 tracking-wider xl:tracking-widest uppercase mb-0.5 md:mb-0.5">
                                            {translation.get('streak.longest')}
                                        </div>
                                        <div className="text-xs md:text-xs lg:text-sm xl:text-sm text-center lg:text-right text-gray-600/90 dark:text-dark-300">
                                            {formatStreakDateRange(
                                                longestStreak.start_date,
                                                longestStreak.end_date,
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </CollapsibleSection>
    );
}
