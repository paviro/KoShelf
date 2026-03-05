import { useQuery } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';

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
import { PageContent } from '../../../shared/ui/layout/PageContent';
import { PageHeader } from '../../../shared/ui/layout/PageHeader';

const PREVIEW_DATE = new Date(Date.UTC(2026, 2, 5, 18, 20, 0, 0));
const PREVIEW_NUMBER = 10000;
const EMPTY_LANGUAGE_OPTIONS: SupportedLanguageOption[] = [];

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
        () => languageOptions.find((option) => option.code === selectedLanguage),
        [languageOptions, selectedLanguage],
    );
    const regionOptions = useMemo(
        () =>
            getRegionOptionsForLanguage(
                selectedLanguage,
                currentUiLocale,
                selectedLanguageOption?.defaultRegion ?? null,
            ),
        [currentUiLocale, selectedLanguage, selectedLanguageOption?.defaultRegion],
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
        selectedRegion && visibleRegionOptions.some((option) => option.code === selectedRegion)
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
        async (languageCode: string, regionCode: string | null): Promise<void> => {
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
            <PageContent className="space-y-6 md:space-y-8">
                <section className="max-w-3xl">
                    <div className="rounded-2xl border border-gray-200/70 dark:border-dark-700/70 bg-white/80 dark:bg-dark-900/80 backdrop-blur-sm p-5 md:p-6 space-y-6">
                        <div className="space-y-2">
                            <h2 className="text-lg md:text-xl font-semibold text-gray-900 dark:text-white">
                                {translation.get('appearance-setting')}
                            </h2>
                            <p className="text-sm text-gray-600 dark:text-dark-300">
                                {translation.get('theme-setting-description')}
                            </p>
                        </div>

                        <div className="space-y-2">
                            <label
                                htmlFor="settings-theme-preference"
                                className="block text-sm font-medium text-gray-700 dark:text-dark-200"
                            >
                                {translation.get('theme-setting')}
                            </label>
                            <select
                                id="settings-theme-preference"
                                value={selectedThemePreference}
                                className="w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500/60"
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
                            </select>
                        </div>

                        <div className="space-y-2 border-t border-gray-200/70 dark:border-dark-700/70 pt-2">
                            <h2 className="text-lg md:text-xl font-semibold text-gray-900 dark:text-white">
                                {translation.get('language-setting')}
                            </h2>
                            <p className="text-sm text-gray-600 dark:text-dark-300">
                                {translation.get('locale-setting-description')}
                            </p>
                        </div>

                        <div className="space-y-2">
                            <label
                                htmlFor="settings-language"
                                className="block text-sm font-medium text-gray-700 dark:text-dark-200"
                            >
                                {translation.get('language')}
                            </label>
                            <select
                                id="settings-language"
                                value={selectedLanguage}
                                disabled={!languageOptionsLoaded}
                                className="w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500/60"
                                onChange={(event) => {
                                    const nextLanguage = event.target.value;
                                    const nextLanguageOption = languageOptions.find(
                                        (option) =>
                                            option.code === nextLanguage,
                                    );
                                    const nextRegions =
                                        getRegionOptionsForLanguage(
                                            nextLanguage,
                                            currentUiLocale,
                                        );
                                    const fallbackRegion =
                                        nextLanguageOption?.defaultRegion ??
                                        nextRegions.likelyRegions[0]?.code ??
                                        null;
                                    void applyLocale(
                                        nextLanguage,
                                        fallbackRegion,
                                    );
                                }}
                            >
                                {languageOptionsLoaded ? (
                                    languageOptions.map((option) => (
                                        <option key={option.code} value={option.code}>
                                            {option.label}
                                        </option>
                                    ))
                                ) : (
                                    <option value={selectedLanguage}>
                                        {selectedLanguage.toUpperCase()}
                                    </option>
                                )}
                            </select>
                        </div>

                        <div className="space-y-2">
                            <label
                                htmlFor="settings-region"
                                className="block text-sm font-medium text-gray-700 dark:text-dark-200"
                            >
                                {translation.get('region-setting')}
                            </label>
                            <select
                                id="settings-region"
                                value={effectiveSelectedRegion ?? ''}
                                className="w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg px-3 py-2.5 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-primary-500/60"
                                onChange={(event) => {
                                    const value = event.target.value.trim();
                                    const nextRegion = value || null;
                                    void applyLocale(
                                        selectedLanguage,
                                        nextRegion,
                                    );
                                }}
                            >
                                {visibleRegionOptions.map((option) => (
                                    <option key={option.code} value={option.code}>
                                        {option.label}
                                    </option>
                                ))}
                            </select>
                        </div>

                        <div className="rounded-lg border border-gray-200/70 dark:border-dark-700/70 bg-gray-50/80 dark:bg-dark-850/70 p-3">
                            <p className="text-sm text-gray-600 dark:text-dark-300">
                                {translation.get('example')}
                            </p>
                            <div className="mt-1 space-y-1 text-sm md:text-base text-gray-900 dark:text-white">
                                <p>{localePreview}</p>
                                <p className="tabular-nums">{numberPreview}</p>
                            </div>
                        </div>
                    </div>
                </section>
            </PageContent>
        </>
    );
}
