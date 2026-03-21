import {
    useCallback,
    useEffect,
    useId,
    useLayoutEffect,
    useRef,
    useState,
    type ReactNode,
} from 'react';
import { createPortal } from 'react-dom';
import {
    LuALargeSmall,
    LuAlignJustify,
    LuMinus,
    LuPlus,
    LuSettings,
} from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { useClickOutside } from '../../../shared/lib/dom/useClickOutside';
import type {
    ReaderModeControl,
    ReaderModeValue,
    ReaderStyleControl,
    ReaderToggleControl,
} from '../hooks/useReaderStyle';
import { DEFAULT_READER_LINE_SPACING } from '../lib/reader-theme';

const HEADER_ICON_BUTTON_CLASS =
    'flex items-center justify-center w-10 h-10 p-2.5 bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors duration-200 backdrop-blur-xs focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const PANEL_CONTROL_BUTTON_CLASS =
    'flex items-center justify-center w-9 h-9 rounded-lg border border-gray-300/60 dark:border-dark-700/60 bg-white/80 dark:bg-dark-900/60 text-gray-600 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const SEGMENTED_OPTION_BASE_CLASS =
    'h-9 rounded-lg border text-xs font-semibold transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const SEGMENTED_OPTION_ACTIVE_CLASS =
    'border-primary-400/60 dark:border-primary-300/50 bg-primary-100/80 dark:bg-primary-400/20 text-primary-800 dark:text-primary-100';

const SEGMENTED_OPTION_INACTIVE_CLASS =
    'border-gray-300/60 dark:border-dark-700/60 bg-white/80 dark:bg-dark-900/60 text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70';

const PANEL_OFFSET_PX = 8;

type ReaderSettingsPanelProps = {
    fontSize: ReaderStyleControl;
    lineSpacing: ReaderStyleControl;
    leftMargin: ReaderStyleControl;
    rightMargin: ReaderStyleControl;
    topMargin: ReaderStyleControl;
    bottomMargin: ReaderStyleControl;
    hyphenation: ReaderModeControl;
    floatingPunctuation: ReaderModeControl;
    embeddedFonts: ReaderToggleControl;
    onResetDefaults: () => void;
    canResetDefaults: boolean;
};

type FoliateContent = {
    doc?: Document | null;
};

type FoliateRendererLike = {
    getContents?: () => FoliateContent[];
};

type FoliateViewLike = HTMLElement & {
    renderer?: FoliateRendererLike;
};

function getReaderViews(): FoliateViewLike[] {
    return Array.from(
        document.querySelectorAll('foliate-view'),
    ) as FoliateViewLike[];
}

function formatRelativeSettingValue(value: number, baseline: number): string {
    const normalized = Math.round((value / baseline) * 10) / 10;
    const isDefaultValue = Math.abs(normalized - 1) < 0.001;

    return `${normalized.toLocaleString(undefined, {
        minimumFractionDigits: isDefaultValue ? 0 : 1,
        maximumFractionDigits: 1,
    })}x`;
}

function formatPointSettingValue(value: number): string {
    return `${Math.round(value)}pt`;
}

function formatPixelSettingValue(value: number): string {
    return `${Math.round(value)}px`;
}

function ReaderControlBadge({ text }: { text: string }) {
    return (
        <span className="inline-flex min-w-6 h-6 px-1.5 items-center justify-center rounded-md border border-primary-200/70 dark:border-primary-400/40 bg-primary-50/80 dark:bg-primary-400/10 text-[10px] font-semibold uppercase tracking-wide text-primary-700 dark:text-primary-200">
            {text}
        </span>
    );
}

type ReaderSettingControlProps = {
    icon: ReactNode;
    label: string;
    value: string;
    decreaseAriaLabel: string;
    increaseAriaLabel: string;
    control: ReaderStyleControl;
};

type ReaderChoiceOption<T extends string | boolean> = {
    value: T;
    label: string;
};

type ReaderChoiceControlProps<T extends string | boolean> = {
    icon: ReactNode;
    label: string;
    value: T;
    options: readonly ReaderChoiceOption<T>[];
    onSelect: (nextValue: T) => void;
};

function ReaderSettingControl({
    icon,
    label,
    value,
    decreaseAriaLabel,
    increaseAriaLabel,
    control,
}: ReaderSettingControlProps) {
    return (
        <div className="space-y-3">
            <div className="flex items-center gap-2 text-gray-900 dark:text-white">
                {icon}
                <span className="text-sm font-semibold">{label}</span>
            </div>

            <div className="flex items-center gap-2 p-1.5 rounded-xl bg-gray-100/70 dark:bg-dark-800/60 border border-gray-200/70 dark:border-dark-700/60">
                <button
                    type="button"
                    onClick={control.decrease}
                    className={PANEL_CONTROL_BUTTON_CLASS}
                    aria-label={decreaseAriaLabel}
                >
                    <LuMinus className="w-4 h-4" aria-hidden="true" />
                </button>

                <div className="flex-1 px-3 py-2 rounded-lg border border-gray-200/80 dark:border-dark-700/70 bg-white/85 dark:bg-dark-900/70 text-center">
                    <span className="text-sm font-medium text-gray-700 dark:text-dark-200 tabular-nums">
                        {value}
                    </span>
                </div>

                <button
                    type="button"
                    onClick={control.increase}
                    className={PANEL_CONTROL_BUTTON_CLASS}
                    aria-label={increaseAriaLabel}
                >
                    <LuPlus className="w-4 h-4" aria-hidden="true" />
                </button>
            </div>
        </div>
    );
}

function ReaderChoiceControl<T extends string | boolean>({
    icon,
    label,
    value,
    options,
    onSelect,
}: ReaderChoiceControlProps<T>) {
    return (
        <div className="space-y-3">
            <div className="flex items-center gap-2 text-gray-900 dark:text-white">
                {icon}
                <span className="text-sm font-semibold">{label}</span>
            </div>

            <div
                className="grid gap-2 p-1.5 rounded-xl bg-gray-100/70 dark:bg-dark-800/60 border border-gray-200/70 dark:border-dark-700/60"
                style={{
                    gridTemplateColumns: `repeat(${options.length}, minmax(0, 1fr))`,
                }}
            >
                {options.map((option) => {
                    const isActive = value === option.value;
                    return (
                        <button
                            key={String(option.value)}
                            type="button"
                            onClick={() => onSelect(option.value)}
                            className={`${SEGMENTED_OPTION_BASE_CLASS} ${
                                isActive
                                    ? SEGMENTED_OPTION_ACTIVE_CLASS
                                    : SEGMENTED_OPTION_INACTIVE_CLASS
                            }`}
                        >
                            {option.label}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

export function ReaderSettingsPanel({
    fontSize,
    lineSpacing,
    leftMargin,
    rightMargin,
    topMargin,
    bottomMargin,
    hyphenation,
    floatingPunctuation,
    embeddedFonts,
    onResetDefaults,
    canResetDefaults,
}: ReaderSettingsPanelProps) {
    const [open, setOpen] = useState(false);
    const buttonRef = useRef<HTMLButtonElement>(null);
    const panelRef = useRef<HTMLDivElement>(null);
    const [position, setPosition] = useState({ top: 0, right: 0 });
    const panelId = useId();

    const close = useCallback(() => setOpen(false), []);
    const updatePosition = useCallback(() => {
        const button = buttonRef.current;
        if (!button) {
            return;
        }

        const rect = button.getBoundingClientRect();
        setPosition({
            top: rect.bottom + PANEL_OFFSET_PX,
            right: window.innerWidth - rect.right,
        });
    }, []);

    useClickOutside(panelRef, close, open, buttonRef);

    useLayoutEffect(() => {
        if (!open) {
            return;
        }

        updatePosition();
    }, [open, updatePosition]);

    useEffect(() => {
        if (!open) {
            return;
        }

        window.addEventListener('resize', updatePosition);
        window.addEventListener('scroll', updatePosition, true);
        return () => {
            window.removeEventListener('resize', updatePosition);
            window.removeEventListener('scroll', updatePosition, true);
        };
    }, [open, updatePosition]);

    useEffect(() => {
        if (!open) {
            return;
        }

        const attachedDocs = new Set<Document>();
        const readers = getReaderViews();

        const attachDocListener = (doc: Document | null | undefined) => {
            if (!doc || attachedDocs.has(doc)) {
                return;
            }

            doc.addEventListener('pointerdown', close);
            attachedDocs.add(doc);
        };

        for (const reader of readers) {
            const docs = reader.renderer?.getContents?.() ?? [];
            for (const content of docs) {
                attachDocListener(content.doc);
            }
        }

        const handleReaderLoad = (event: Event) => {
            const detail = (event as CustomEvent<{ doc?: Document }>).detail;
            attachDocListener(detail?.doc);
        };

        for (const reader of readers) {
            reader.addEventListener('load', handleReaderLoad);
        }

        return () => {
            for (const reader of readers) {
                reader.removeEventListener('load', handleReaderLoad);
            }

            for (const doc of attachedDocs) {
                doc.removeEventListener('pointerdown', close);
            }
        };
    }, [close, open]);

    useEffect(() => {
        if (!open) {
            return;
        }

        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                close();
            }
        };

        document.addEventListener('keydown', handleEscape);
        return () => document.removeEventListener('keydown', handleEscape);
    }, [close, open]);

    const displayFontSize = formatPointSettingValue(fontSize.value);
    const displayLineSpacing = formatRelativeSettingValue(
        lineSpacing.value,
        DEFAULT_READER_LINE_SPACING,
    );
    const displayLeftMargin = formatPixelSettingValue(leftMargin.value);
    const displayRightMargin = formatPixelSettingValue(rightMargin.value);
    const displayTopMargin = formatPixelSettingValue(topMargin.value);
    const displayBottomMargin = formatPixelSettingValue(bottomMargin.value);

    const readerModeOptions: readonly ReaderChoiceOption<ReaderModeValue>[] = [
        { value: 'auto', label: translation.get('reader-mode-auto') },
        { value: 'on', label: translation.get('reader-mode-on') },
        { value: 'off', label: translation.get('reader-mode-off') },
    ];

    const embeddedFontOptions: readonly ReaderChoiceOption<boolean>[] = [
        { value: true, label: translation.get('reader-mode-on') },
        { value: false, label: translation.get('reader-mode-off') },
    ];

    const marginSettings: Array<{
        badge: string;
        label: string;
        value: string;
        decreaseAriaLabel: string;
        increaseAriaLabel: string;
        control: ReaderStyleControl;
    }> = [
        {
            badge: 'Lm',
            label: translation.get('reader-left-margin'),
            value: displayLeftMargin,
            decreaseAriaLabel: translation.get(
                'reader-left-margin-decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-left-margin-increase-aria',
            ),
            control: leftMargin,
        },
        {
            badge: 'Rm',
            label: translation.get('reader-right-margin'),
            value: displayRightMargin,
            decreaseAriaLabel: translation.get(
                'reader-right-margin-decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-right-margin-increase-aria',
            ),
            control: rightMargin,
        },
        {
            badge: 'Tm',
            label: translation.get('reader-top-margin'),
            value: displayTopMargin,
            decreaseAriaLabel: translation.get(
                'reader-top-margin-decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-top-margin-increase-aria',
            ),
            control: topMargin,
        },
        {
            badge: 'Bm',
            label: translation.get('reader-bottom-margin'),
            value: displayBottomMargin,
            decreaseAriaLabel: translation.get(
                'reader-bottom-margin-decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-bottom-margin-increase-aria',
            ),
            control: bottomMargin,
        },
    ];

    return (
        <>
            <button
                ref={buttonRef}
                type="button"
                onClick={() => setOpen((prev) => !prev)}
                className={`${HEADER_ICON_BUTTON_CLASS} ${
                    open
                        ? 'bg-gray-200/60 dark:bg-dark-700/60 border-gray-300/80 dark:border-dark-600/70 text-gray-900 dark:text-white'
                        : ''
                }`}
                aria-label={translation.get('reader-settings-aria')}
                aria-expanded={open}
                aria-haspopup="dialog"
                aria-controls={open ? panelId : undefined}
            >
                <LuSettings className="w-5 h-5" aria-hidden="true" />
            </button>

            {open &&
                createPortal(
                    <div
                        id={panelId}
                        ref={panelRef}
                        className="fixed w-80 max-w-[calc(100vw-1.5rem)] max-h-[70vh] overflow-y-auto bg-white/95 dark:bg-dark-900/88 backdrop-blur-xs border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl p-4 z-[100]"
                        style={{ top: position.top, right: position.right }}
                        role="dialog"
                        aria-label={translation.get('reader-settings-aria')}
                    >
                        <div className="space-y-4">
                            <ReaderSettingControl
                                icon={
                                    <LuALargeSmall
                                        className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                        aria-hidden="true"
                                    />
                                }
                                label={translation.get('reader-font-size')}
                                value={displayFontSize}
                                decreaseAriaLabel={translation.get(
                                    'reader-font-size-decrease-aria',
                                )}
                                increaseAriaLabel={translation.get(
                                    'reader-font-size-increase-aria',
                                )}
                                control={fontSize}
                            />

                            <ReaderSettingControl
                                icon={
                                    <LuAlignJustify
                                        className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                        aria-hidden="true"
                                    />
                                }
                                label={translation.get('reader-line-spacing')}
                                value={displayLineSpacing}
                                decreaseAriaLabel={translation.get(
                                    'reader-line-spacing-decrease-aria',
                                )}
                                increaseAriaLabel={translation.get(
                                    'reader-line-spacing-increase-aria',
                                )}
                                control={lineSpacing}
                            />

                            <ReaderChoiceControl
                                icon={<ReaderControlBadge text="Hy" />}
                                label={translation.get('reader-hyphenation')}
                                value={hyphenation.value}
                                options={readerModeOptions}
                                onSelect={hyphenation.setValue}
                            />

                            <ReaderChoiceControl
                                icon={<ReaderControlBadge text="Fp" />}
                                label={translation.get(
                                    'reader-floating-punctuation',
                                )}
                                value={floatingPunctuation.value}
                                options={readerModeOptions}
                                onSelect={floatingPunctuation.setValue}
                            />

                            <ReaderChoiceControl
                                icon={<ReaderControlBadge text="Ef" />}
                                label={translation.get('reader-embedded-fonts')}
                                value={embeddedFonts.value}
                                options={embeddedFontOptions}
                                onSelect={embeddedFonts.setValue}
                            />

                            {marginSettings.map((setting) => (
                                <ReaderSettingControl
                                    key={setting.badge}
                                    icon={
                                        <ReaderControlBadge
                                            text={setting.badge}
                                        />
                                    }
                                    label={setting.label}
                                    value={setting.value}
                                    decreaseAriaLabel={
                                        setting.decreaseAriaLabel
                                    }
                                    increaseAriaLabel={
                                        setting.increaseAriaLabel
                                    }
                                    control={setting.control}
                                />
                            ))}

                            <button
                                type="button"
                                onClick={onResetDefaults}
                                disabled={!canResetDefaults}
                                className="w-full px-3 py-2.5 rounded-lg border border-gray-300/70 dark:border-dark-700/70 bg-white/85 dark:bg-dark-900/70 text-sm font-medium text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                                aria-label={translation.get(
                                    'reader-reset-defaults-aria',
                                )}
                            >
                                {translation.get('reader-reset-defaults')}
                            </button>
                        </div>
                    </div>,
                    document.body,
                )}
        </>
    );
}
