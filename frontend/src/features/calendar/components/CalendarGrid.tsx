import { createCalendar, DayGrid, destroyCalendar } from '@event-calendar/core';
import type { Calendar } from '@event-calendar/core';
import { useCallback, useEffect, useMemo, useRef } from 'react';

import { translation } from '../../../shared/i18n';
import type {
    CalendarEventResponse,
    CalendarItemResponse,
} from '../api/calendar-data';

type CalendarGridProps = {
    locale: string;
    displayedMonth: Date;
    events: CalendarEventResponse[];
    items: Record<string, CalendarItemResponse>;
    onDisplayedMonthChange: (date: Date) => void;
    onEventSelect: (event: CalendarEventResponse) => void;
};

type EventExtendedProps = {
    rawEvent: CalendarEventResponse;
};

const EVENT_COLOR_PALETTE = [
    '#3B82F6',
    '#10B981',
    '#F59E0B',
    '#EF4444',
    '#8B5CF6',
    '#06B6D4',
    '#84CC16',
    '#F97316',
    '#EC4899',
    '#6366F1',
];

function fallbackColorForEvent(
    event: CalendarEventResponse,
    title: string,
): string {
    let hash = 0;
    const token = `${title}${event.item_id}`;

    for (let index = 0; index < token.length; index += 1) {
        hash = (hash << 5) - hash + token.charCodeAt(index);
        hash |= 0;
    }

    return EVENT_COLOR_PALETTE[Math.abs(hash) % EVENT_COLOR_PALETTE.length];
}

function normalizeToMonth(date: Date): Date {
    return new Date(date.getFullYear(), date.getMonth(), 1, 12, 0, 0, 0);
}

export function CalendarGrid({
    locale,
    displayedMonth,
    events,
    items,
    onDisplayedMonthChange,
    onEventSelect,
}: CalendarGridProps) {
    const containerRef = useRef<HTMLElement | null>(null);
    const calendarRef = useRef<Calendar | null>(null);
    const scrollTimeoutRef = useRef<number | null>(null);
    const optionRefs = useRef<{
        locale: string;
        displayedMonth: Date;
        mappedEvents: Calendar.EventInput[];
    }>({ locale, displayedMonth, mappedEvents: [] });

    const scrollCurrentDayIntoView = useCallback(() => {
        const calendarContainer = containerRef.current;
        const todayCell =
            calendarContainer?.querySelector<HTMLElement>('.ec-day.ec-today');

        if (!calendarContainer || !todayCell) {
            return;
        }

        const maxScrollLeft =
            calendarContainer.scrollWidth - calendarContainer.clientWidth;
        if (maxScrollLeft <= 0) {
            return;
        }

        const containerRect = calendarContainer.getBoundingClientRect();
        const todayRect = todayCell.getBoundingClientRect();
        const todayCenterRelativeToContainer =
            todayRect.left -
            containerRect.left +
            todayRect.width / 2 +
            calendarContainer.scrollLeft;
        const desiredScrollLeft =
            todayCenterRelativeToContainer - calendarContainer.clientWidth / 2;
        const clampedScrollLeft = Math.max(
            0,
            Math.min(desiredScrollLeft, maxScrollLeft),
        );

        if (Math.abs(clampedScrollLeft - calendarContainer.scrollLeft) > 1) {
            calendarContainer.scrollTo({
                left: clampedScrollLeft,
                behavior: 'auto',
            });
        }
    }, []);

    const mappedEvents = useMemo<Calendar.EventInput[]>(
        () =>
            events.map((event) => {
                const item = items[event.item_id];
                const title = item?.title ?? translation.get('unknown-book');

                return {
                    id: `${event.item_id}-${event.start}-${event.end ?? event.start}`,
                    title,
                    start: event.start,
                    end: event.end || event.start,
                    allDay: true,
                    backgroundColor:
                        item?.color ?? fallbackColorForEvent(event, title),
                    textColor: '#ffffff',
                    extendedProps: {
                        rawEvent: event,
                    },
                };
            }),
        [events, items],
    );

    useEffect(() => {
        optionRefs.current = { locale, displayedMonth, mappedEvents };
    }, [locale, displayedMonth, mappedEvents]);

    const handleDatesSet = useCallback(
        (info: Calendar.DatesSetInfo) => {
            onDisplayedMonthChange(normalizeToMonth(info.view.currentStart));

            if (scrollTimeoutRef.current !== null) {
                window.clearTimeout(scrollTimeoutRef.current);
            }

            scrollTimeoutRef.current = window.setTimeout(() => {
                scrollCurrentDayIntoView();
                scrollTimeoutRef.current = null;
            }, 100);
        },
        [onDisplayedMonthChange, scrollCurrentDayIntoView],
    );

    useEffect(() => {
        return () => {
            if (scrollTimeoutRef.current !== null) {
                window.clearTimeout(scrollTimeoutRef.current);
            }
        };
    }, []);

    const handleEventClick = useCallback(
        (info: Calendar.EventClickInfo) => {
            const payload = info.event.extendedProps as EventExtendedProps;
            if (payload?.rawEvent) {
                onEventSelect(payload.rawEvent);
            }
        },
        [onEventSelect],
    );

    useEffect(() => {
        if (!containerRef.current) {
            return;
        }

        const opts = optionRefs.current;
        const instance = createCalendar(containerRef.current, [DayGrid], {
            view: 'dayGridMonth',
            height: 'auto',
            locale: opts.locale,
            date: opts.displayedMonth,
            firstDay: 1,
            displayEventEnd: false,
            editable: false,
            eventStartEditable: false,
            eventDurationEditable: false,
            events: opts.mappedEvents,
            eventClick: handleEventClick,
            datesSet: handleDatesSet,
        });

        calendarRef.current = instance;

        return () => {
            void destroyCalendar(instance);
            calendarRef.current = null;
        };
    }, [handleDatesSet, handleEventClick]);

    useEffect(() => {
        calendarRef.current?.setOption('events', mappedEvents);
    }, [mappedEvents]);

    useEffect(() => {
        calendarRef.current?.setOption('date', displayedMonth);
    }, [displayedMonth]);

    useEffect(() => {
        calendarRef.current?.setOption('locale', locale);
    }, [locale]);

    return (
        <section
            ref={containerRef}
            className="calendar-container bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg shadow-sm"
        />
    );
}
