import { useQuery } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { LuChevronDown } from 'react-icons/lu';

import { api } from '../../../shared/api';
import type { SiteResponse } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
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
    'w-full appearance-none bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg pl-3 pr-10 py-2.5 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500/60';

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
    children: React.ReactNode;
};

function SettingsField({
    label,
    htmlFor,
    hints,
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
                <div className="sm:w-56 flex-shrink-0">{children}</div>
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
    const siteQuery = useQuery({
        queryKey: ['site'],
        queryFn: () => api.site.get<SiteResponse>(),
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

    const visibleRegionOptions = useMemo(() => {
        const options = [...regionOptions.likelyRegions];

        if (selectedRegion) {
            const hasSelectedRegion = options.some(
                (option) => option.code === selectedRegion,
            );
            if (!hasSelectedRegion) {
                const matchingRegion = regionOptions.allRegions.find(
                    (option) => option.code === selectedRegion,
                );
                if (matchingRegion) {
                    options.unshift(matchingRegion);
                }
            }
        }

        return options;
    }, [regionOptions, selectedRegion]);

    const effectiveSelectedRegion =
        selectedRegion &&
        visibleRegionOptions.some((option) => option.code === selectedRegion)
            ? selectedRegion
            : (selectedLanguageOption?.defaultRegion ??
              regionOptions.likelyRegions[0]?.code ??
              visibleRegionOptions[0]?.code ??
              null);
    const previewLocale = joinLocale(selectedLanguage, effectiveSelectedRegion);
    const localePreview = useMemo(() => {
        try {
            return new Intl.DateTimeFormat(previewLocale, {
                dateStyle: 'full',
                timeStyle: 'short',
            }).format(PREVIEW_DATE);
        } catch {
            return new Intl.DateTimeFormat('en-US', {
                dateStyle: 'full',
                timeStyle: 'short',
            }).format(PREVIEW_DATE);
        }
    }, [previewLocale]);
    const numberPreview = useMemo(() => {
        try {
            return new Intl.NumberFormat(previewLocale, {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
            }).format(PREVIEW_NUMBER);
        } catch {
            return new Intl.NumberFormat('en-US', {
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
            }).format(PREVIEW_NUMBER);
        }
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
                    accentClass="bg-gradient-to-b from-purple-400 to-purple-600"
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
                    accentClass="bg-gradient-to-b from-amber-400 to-amber-600"
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
                    accentClass="bg-gradient-to-b from-primary-400 to-primary-600"
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
                            {visibleRegionOptions.map((option) => (
                                <option key={option.code} value={option.code}>
                                    {option.label}
                                </option>
                            ))}
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
            </PageContent>
        </>
    );
}
