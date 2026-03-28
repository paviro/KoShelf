import { Fragment } from 'react';

import { translation } from '../../../shared/i18n';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';

type RecapDurationValueProps = {
    hours: number;
    minutes: number;
    days?: number;
    responsiveLabels?: boolean;
    valueClassName?: string;
    unitClassName?: string;
};

type Segment = {
    amount: number;
    fullLabel: string;
    shortLabel: string;
};

export function RecapDurationValue({
    hours,
    minutes,
    days,
    responsiveLabels = false,
    valueClassName = 'text-xl font-bold text-gray-900 dark:text-white leading-none',
    unitClassName = 'text-xs font-medium text-gray-500 dark:text-gray-400',
}: RecapDurationValueProps) {
    const segments: Segment[] = [];

    if (responsiveLabels) {
        if (days != null && days > 0) {
            segments.push({
                amount: days,
                fullLabel: translation.get('days_label', days),
                shortLabel: translation.get('units.d'),
            });
        }
        segments.push({
            amount: hours,
            fullLabel: translation.get('hours_label', hours),
            shortLabel: translation.get('units.h'),
        });
    } else {
        if (hours > 0) {
            segments.push({
                amount: hours,
                fullLabel: translation.get('hours_label', hours),
                shortLabel: translation.get('units.h'),
            });
        }
        if (hours === 0 || minutes > 0) {
            segments.push({
                amount: minutes,
                fullLabel: translation.get('minutes_label', minutes),
                shortLabel: translation.get('units.m'),
            });
        }
    }

    const hasTwoValues = segments.length > 1;

    return (
        <div className="flex items-baseline gap-1">
            {segments.map((seg, index) => (
                <Fragment key={index}>
                    <span className={valueClassName}>
                        {formatNumber(seg.amount)}
                    </span>
                    <span className={unitClassName}>
                        {responsiveLabels ? (
                            <>
                                <span className="hidden md:inline">
                                    {seg.fullLabel}
                                </span>
                                <span className="md:hidden">
                                    {seg.shortLabel}
                                </span>
                            </>
                        ) : hasTwoValues ? (
                            seg.shortLabel
                        ) : (
                            seg.fullLabel
                        )}
                    </span>
                </Fragment>
            ))}
        </div>
    );
}
