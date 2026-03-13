import { useState } from 'react';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import { LuClock3, LuEye, LuFileText, LuUser, LuX } from 'react-icons/lu';
import { useLocation, useNavigate } from 'react-router-dom';

import {
    buildRoutePath,
    detailRouteIdForContentType,
} from '../../../app/routes/route-registry';
import { translation } from '../../../shared/i18n';
import { createDetailReturnState } from '../../../shared/lib/navigation/detail-return-state';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';
import type {
    CalendarEventResponse,
    CalendarItemResponse,
} from '../api/calendar-data';
import { formatDuration } from '../model/calendar-model';

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
            cardClassName="max-w-lg max-h-[85vh] overflow-y-auto bg-white/95 dark:bg-dark-900/70 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl"
            showCloseButton={false}
        >
            <div className="relative p-6 pb-4">
                <button
                    type="button"
                    className="absolute top-4 right-4 text-gray-400 dark:text-dark-400 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-dark-700/50 rounded-full p-2 transition-all duration-200 z-20"
                    title={translation.get('close.aria-label')}
                    aria-label={translation.get('close.aria-label')}
                    onClick={onClose}
                >
                    <LuX className="w-5 h-5" aria-hidden="true" />
                </button>
            </div>

            <div className="@container px-6 pb-6">
                <div className="flex flex-col @[280px]:flex-row items-center @[280px]:items-start gap-4">
                    <div className="flex-shrink-0">
                        {coverUrl && !coverFailed ? (
                            <img
                                src={coverUrl}
                                alt={title}
                                className="w-20 aspect-square rounded-lg overflow-hidden shadow-lg object-cover border border-primary-500/30"
                                onError={() => setCoverFailed(true)}
                            />
                        ) : (
                            <div className="w-20 aspect-square rounded-lg shadow-lg bg-gradient-to-br from-primary-500/20 to-primary-600/20 border border-primary-500/30 flex items-center justify-center">
                                <HiOutlineBookOpen
                                    className="w-8 h-8 text-primary-400/70"
                                    aria-hidden="true"
                                />
                            </div>
                        )}
                    </div>

                    <div className="flex-1 min-w-0 @[280px]:pr-8 text-center @[280px]:text-left">
                        <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-2 leading-tight">
                            {title}
                        </h3>
                        <div className="flex items-center justify-center @[280px]:justify-start text-sm text-gray-500 dark:text-dark-300 mb-1">
                            <LuUser className="w-4 h-4 mr-2 text-primary-400 flex-shrink-0" />
                            <span className="truncate">
                                {translation.get('by')}{' '}
                                <span className="text-gray-900 dark:text-white font-medium">
                                    {authors}
                                </span>
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <div className="px-6 pb-6">
                <div className="grid grid-cols-2 gap-3">
                    <div className="@container bg-green-50 dark:bg-dark-800/60 rounded-xl p-4 border border-green-200/70 dark:border-dark-700/50">
                        <div className="flex flex-col @[120px]:flex-row items-center justify-center @[120px]:justify-start @[120px]:items-center gap-2 @[120px]:gap-3 h-full">
                            <div className="w-8 h-8 bg-green-500/20 dark:bg-gradient-to-br dark:from-green-500 dark:to-green-600 rounded-lg flex items-center justify-center flex-shrink-0">
                                <LuClock3 className="w-4 h-4 text-green-600 dark:text-white" />
                            </div>
                            <div className="text-center @[120px]:text-left">
                                <div className="text-xs text-gray-500 dark:text-dark-400 uppercase tracking-wide font-medium">
                                    {translation.get('reading-time')}
                                </div>
                                <div className="text-gray-900 dark:text-white font-semibold">
                                    {formatDuration(event.reading_time_sec)}
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="@container bg-blue-50 dark:bg-dark-800/60 rounded-xl p-4 border border-blue-200/70 dark:border-dark-700/50">
                        <div className="flex flex-col @[120px]:flex-row items-center justify-center @[120px]:justify-start @[120px]:items-center gap-2 @[120px]:gap-3 h-full">
                            <div className="w-8 h-8 bg-blue-500/20 dark:bg-gradient-to-br dark:from-blue-500 dark:to-blue-600 rounded-lg flex items-center justify-center flex-shrink-0">
                                <LuFileText className="w-4 h-4 text-blue-600 dark:text-white" />
                            </div>
                            <div className="text-center @[120px]:text-left">
                                <div className="text-xs text-gray-500 dark:text-dark-400 uppercase tracking-wide font-medium">
                                    {translation.get('pages-read')}
                                </div>
                                <div className="text-gray-900 dark:text-white font-semibold">
                                    {formatNumber(event.pages_read)}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {canViewDetails && (
                <div className="px-6 pb-6">
                    <div className="flex justify-center space-x-3">
                        <button
                            type="button"
                            className="px-6 py-3 bg-gradient-to-r from-primary-600 to-primary-700 hover:from-primary-700 hover:to-primary-800 text-white rounded-xl font-medium transition-all duration-200 shadow-lg hover:shadow-xl transform hover:scale-105 focus:outline-none focus:ring-2 focus:ring-primary-500/50"
                            onClick={() => {
                                onClose();
                                if (detailPath) {
                                    navigate(detailPath, {
                                        state: detailReturnState,
                                    });
                                }
                            }}
                        >
                            <span className="flex items-center">
                                <LuEye
                                    className="w-4 h-4 mr-2"
                                    aria-hidden="true"
                                />
                                {translation.get('view-details')}
                            </span>
                        </button>
                    </div>
                </div>
            )}
        </ModalShell>
    );
}
