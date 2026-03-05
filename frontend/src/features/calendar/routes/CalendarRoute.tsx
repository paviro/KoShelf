import { useQuery } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';

import '../../../styles/calendar.css';
import { CalendarEventModal } from '../components/CalendarEventModal';
import { CalendarGrid } from '../components/CalendarGrid';
import { CalendarHeader } from '../components/CalendarHeader';
import { CalendarMonthPickerModal } from '../components/CalendarMonthPickerModal';
import { CalendarYearPickerModal } from '../components/CalendarYearPickerModal';
import { CalendarMonthlyStatsSection } from '../sections/CalendarMonthlyStatsSection';
import {
    aggregateCalendarData,
    eventMatchesScope,
    isCurrentMonth,
    loadInitialCalendarViewState,
    monthKey,
    normalizeToMonthStart,
    parseMonthKey,
    persistCalendarViewState,
    resolveMonthlyStats,
    shiftMonth,
    shiftMonthKey,
} from '../model/calendar-model';
import {
    useCalendarMonthQuery,
    useCalendarMonthsQuery,
} from '../hooks/useCalendarQueries';
import { useQueryTransitionState } from '../../../shared/lib/state/useQueryTransitionState';
import type { CalendarEventResponse } from '../api/calendar-data';
import type { ScopeValue } from '../../../shared/api';
import { api } from '../../../shared/api';
import type { SiteResponse } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { LoadingSpinner } from '../../../shared/ui/feedback/LoadingSpinner';
import { PageErrorState } from '../../../shared/ui/feedback/PageErrorState';
import { MODAL_TRANSITION_DURATION_MS } from '../../../shared/ui/modal/ModalShell';

const FALLBACK_LOCALE = 'en-US';

function safeFormatDateLabel(
    date: Date,
    locale: string,
    options: Intl.DateTimeFormatOptions,
): string {
    try {
        return new Intl.DateTimeFormat(
            locale || FALLBACK_LOCALE,
            options,
        ).format(date);
    } catch {
        return new Intl.DateTimeFormat(FALLBACK_LOCALE, options).format(date);
    }
}

export function CalendarRoute() {
    const [initialCalendarView] = useState(() =>
        loadInitialCalendarViewState(),
    );
    const [scope, setScope] = useState<ScopeValue>(
        () => initialCalendarView.scope,
    );
    const [persistMonthSelection, setPersistMonthSelection] = useState<boolean>(
        () => initialCalendarView.monthKey !== null,
    );
    const [displayedMonth, setDisplayedMonth] = useState<Date>(() =>
        initialCalendarView.monthKey
            ? parseMonthKey(initialCalendarView.monthKey)
            : normalizeToMonthStart(new Date()),
    );
    const [monthPickerOpen, setMonthPickerOpen] = useState(false);
    const [yearPickerOpen, setYearPickerOpen] = useState(false);
    const [yearPickerStartYear, setYearPickerStartYear] = useState(
        new Date().getFullYear() - 4,
    );
    const [selectedEvent, setSelectedEvent] =
        useState<CalendarEventResponse | null>(null);
    const [isEventModalOpen, setIsEventModalOpen] = useState(false);

    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
    });

    const displayedMonthKey = monthKey(displayedMonth);
    const previousMonthKey = shiftMonthKey(displayedMonthKey, -1);
    const nextMonthKey = shiftMonthKey(displayedMonthKey, 1);

    const monthsQuery = useCalendarMonthsQuery();
    const availableMonths = useMemo(
        () => monthsQuery.data?.months ?? [],
        [monthsQuery.data?.months],
    );
    const availableMonthSet = useMemo(
        () => new Set(availableMonths),
        [availableMonths],
    );
    const canStartMonthQueries = monthsQuery.isSuccess || monthsQuery.isError;

    const shouldFetchMonth = useCallback(
        (targetMonthKey: string) => {
            if (!canStartMonthQueries) {
                return false;
            }

            if (!monthsQuery.isSuccess) {
                // If the month index fails, avoid blocking the calendar on it.
                return true;
            }

            if (availableMonthSet.size === 0) {
                return false;
            }

            return availableMonthSet.has(targetMonthKey);
        },
        [availableMonthSet, canStartMonthQueries, monthsQuery.isSuccess],
    );

    const previousMonthEnabled = shouldFetchMonth(previousMonthKey);
    const currentMonthEnabled = shouldFetchMonth(displayedMonthKey);
    const nextMonthEnabled = shouldFetchMonth(nextMonthKey);

    const previousMonthQuery = useCalendarMonthQuery(
        previousMonthKey,
        previousMonthEnabled,
    );
    const currentMonthQuery = useCalendarMonthQuery(
        displayedMonthKey,
        currentMonthEnabled,
    );
    const currentMonthTransition = useQueryTransitionState({
        data: currentMonthQuery.data,
        enabled: currentMonthEnabled,
        isLoading: currentMonthQuery.isLoading,
        isFetching: currentMonthQuery.isFetching,
        isPlaceholderData: currentMonthQuery.isPlaceholderData,
    });
    const nextMonthQuery = useCalendarMonthQuery(nextMonthKey, nextMonthEnabled);

    useEffect(() => {
        persistCalendarViewState({
            scope,
            monthKey: persistMonthSelection ? displayedMonthKey : null,
        });
    }, [displayedMonthKey, persistMonthSelection, scope]);

    useEffect(() => {
        if (siteQuery.data?.title) {
            document.title = `${translation.get('calendar')} - ${siteQuery.data.title}`;
        }
    }, [siteQuery.data]);

    const locale = translation.getLanguage() || 'en-US';

    const monthLabel = useMemo(
        () => safeFormatDateLabel(displayedMonth, locale, { month: 'long' }),
        [displayedMonth, locale],
    );
    const yearLabel = useMemo(
        () => safeFormatDateLabel(displayedMonth, locale, { year: 'numeric' }),
        [displayedMonth, locale],
    );

    const currentMonthData = currentMonthTransition.displayData;
    const mergedCalendarData = useMemo(
        () =>
            aggregateCalendarData(
                [
                    previousMonthQuery.data,
                    currentMonthData ?? undefined,
                    nextMonthQuery.data,
                ].filter(
                    (monthData): monthData is NonNullable<typeof monthData> =>
                        Boolean(monthData),
                ),
            ),
        [currentMonthData, nextMonthQuery.data, previousMonthQuery.data],
    );

    const filteredEvents = useMemo(
        () =>
            mergedCalendarData.events.filter((event) =>
                eventMatchesScope(event, mergedCalendarData.items, scope),
            ),
        [mergedCalendarData.events, mergedCalendarData.items, scope],
    );

    const selectedItem = selectedEvent
        ? (mergedCalendarData.items[selectedEvent.item_id] ?? null)
        : null;
    const monthlyStats = resolveMonthlyStats(
        currentMonthData ?? undefined,
        scope,
    );

    const showTypeFilter = Boolean(
        siteQuery.data?.capabilities.has_books &&
        siteQuery.data?.capabilities.has_comics,
    );

    const handleDisplayedMonthChange = useCallback((nextDate: Date) => {
        setPersistMonthSelection(true);
        setDisplayedMonth((currentDate) => {
            const nextMonthKey = monthKey(nextDate);
            if (monthKey(currentDate) === nextMonthKey) {
                return currentDate;
            }

            return normalizeToMonthStart(nextDate);
        });
    }, []);

    const handleEventSelect = useCallback((event: CalendarEventResponse) => {
        setSelectedEvent(event);
        setIsEventModalOpen(true);
    }, []);

    const handlePreviousMonth = useCallback(() => {
        setPersistMonthSelection(true);
        setDisplayedMonth((currentDate) => shiftMonth(currentDate, -1));
    }, []);

    const handleNextMonth = useCallback(() => {
        setPersistMonthSelection(true);
        setDisplayedMonth((currentDate) => shiftMonth(currentDate, 1));
    }, []);

    const handleToday = useCallback(() => {
        setPersistMonthSelection(false);
        setDisplayedMonth(normalizeToMonthStart(new Date()));
    }, []);

    useEffect(() => {
        if (isEventModalOpen || !selectedEvent) {
            return;
        }

        const timerId = window.setTimeout(() => {
            setSelectedEvent(null);
        }, MODAL_TRANSITION_DURATION_MS);

        return () => {
            window.clearTimeout(timerId);
        };
    }, [isEventModalOpen, selectedEvent]);

    const initialLoading =
        !canStartMonthQueries ||
        (currentMonthEnabled && currentMonthTransition.showBlockingSpinner);

    return (
        <>
            <div className="min-h-screen flex flex-col">
                <CalendarHeader
                    monthLabel={monthLabel}
                    yearLabel={yearLabel}
                    scope={scope}
                    showTypeFilter={showTypeFilter}
                    onScopeChange={setScope}
                    onPreviousMonth={handlePreviousMonth}
                    onNextMonth={handleNextMonth}
                    onToday={handleToday}
                    onOpenMonthPicker={() => setMonthPickerOpen(true)}
                    onOpenYearPicker={() => {
                        setYearPickerStartYear(
                            displayedMonth.getFullYear() - 4,
                        );
                        setYearPickerOpen(true);
                    }}
                    todayDisabled={isCurrentMonth(displayedMonth)}
                />

                <main className="relative flex-1 flex flex-col pt-[88px] md:pt-24 pb-28 lg:pb-4 px-4 md:px-6 space-y-4">
                    {initialLoading && (
                        <section className="flex-1 flex items-center justify-center">
                            <LoadingSpinner
                                size="lg"
                                srLabel="Loading calendar"
                            />
                        </section>
                    )}

                    {currentMonthQuery.isError && currentMonthEnabled && (
                        <PageErrorState
                            error={currentMonthQuery.error}
                            onRetry={() => currentMonthQuery.refetch()}
                        />
                    )}

                    {!currentMonthQuery.isError && !initialLoading && (
                        <>
                            {currentMonthTransition.showOverlaySpinner && (
                                <div className="absolute inset-0 z-20 flex items-center justify-center bg-white/70 dark:bg-dark-900/70 backdrop-blur-[1px]">
                                    <LoadingSpinner
                                        size="md"
                                        srLabel="Loading calendar"
                                    />
                                </div>
                            )}

                            <CalendarMonthlyStatsSection
                                stats={monthlyStats}
                                scope={scope}
                            />

                            <CalendarGrid
                                locale={locale}
                                displayedMonth={displayedMonth}
                                events={filteredEvents}
                                items={mergedCalendarData.items}
                                onDisplayedMonthChange={
                                    handleDisplayedMonthChange
                                }
                                onEventSelect={handleEventSelect}
                            />
                        </>
                    )}
                </main>
            </div>

            <CalendarMonthPickerModal
                open={monthPickerOpen}
                year={displayedMonth.getFullYear()}
                selectedMonthIndex={displayedMonth.getMonth()}
                locale={locale}
                onClose={() => setMonthPickerOpen(false)}
                onSelectMonth={(monthIndex) => {
                    setPersistMonthSelection(true);
                    setDisplayedMonth(
                        (currentDate) =>
                            new Date(
                                currentDate.getFullYear(),
                                monthIndex,
                                1,
                                12,
                                0,
                                0,
                                0,
                            ),
                    );
                }}
            />

            <CalendarYearPickerModal
                open={yearPickerOpen}
                selectedYear={displayedMonth.getFullYear()}
                rangeStartYear={yearPickerStartYear}
                onClose={() => setYearPickerOpen(false)}
                onPreviousRange={() =>
                    setYearPickerStartYear((current) => current - 9)
                }
                onNextRange={() =>
                    setYearPickerStartYear((current) => current + 9)
                }
                onSelectYear={(year) => {
                    setPersistMonthSelection(true);
                    setDisplayedMonth(
                        (currentDate) =>
                            new Date(
                                year,
                                currentDate.getMonth(),
                                1,
                                12,
                                0,
                                0,
                                0,
                            ),
                    );
                }}
            />

            <CalendarEventModal
                open={isEventModalOpen}
                event={selectedEvent}
                item={selectedItem}
                onClose={() => setIsEventModalOpen(false)}
            />
        </>
    );
}
