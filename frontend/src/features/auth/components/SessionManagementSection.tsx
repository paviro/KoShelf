import { useCallback, useMemo, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';

import { api, isApiHttpError } from '../../../shared/api';
import { redirectToLogin } from '../../../shared/api-fetch';
import { translation } from '../../../shared/i18n';

const RELATIVE_TIME_UNITS: ReadonlyArray<{
    unit: Intl.RelativeTimeFormatUnit;
    seconds: number;
}> = [
    { unit: 'year', seconds: 365 * 24 * 60 * 60 },
    { unit: 'month', seconds: 30 * 24 * 60 * 60 },
    { unit: 'week', seconds: 7 * 24 * 60 * 60 },
    { unit: 'day', seconds: 24 * 60 * 60 },
    { unit: 'hour', seconds: 60 * 60 },
    { unit: 'minute', seconds: 60 },
    { unit: 'second', seconds: 1 },
];

function formatRelativeTimeFromNow(timestamp: string, locale: string): string {
    const parsedTime = Date.parse(timestamp);
    if (Number.isNaN(parsedTime)) {
        return '--';
    }

    const diffSeconds = Math.round((parsedTime - Date.now()) / 1000);
    const absSeconds = Math.abs(diffSeconds);
    const formatter = new Intl.RelativeTimeFormat(locale, {
        numeric: 'auto',
    });

    for (const unit of RELATIVE_TIME_UNITS) {
        if (absSeconds >= unit.seconds || unit.unit === 'second') {
            const value = Math.round(diffSeconds / unit.seconds);
            return formatter.format(value, unit.unit);
        }
    }

    return '--';
}

function resolveGenericApiErrorMessage(error: unknown): string {
    if (isApiHttpError(error)) {
        return error.apiMessage ?? translation.get('error-state.title');
    }

    return translation.get('error-state.connection-title');
}

function sortSessionsByLastSeenDesc<Session extends { last_seen_at: string }>(
    sessions: Session[],
): Session[] {
    return [...sessions].sort((left, right) => {
        const leftTime = Date.parse(left.last_seen_at);
        const rightTime = Date.parse(right.last_seen_at);

        if (Number.isNaN(leftTime) && Number.isNaN(rightTime)) {
            return 0;
        }

        if (Number.isNaN(leftTime)) {
            return 1;
        }

        if (Number.isNaN(rightTime)) {
            return -1;
        }

        return rightTime - leftTime;
    });
}

type SessionManagementSectionProps = {
    locale: string;
};

export function SessionManagementSection({
    locale,
}: SessionManagementSectionProps) {
    const queryClient = useQueryClient();
    const sessionsQuery = useQuery({
        queryKey: ['auth', 'sessions'],
        queryFn: () => api.getSessions(),
    });
    const [feedback, setFeedback] = useState<{
        type: 'success' | 'error';
        message: string;
    } | null>(null);
    const [revokePendingId, setRevokePendingId] = useState<string | null>(null);
    const [logoutPending, setLogoutPending] = useState(false);

    const orderedSessions = useMemo(() => {
        const sessions = sessionsQuery.data ?? [];
        const currentSession = sessions.find((session) => session.is_current);
        const otherSessions = sortSessionsByLastSeenDesc(
            sessions.filter((session) => !session.is_current),
        );

        if (!currentSession) {
            return otherSessions;
        }

        return [currentSession, ...otherSessions];
    }, [sessionsQuery.data]);

    const handleRevokeSession = useCallback(
        async (sessionId: string) => {
            if (!window.confirm(translation.get('revoke-session-confirm'))) {
                return;
            }

            setFeedback(null);
            setRevokePendingId(sessionId);

            try {
                await api.revokeSession(sessionId);
                setFeedback({
                    type: 'success',
                    message: translation.get('session-revoked'),
                });
                void sessionsQuery.refetch();
            } catch (error) {
                setFeedback({
                    type: 'error',
                    message: resolveGenericApiErrorMessage(error),
                });
            } finally {
                setRevokePendingId(null);
            }
        },
        [sessionsQuery],
    );

    const handleLogout = useCallback(async () => {
        if (logoutPending) {
            return;
        }

        setFeedback(null);
        setLogoutPending(true);

        try {
            await api.logout();
            queryClient.clear();
            redirectToLogin();
        } catch (error) {
            setFeedback({
                type: 'error',
                message: resolveGenericApiErrorMessage(error),
            });
            setLogoutPending(false);
        }
    }, [logoutPending, queryClient]);

    return (
        <>
            {feedback ? (
                <p
                    className={`text-sm px-3 py-2 rounded-lg border ${
                        feedback.type === 'success'
                            ? 'border-emerald-300/70 dark:border-emerald-500/40 bg-emerald-50/80 dark:bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                            : 'border-red-200/80 dark:border-red-500/40 bg-red-50/80 dark:bg-red-500/10 text-red-700 dark:text-red-300'
                    }`}
                >
                    {feedback.message}
                </p>
            ) : null}

            {sessionsQuery.isLoading ? (
                <p className="text-sm text-gray-500 dark:text-dark-400">
                    {translation.get('current-session')}...
                </p>
            ) : null}

            {sessionsQuery.isError ? (
                <p className="text-sm text-red-700 dark:text-red-300">
                    {resolveGenericApiErrorMessage(sessionsQuery.error)}
                </p>
            ) : null}

            {sessionsQuery.isSuccess && orderedSessions.length > 0 ? (
                <ul className="space-y-4">
                    {orderedSessions.map((session) => (
                        <li
                            key={session.id}
                            className={`rounded-lg border px-4 py-3 ${
                                session.is_current
                                    ? 'border-primary-300/80 dark:border-primary-600/60 bg-primary-50/40 dark:bg-primary-900/15'
                                    : 'bg-white dark:bg-dark-850/50 border-gray-200/70 dark:border-dark-700/70'
                            }`}
                        >
                            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
                                <div className="min-w-0">
                                    <div className="flex flex-wrap items-center gap-2">
                                        <p className="text-sm font-medium text-gray-900 dark:text-white">
                                            {session.browser} on {session.os}
                                        </p>
                                        {session.is_current ? (
                                            <span className="text-xs font-medium px-2 py-0.5 rounded-full border border-primary-300/70 dark:border-primary-600/70 text-primary-700 dark:text-primary-300 bg-primary-100/80 dark:bg-primary-900/40">
                                                {translation.get('this-device')}
                                            </span>
                                        ) : null}
                                    </div>
                                    <p className="mt-0.5 text-xs text-gray-500 dark:text-dark-400">
                                        {session.last_seen_ip ?? '--'} ·{' '}
                                        {translation.get('last-active')}{' '}
                                        {formatRelativeTimeFromNow(
                                            session.last_seen_at,
                                            locale,
                                        )}
                                    </p>
                                </div>

                                {session.is_current ? (
                                    <button
                                        type="button"
                                        className="inline-flex items-center justify-center rounded-lg min-w-28 px-4 py-2.5 text-sm font-medium border border-gray-300/80 dark:border-dark-600 bg-gray-50 dark:bg-dark-800/70 text-gray-800 dark:text-dark-100 hover:bg-gray-100 dark:hover:bg-dark-700 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
                                        disabled={logoutPending}
                                        onClick={() => void handleLogout()}
                                    >
                                        {translation.get('logout')}
                                    </button>
                                ) : (
                                    <button
                                        type="button"
                                        className="inline-flex items-center justify-center rounded-lg min-w-28 px-4 py-2.5 text-sm font-medium border border-red-300/80 dark:border-red-500/50 text-red-700 dark:text-red-300 hover:bg-red-50 dark:hover:bg-red-500/10 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                        disabled={
                                            revokePendingId === session.id
                                        }
                                        onClick={() =>
                                            void handleRevokeSession(session.id)
                                        }
                                    >
                                        {translation.get('revoke-session')}
                                    </button>
                                )}
                            </div>
                        </li>
                    ))}
                </ul>
            ) : null}
        </>
    );
}
