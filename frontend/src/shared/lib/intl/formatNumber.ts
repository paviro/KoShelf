import { translation } from '../../i18n';
import { resolveLocalePatternContext } from './locale-options';

const FALLBACK_LOCALE = 'en-US';

function currentLocale(): string {
    return translation.getLanguage() || FALLBACK_LOCALE;
}

export function formatNumber(
    value: number,
    options?: Intl.NumberFormatOptions,
    locale = currentLocale(),
): string {
    const resolvedLocale =
        resolveLocalePatternContext(locale).patternLocale ?? locale;

    try {
        return new Intl.NumberFormat(resolvedLocale, options).format(value);
    } catch {
        return new Intl.NumberFormat(FALLBACK_LOCALE, options).format(value);
    }
}
