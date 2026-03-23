import { useQuery } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';

import { PasswordChangeSection } from '../../auth/components/PasswordChangeSection';
import { SessionManagementSection } from '../../auth/components/SessionManagementSection';
import { api } from '../../../shared/api';
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
import { SettingsField } from '../components/SettingsField';
import { SettingsSelect } from '../components/SettingsSelect';

const PREVIEW_DATE = new Date(Date.UTC(2026, 2, 5, 18, 20, 0, 0));
const PREVIEW_NUMBER = 10000;
const EMPTY_LANGUAGE_OPTIONS: SupportedLanguageOption[] = [];

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

export function SettingsRoute() {
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

    const authEnabled = siteQuery.data?.capabilities.auth_enabled === true;

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
                        hints={[translation.get('theme-setting.description')]}
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
                                {translation.get('theme-setting.option-auto')}
                            </option>
                            <option value="light">
                                {translation.get('theme-setting.option-light')}
                            </option>
                            <option value="dark">
                                {translation.get('theme-setting.option-dark')}
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
                            translation.get('prefetch-setting.description'),
                            translation.get('prefetch-setting.connection-note'),
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
                                {translation.get(
                                    'prefetch-setting.option-enabled',
                                )}
                            </option>
                            <option value="disabled">
                                {translation.get(
                                    'prefetch-setting.option-disabled',
                                )}
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
                        hints={[translation.get('language-setting.hint')]}
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
                        hints={[translation.get('region-setting.hint')]}
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
                                            'region-setting.majority-group',
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
                                                'region-setting.all-group',
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
                            <p className="text-sm font-medium uppercase tracking-wider text-gray-500 dark:text-dark-400 mb-1.5">
                                {translation.get('region-setting.preview-date')}
                            </p>
                            <p className="text-base md:text-lg font-medium text-gray-900 dark:text-white">
                                {localePreview}
                            </p>
                        </div>
                        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4">
                            <p className="text-sm font-medium uppercase tracking-wider text-gray-500 dark:text-dark-400 mb-1.5">
                                {translation.get(
                                    'region-setting.preview-number',
                                )}
                            </p>
                            <p className="text-base md:text-lg font-medium tabular-nums text-gray-900 dark:text-white">
                                {numberPreview}
                            </p>
                        </div>
                    </div>
                </SettingsSection>

                {authEnabled ? (
                    <SettingsSection
                        accentClass="bg-linear-to-b from-rose-400 to-red-600"
                        title={translation.get('change-password.setting')}
                    >
                        <PasswordChangeSection
                            minPasswordChars={
                                siteQuery.data?.password_policy?.min_chars
                            }
                        />
                    </SettingsSection>
                ) : null}

                {authEnabled ? (
                    <SettingsSection
                        accentClass="bg-linear-to-b from-sky-400 to-blue-600"
                        title={translation.get('session-management.setting')}
                    >
                        <SessionManagementSection locale={currentUiLocale} />
                    </SettingsSection>
                ) : null}
            </PageContent>
        </>
    );
}
