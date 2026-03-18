import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { LuChevronDown } from 'react-icons/lu';

import { api, isApiHttpError } from '../../../shared/api';
import { redirectToLogin } from '../../../shared/api-fetch';
import { translation } from '../../../shared/i18n';
import { formatDateObject } from '../../../shared/lib/intl/formatDate';
import { formatNumber } from '../../../shared/lib/intl/formatNumber';
import {
    getRegionOptionsForLanguage,
    getSupportedLanguageOptions,
    joinLocale,
    splitLocale,
    type SupportedLanguageOption,
} from '../../../shared/lib/intl/locale-options';
import {
    getThemePreference,
    setThemePreference,
    THEME_PREFERENCE_CHANGE_EVENT,
    type ThemePreference,
} from '../../../shared/theme';
import {
    getPrefetchOnIntentPreference,
    setPrefetchOnIntentPreference,
} from '../../../shared/lib/network/prefetch-preference';
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';

const PREVIEW_DATE = new Date(Date.UTC(2026, 2, 5, 18, 20, 0, 0));
const PREVIEW_NUMBER = 10000;
const EMPTY_LANGUAGE_OPTIONS: SupportedLanguageOption[] = [];

const selectClassName =
    'w-full appearance-none bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg pl-3 pr-10 py-2.5 text-gray-900 dark:text-white focus:outline-hidden focus:ring-2 focus:ring-primary-500/60';

const inputClassName =
    'w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg px-3 py-2.5 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-dark-400 focus:outline-hidden focus:ring-2 focus:ring-primary-500/60 disabled:opacity-60 disabled:cursor-not-allowed';

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

type SettingsSectionProps = {
    accentClass: string;
    title: string;
    children: React.ReactNode;
};

function SettingsSection({
    accentClass,
    title,
    children,
}: SettingsSectionProps) {
    return (
        <section>
            <div className="flex items-center min-h-10 space-x-3 mb-4 md:mb-6 pb-4 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className={`w-2 h-6 md:h-8 rounded-full ${accentClass}`} />
                <h2 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                    {title}
                </h2>
            </div>
            <div className="space-y-4">{children}</div>
        </section>
    );
}

type SettingsFieldProps = {
    label: string;
    htmlFor: string;
    hints?: string[];
    wide?: boolean;
    children: React.ReactNode;
};

function SettingsField({
    label,
    htmlFor,
    hints,
    wide,
    children,
}: SettingsFieldProps) {
    return (
        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4">
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                <div className="space-y-0.5">
                    <label
                        htmlFor={htmlFor}
                        className="block text-sm font-medium text-gray-900 dark:text-white"
                    >
                        {label}
                    </label>
                    {hints?.map((line, i) => (
                        <p
                            key={i}
                            className="text-xs text-gray-500 dark:text-dark-400"
                        >
                            {line}
                        </p>
                    ))}
                </div>
                <div className={`${wide ? 'sm:w-72' : 'sm:w-56'} shrink-0`}>
                    {children}
                </div>
            </div>
        </div>
    );
}

type SettingsSelectProps = {
    children: React.ReactNode;
    className?: string;
} & Omit<
    React.SelectHTMLAttributes<HTMLSelectElement>,
    'children' | 'className'
>;

function SettingsSelect({
    children,
    className,
    ...props
}: SettingsSelectProps) {
    const resolvedClassName = className
        ? `${selectClassName} ${className}`
        : selectClassName;

    return (
        <div className="relative">
            <select {...props} className={resolvedClassName}>
                {children}
            </select>
            <LuChevronDown
                className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-dark-400"
                aria-hidden="true"
            />
        </div>
    );
}

export function SettingsRoute() {
    const queryClient = useQueryClient();
    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.getSite(),
    });
    const languageOptionsQuery = useQuery({
        queryKey: ['settings', 'language-options'],
        queryFn: getSupportedLanguageOptions,
        staleTime: Number.POSITIVE_INFINITY,
    });
    const [selectedThemePreference, setSelectedThemePreference] =
        useState<ThemePreference>(() => getThemePreference());
    const [prefetchOnIntentEnabled, setPrefetchOnIntentEnabled] = useState(() =>
        getPrefetchOnIntentPreference(),
    );
    const [currentPassword, setCurrentPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordPending, setPasswordPending] = useState(false);
    const [passwordFeedback, setPasswordFeedback] = useState<{
        type: 'success' | 'error';
        message: string;
    } | null>(null);
    const [sessionFeedback, setSessionFeedback] = useState<{
        type: 'success' | 'error';
        message: string;
    } | null>(null);
    const [revokePendingId, setRevokePendingId] = useState<string | null>(null);
    const [logoutPending, setLogoutPending] = useState(false);

    const authEnabled = siteQuery.data?.capabilities.auth_enabled === true;
    const sessionsQuery = useQuery({
        queryKey: ['auth', 'sessions'],
        queryFn: () => api.getSessions(),
        enabled: authEnabled,
    });

    const currentUiLocale = translation.getLanguage();
    const currentLocaleParts = splitLocale(currentUiLocale);
    const languageOptions = languageOptionsQuery.data ?? EMPTY_LANGUAGE_OPTIONS;
    const languageOptionsLoaded = languageOptions.length > 0;
    const selectedLanguage = languageOptions.some(
        (option) => option.code === currentLocaleParts.languageCode,
    )
        ? currentLocaleParts.languageCode
        : (languageOptions[0]?.code ?? 'en');
    const selectedLanguageOption = useMemo(
        () =>
            languageOptions.find((option) => option.code === selectedLanguage),
        [languageOptions, selectedLanguage],
    );
    const regionOptions = useMemo(
        () =>
            getRegionOptionsForLanguage(
                selectedLanguage,
                currentUiLocale,
                selectedLanguageOption?.defaultRegion ?? null,
            ),
        [
            currentUiLocale,
            selectedLanguage,
            selectedLanguageOption?.defaultRegion,
        ],
    );
    const selectedRegion = currentLocaleParts.regionCode;
    const likelyRegionOptions = regionOptions.likelyRegions;
    const otherRegionOptions = useMemo(() => {
        const likelyCodes = new Set(
            regionOptions.likelyRegions.map((option) => option.code),
        );
        return regionOptions.allRegions.filter(
            (option) => !likelyCodes.has(option.code),
        );
    }, [regionOptions]);

    const effectiveSelectedRegion =
        selectedRegion &&
        regionOptions.allRegions.some(
            (option) => option.code === selectedRegion,
        )
            ? selectedRegion
            : (selectedLanguageOption?.defaultRegion ??
              regionOptions.likelyRegions[0]?.code ??
              regionOptions.allRegions[0]?.code ??
              null);
    const previewLocale = joinLocale(selectedLanguage, effectiveSelectedRegion);
    const localePreview = useMemo(
        () =>
            formatDateObject(
                PREVIEW_DATE,
                { dateStyle: 'full', timeStyle: 'short' },
                '--',
                previewLocale,
            ),
        [previewLocale],
    );
    const numberPreview = useMemo(() => {
        return formatNumber(
            PREVIEW_NUMBER,
            {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
            },
            previewLocale,
        );
    }, [previewLocale]);

    const applyLocale = useCallback(
        async (
            languageCode: string,
            regionCode: string | null,
        ): Promise<void> => {
            await translation.setLanguage(joinLocale(languageCode, regionCode));
        },
        [],
    );

    const handlePasswordSubmit = useCallback(
        async (event: React.FormEvent<HTMLFormElement>) => {
            event.preventDefault();

            if (passwordPending) {
                return;
            }

            if (newPassword.length < 8) {
                setPasswordFeedback({
                    type: 'error',
                    message: translation.get('password-too-short'),
                });
                return;
            }

            if (newPassword !== confirmPassword) {
                setPasswordFeedback({
                    type: 'error',
                    message: translation.get('password-mismatch'),
                });
                return;
            }

            setPasswordPending(true);
            setPasswordFeedback(null);

            try {
                await api.changePassword(currentPassword, newPassword);
                setCurrentPassword('');
                setNewPassword('');
                setConfirmPassword('');
                setPasswordFeedback({
                    type: 'success',
                    message: translation.get('password-changed'),
                });
                void queryClient.invalidateQueries({
                    queryKey: ['auth', 'sessions'],
                });
            } catch (error) {
                if (
                    isApiHttpError(error) &&
                    error.status === 400 &&
                    error.code === 'invalid_credentials'
                ) {
                    setPasswordFeedback({
                        type: 'error',
                        message: translation.get('incorrect-password'),
                    });
                } else if (
                    isApiHttpError(error) &&
                    error.status === 400 &&
                    error.code === 'invalid_query' &&
                    error.apiMessage?.toLowerCase().includes('at least 8')
                ) {
                    setPasswordFeedback({
                        type: 'error',
                        message: translation.get('password-too-short'),
                    });
                } else {
                    setPasswordFeedback({
                        type: 'error',
                        message: resolveGenericApiErrorMessage(error),
                    });
                }
            } finally {
                setPasswordPending(false);
            }
        },
        [
            confirmPassword,
            currentPassword,
            newPassword,
            passwordPending,
            queryClient,
        ],
    );

    const handleRevokeSession = useCallback(
        async (sessionId: string) => {
            if (!window.confirm(translation.get('revoke-session-confirm'))) {
                return;
            }

            setSessionFeedback(null);
            setRevokePendingId(sessionId);

            try {
                await api.revokeSession(sessionId);
                setSessionFeedback({
                    type: 'success',
                    message: translation.get('session-revoked'),
                });
                await queryClient.invalidateQueries({
                    queryKey: ['auth', 'sessions'],
                });
            } catch (error) {
                setSessionFeedback({
                    type: 'error',
                    message: resolveGenericApiErrorMessage(error),
                });
            } finally {
                setRevokePendingId(null);
            }
        },
        [queryClient],
    );

    const handleLogout = useCallback(async () => {
        if (logoutPending) {
            return;
        }

        setSessionFeedback(null);
        setLogoutPending(true);

        try {
            await api.logout();
            redirectToLogin();
        } catch (error) {
            setSessionFeedback({
                type: 'error',
                message: resolveGenericApiErrorMessage(error),
            });
            setLogoutPending(false);
        }
    }, [logoutPending]);

    useEffect(() => {
        if (!siteQuery.data?.title) {
            return;
        }

        document.title = `${translation.get('settings')} - ${siteQuery.data.title}`;
    }, [currentUiLocale, siteQuery.data?.title]);
    useEffect(() => {
        const handleThemePreferenceChange = () => {
            setSelectedThemePreference(getThemePreference());
        };

        window.addEventListener(
            THEME_PREFERENCE_CHANGE_EVENT,
            handleThemePreferenceChange,
        );
        return () => {
            window.removeEventListener(
                THEME_PREFERENCE_CHANGE_EVENT,
                handleThemePreferenceChange,
            );
        };
    }, []);

    return (
        <>
            <PageHeader title={translation.get('settings')} />
            <PageContent className="pt-[92px] md:pt-[100px] space-y-6 md:space-y-8">
                <SettingsSection
                    accentClass="bg-linear-to-b from-purple-400 to-purple-600"
                    title={translation.get('appearance-setting')}
                >
                    <SettingsField
                        label={translation.get('theme-setting')}
                        htmlFor="settings-theme-preference"
                        hints={[translation.get('theme-setting-description')]}
                    >
                        <SettingsSelect
                            id="settings-theme-preference"
                            value={selectedThemePreference}
                            onChange={(event) => {
                                const nextPreference = event.target
                                    .value as ThemePreference;
                                setThemePreference(nextPreference);
                                setSelectedThemePreference(nextPreference);
                            }}
                        >
                            <option value="auto">
                                {translation.get('theme-option-auto')}
                            </option>
                            <option value="light">
                                {translation.get('theme-option-light')}
                            </option>
                            <option value="dark">
                                {translation.get('theme-option-dark')}
                            </option>
                        </SettingsSelect>
                    </SettingsField>
                </SettingsSection>

                <SettingsSection
                    accentClass="bg-linear-to-b from-amber-400 to-amber-600"
                    title={translation.get('prefetch-setting')}
                >
                    <SettingsField
                        label={translation.get('prefetch-setting')}
                        htmlFor="settings-prefetch-on-intent"
                        hints={[
                            translation.get('prefetch-setting-description'),
                            translation.get('prefetch-setting-connection-note'),
                        ]}
                    >
                        <SettingsSelect
                            id="settings-prefetch-on-intent"
                            value={
                                prefetchOnIntentEnabled ? 'enabled' : 'disabled'
                            }
                            onChange={(event) => {
                                const enabled =
                                    event.target.value === 'enabled';
                                setPrefetchOnIntentPreference(enabled);
                                setPrefetchOnIntentEnabled(enabled);
                            }}
                        >
                            <option value="enabled">
                                {translation.get('prefetch-option-enabled')}
                            </option>
                            <option value="disabled">
                                {translation.get('prefetch-option-disabled')}
                            </option>
                        </SettingsSelect>
                    </SettingsField>
                </SettingsSection>

                <SettingsSection
                    accentClass="bg-linear-to-b from-primary-400 to-primary-600"
                    title={translation.get('language-setting')}
                >
                    <SettingsField
                        label={translation.get('language')}
                        htmlFor="settings-language"
                        hints={[translation.get('language-setting-hint')]}
                    >
                        <SettingsSelect
                            id="settings-language"
                            value={selectedLanguage}
                            disabled={!languageOptionsLoaded}
                            onChange={(event) => {
                                const nextLanguage = event.target.value;
                                const nextLanguageOption = languageOptions.find(
                                    (option) => option.code === nextLanguage,
                                );
                                const nextRegions = getRegionOptionsForLanguage(
                                    nextLanguage,
                                    currentUiLocale,
                                );
                                const fallbackRegion =
                                    nextLanguageOption?.defaultRegion ??
                                    nextRegions.likelyRegions[0]?.code ??
                                    nextRegions.allRegions[0]?.code ??
                                    null;
                                void applyLocale(nextLanguage, fallbackRegion);
                            }}
                        >
                            {languageOptionsLoaded ? (
                                languageOptions.map((option) => (
                                    <option
                                        key={option.code}
                                        value={option.code}
                                    >
                                        {option.label}
                                    </option>
                                ))
                            ) : (
                                <option value={selectedLanguage}>
                                    {selectedLanguage.toUpperCase()}
                                </option>
                            )}
                        </SettingsSelect>
                    </SettingsField>

                    <SettingsField
                        label={translation.get('region-setting')}
                        htmlFor="settings-region"
                        hints={[translation.get('region-setting-hint')]}
                    >
                        <SettingsSelect
                            id="settings-region"
                            value={effectiveSelectedRegion ?? ''}
                            onChange={(event) => {
                                const value = event.target.value.trim();
                                const nextRegion = value || null;
                                void applyLocale(selectedLanguage, nextRegion);
                            }}
                        >
                            {likelyRegionOptions.length > 0 ? (
                                <>
                                    <optgroup
                                        label={translation.get(
                                            'region-setting-majority-group',
                                        )}
                                    >
                                        {likelyRegionOptions.map((option) => (
                                            <option
                                                key={option.code}
                                                value={option.code}
                                            >
                                                {option.label}
                                            </option>
                                        ))}
                                    </optgroup>
                                    {otherRegionOptions.length > 0 ? (
                                        <optgroup
                                            label={translation.get(
                                                'region-setting-all-group',
                                            )}
                                        >
                                            {otherRegionOptions.map(
                                                (option) => (
                                                    <option
                                                        key={option.code}
                                                        value={option.code}
                                                    >
                                                        {option.label}
                                                    </option>
                                                ),
                                            )}
                                        </optgroup>
                                    ) : null}
                                </>
                            ) : (
                                regionOptions.allRegions.map((option) => (
                                    <option
                                        key={option.code}
                                        value={option.code}
                                    >
                                        {option.label}
                                    </option>
                                ))
                            )}
                        </SettingsSelect>
                    </SettingsField>

                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4">
                            <p className="text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-dark-400 mb-1.5">
                                {translation.get('preview-date')}
                            </p>
                            <p className="text-sm md:text-base text-gray-900 dark:text-white">
                                {localePreview}
                            </p>
                        </div>
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4">
                            <p className="text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-dark-400 mb-1.5">
                                {translation.get('preview-number')}
                            </p>
                            <p className="text-sm md:text-base tabular-nums text-gray-900 dark:text-white">
                                {numberPreview}
                            </p>
                        </div>
                    </div>
                </SettingsSection>

                {authEnabled ? (
                    <SettingsSection
                        accentClass="bg-linear-to-b from-rose-400 to-red-600"
                        title={translation.get('password-setting')}
                    >
                        <form
                            onSubmit={handlePasswordSubmit}
                            className="space-y-4"
                        >
                            <SettingsField
                                label={translation.get('current-password')}
                                htmlFor="settings-current-password"
                                wide
                            >
                                <input
                                    id="settings-current-password"
                                    type="password"
                                    autoComplete="current-password"
                                    placeholder={translation.get(
                                        'current-password-placeholder',
                                    )}
                                    className={inputClassName}
                                    value={currentPassword}
                                    onChange={(event) => {
                                        setCurrentPassword(event.target.value);
                                        setPasswordFeedback(null);
                                    }}
                                    disabled={passwordPending}
                                />
                            </SettingsField>

                            <SettingsField
                                label={translation.get('new-password')}
                                htmlFor="settings-new-password"
                                hints={[translation.get('new-password-hint')]}
                                wide
                            >
                                <input
                                    id="settings-new-password"
                                    type="password"
                                    autoComplete="new-password"
                                    placeholder={translation.get(
                                        'new-password-placeholder',
                                    )}
                                    className={inputClassName}
                                    value={newPassword}
                                    onChange={(event) => {
                                        setNewPassword(event.target.value);
                                        setPasswordFeedback(null);
                                    }}
                                    disabled={passwordPending}
                                />
                            </SettingsField>

                            <SettingsField
                                label={translation.get('confirm-password')}
                                htmlFor="settings-confirm-password"
                                wide
                            >
                                <input
                                    id="settings-confirm-password"
                                    type="password"
                                    autoComplete="new-password"
                                    placeholder={translation.get(
                                        'confirm-password-placeholder',
                                    )}
                                    className={inputClassName}
                                    value={confirmPassword}
                                    onChange={(event) => {
                                        setConfirmPassword(event.target.value);
                                        setPasswordFeedback(null);
                                    }}
                                    disabled={passwordPending}
                                />
                            </SettingsField>

                            {passwordFeedback ? (
                                <p
                                    className={`text-sm px-3 py-2 rounded-lg border ${
                                        passwordFeedback.type === 'success'
                                            ? 'border-emerald-300/70 dark:border-emerald-500/40 bg-emerald-50/80 dark:bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                            : 'border-red-200/80 dark:border-red-500/40 bg-red-50/80 dark:bg-red-500/10 text-red-700 dark:text-red-300'
                                    }`}
                                >
                                    {passwordFeedback.message}
                                </p>
                            ) : null}

                            <SettingsField
                                label={translation.get('change-password')}
                                htmlFor="settings-change-password-submit"
                            >
                                <button
                                    id="settings-change-password-submit"
                                    type="submit"
                                    className="w-full inline-flex items-center justify-center rounded-lg px-4 py-2.5 text-sm font-medium bg-primary-600 text-white hover:bg-primary-500 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
                                    disabled={
                                        passwordPending ||
                                        currentPassword.length === 0 ||
                                        newPassword.length === 0 ||
                                        confirmPassword.length === 0
                                    }
                                >
                                    {translation.get('change-password')}
                                </button>
                            </SettingsField>
                        </form>
                    </SettingsSection>
                ) : null}

                {authEnabled ? (
                    <SettingsSection
                        accentClass="bg-linear-to-b from-sky-400 to-blue-600"
                        title={translation.get('sessions-setting')}
                    >
                        {sessionFeedback ? (
                            <p
                                className={`text-sm px-3 py-2 rounded-lg border ${
                                    sessionFeedback.type === 'success'
                                        ? 'border-emerald-300/70 dark:border-emerald-500/40 bg-emerald-50/80 dark:bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                                        : 'border-red-200/80 dark:border-red-500/40 bg-red-50/80 dark:bg-red-500/10 text-red-700 dark:text-red-300'
                                }`}
                            >
                                {sessionFeedback.message}
                            </p>
                        ) : null}

                        {sessionsQuery.isLoading ? (
                            <p className="text-sm text-gray-500 dark:text-dark-400">
                                {translation.get('current-session')}...
                            </p>
                        ) : null}

                        {sessionsQuery.isError ? (
                            <p className="text-sm text-red-700 dark:text-red-300">
                                {resolveGenericApiErrorMessage(
                                    sessionsQuery.error,
                                )}
                            </p>
                        ) : null}

                        {sessionsQuery.isSuccess &&
                        sessionsQuery.data.length > 0 ? (
                            <ul className="space-y-4">
                                {sessionsQuery.data.map((session) => (
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
                                                        {session.browser} on{' '}
                                                        {session.os}
                                                    </p>
                                                    {session.is_current ? (
                                                        <span className="text-xs font-medium px-2 py-0.5 rounded-full border border-primary-300/70 dark:border-primary-600/70 text-primary-700 dark:text-primary-300 bg-primary-100/80 dark:bg-primary-900/40">
                                                            {translation.get(
                                                                'this-device',
                                                            )}
                                                        </span>
                                                    ) : null}
                                                </div>
                                                <p className="mt-0.5 text-xs text-gray-500 dark:text-dark-400">
                                                    {session.last_seen_ip ??
                                                        '--'}{' '}
                                                    ·{' '}
                                                    {translation.get(
                                                        'last-active',
                                                    )}{' '}
                                                    {formatRelativeTimeFromNow(
                                                        session.last_seen_at,
                                                        currentUiLocale,
                                                    )}
                                                </p>
                                            </div>

                                            {session.is_current ? (
                                                <button
                                                    type="button"
                                                    className="inline-flex items-center justify-center rounded-lg min-w-28 px-4 py-2.5 text-sm font-medium border border-gray-300/80 dark:border-dark-600 bg-gray-50 dark:bg-dark-800/70 text-gray-800 dark:text-dark-100 hover:bg-gray-100 dark:hover:bg-dark-700 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
                                                    disabled={logoutPending}
                                                    onClick={() =>
                                                        void handleLogout()
                                                    }
                                                >
                                                    {translation.get('logout')}
                                                </button>
                                            ) : (
                                                <button
                                                    type="button"
                                                    className="inline-flex items-center justify-center rounded-lg min-w-28 px-4 py-2.5 text-sm font-medium border border-red-300/80 dark:border-red-500/50 text-red-700 dark:text-red-300 hover:bg-red-50 dark:hover:bg-red-500/10 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                                    disabled={
                                                        revokePendingId ===
                                                        session.id
                                                    }
                                                    onClick={() =>
                                                        void handleRevokeSession(
                                                            session.id,
                                                        )
                                                    }
                                                >
                                                    {translation.get(
                                                        'revoke-session',
                                                    )}
                                                </button>
                                            )}
                                        </div>
                                    </li>
                                ))}
                            </ul>
                        ) : null}
                    </SettingsSection>
                ) : null}
            </PageContent>
        </>
    );
}
