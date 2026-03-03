import { translation } from '../../i18n';

const FALLBACK_LOCALE = 'en-US';

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
    const locale = translation.getLanguage() || FALLBACK_LOCALE;

    try {
        return new Intl.NumberFormat(locale, options).format(value);
    } catch {
        return new Intl.NumberFormat(FALLBACK_LOCALE, options).format(value);
    }
}
