import { useState } from 'react';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import { LuClock3, LuEye, LuFileText } from 'react-icons/lu';
import { useLocation, useNavigate } from 'react-router';

import {
    buildRoutePath,
    detailRouteIdForContentType,
} from '../../../app/routes/route-registry';
import { translation } from '../../../shared/i18n';
import { createDetailReturnState } from '../../../shared/lib/navigation/detail-return-state';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { Button } from '../../../shared/ui/button/Button';
import { CloseButton } from '../../../shared/ui/button/CloseButton';
import { MetricCard } from '../../../shared/ui/cards/MetricCard';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';
import type {
    CalendarEventResponse,
    CalendarItemResponse,
} from '../api/calendar-data';
import { formatDuration } from '../../../shared/lib/intl/formatDuration';

type CalendarEventModalProps = {
    open: boolean;
    event: CalendarEventResponse | null;
    item: CalendarItemResponse | null;
    onClose: () => void;
};

export function CalendarEventModal({
    open,
    event,
    item,
    onClose,
}: CalendarEventModalProps) {
    const [coverFailed, setCoverFailed] = useState(false);
    const coverKey = `${event?.item_ref}\0${item?.item_cover}`;
    const [prevCoverKey, setPrevCoverKey] = useState(coverKey);
    const location = useLocation();
    const navigate = useNavigate();
    const detailReturnState = createDetailReturnState(
        location.pathname,
        location.search,
    );

    if (prevCoverKey !== coverKey) {
        setPrevCoverKey(coverKey);
        setCoverFailed(false);
    }

    if (!event) {
        return null;
    }

    const title = item?.title ?? translation.get('unknown-book');
    const authors = item?.authors.length
        ? item.authors.join(', ')
        : translation.get('unknown-author');
    const coverUrl = item?.item_cover?.trim() ? item.item_cover : null;
    const detailItemId = item?.item_id?.trim() ?? '';
    const detailPath =
        item && detailItemId
            ? buildRoutePath(detailRouteIdForContentType(item.content_type), {
                  id: detailItemId,
              })
            : null;
    const canViewDetails = Boolean(detailPath);

    return (
        <ModalShell
            open={open}
            onClose={onClose}
            cardClassName="max-w-md w-full max-h-[85vh] overflow-y-auto bg-white/95 dark:bg-dark-900/70 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            showCloseButton={false}
        >
            <div className="flex items-center justify-between p-4 border-b border-gray-200/70 dark:border-dark-700/50">
                <div className="flex items-center gap-3 min-w-0 flex-1 pr-3">
                    <div className="w-8 h-8 bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600 rounded-lg flex items-center justify-center shrink-0">
                        <HiOutlineBookOpen
                            className="w-4 h-4 text-primary-600 dark:text-white"
                            aria-hidden="true"
                        />
                    </div>
                    <div className="min-w-0">
                        <h3 className="text-base font-bold text-gray-900 dark:text-white truncate leading-tight">
                            {title}
                        </h3>
                        <p className="text-xs font-medium text-gray-500 dark:text-dark-300 truncate">
                            {translation.get('by')}{' '}
                            <span className="text-gray-700 dark:text-dark-200">
                                {authors}
                            </span>
                        </p>
                    </div>
                </div>
                <CloseButton
                    onClick={onClose}
                    className="w-8 h-8 rounded-lg shrink-0"
                />
            </div>

            <div className="p-4 space-y-4">
                <div className="flex gap-4">
                    <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-xl overflow-hidden shadow-lg dark:shadow-none w-[160px] shrink-0 self-start">
                        {coverUrl && !coverFailed ? (
                            <img
                                src={coverUrl}
                                alt={title}
                                className="w-full h-auto"
                                onError={() => setCoverFailed(true)}
                            />
                        ) : (
                            <div className="aspect-2/3 w-full flex items-center justify-center bg-linear-to-br from-primary-500/10 to-primary-600/10">
                                <HiOutlineBookOpen
                                    className="w-12 h-12 text-gray-400 dark:text-dark-400"
                                    aria-hidden="true"
                                />
                            </div>
                        )}
                    </div>

                    <div className="flex flex-col justify-center gap-3 flex-1 min-w-0">
                        <MetricCard
                            size="sm"
                            icon={LuClock3}
                            iconContainerClassName="bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600"
                            iconClassName="text-green-600 dark:text-white"
                            value={formatDuration(event.reading_time_sec, {
                                includeSeconds: true,
                            })}
                            label={translation.get('reading-time')}
                        />
                        <MetricCard
                            size="sm"
                            icon={LuFileText}
                            iconContainerClassName="bg-blue-500/20 dark:bg-linear-to-br dark:from-blue-500 dark:to-blue-600"
                            iconClassName="text-blue-600 dark:text-white"
                            value={formatNumber(event.pages_read)}
                            label={translation.get('pages-read')}
                        />
                    </div>
                </div>

                {canViewDetails && (
                    <Button
                        fullWidth
                        icon={LuEye}
                        onClick={() => {
                            onClose();
                            if (detailPath) {
                                navigate(detailPath, {
                                    state: detailReturnState,
                                });
                            }
                        }}
                    >
                        {translation.get('view-details')}
                    </Button>
                )}
            </div>
        </ModalShell>
    );
}
