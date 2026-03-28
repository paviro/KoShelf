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
    LuChevronDown,
    LuMinus,
    LuPlus,
    LuQuote,
    LuSettings,
    LuType,
    LuUnfoldHorizontal,
    LuWrapText,
} from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { computeOverlayPosition } from '../../../shared/overlay/anchored-overlay';
import { useClickOutside } from '../../../shared/lib/dom/useClickOutside';
import type {
    ReaderModeControl,
    ReaderModeValue,
    ReaderStyleControl,
    ReaderToggleControl,
} from '../hooks/useReaderStyle';
import { Button } from '../../../shared/ui/button/Button';

const PANEL_CONTROL_BUTTON_CLASS =
    'flex items-center justify-center w-9 h-9 rounded-lg border border-gray-300/60 dark:border-dark-700/60 bg-white/80 dark:bg-dark-900/60 text-gray-600 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const COMPACT_CONTROL_BUTTON_CLASS =
    'flex items-center justify-center w-7 h-7 rounded-md border border-gray-300/60 dark:border-dark-700/60 bg-white/80 dark:bg-dark-900/60 text-gray-600 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const SEGMENTED_OPTION_BASE_CLASS =
    'h-9 rounded-lg border text-xs font-semibold transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const SEGMENTED_OPTION_ACTIVE_CLASS =
    'border-primary-400/60 dark:border-primary-300/50 bg-primary-100/80 dark:bg-primary-400/20 text-primary-800 dark:text-primary-100';

const SEGMENTED_OPTION_INACTIVE_CLASS =
    'border-gray-300/60 dark:border-dark-700/60 bg-white/80 dark:bg-dark-900/60 text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70';

const PANEL_OFFSET_PX = 8;

function formatLineSpacingForEdit(value: number): string {
    return String(Math.round(value * 100));
}

function parseLineSpacingInput(input: number): number {
    return input / 100;
}

function EditableValue({
    value,
    displayValue,
    onCommit,
    className,
    formatForEdit,
    parseInput,
}: {
    value: number;
    displayValue: string;
    onCommit: (raw: number) => void;
    className?: string;
    formatForEdit?: (value: number) => string;
    parseInput?: (input: number) => number;
}) {
    const [editing, setEditing] = useState(false);
    const [draft, setDraft] = useState('');
    const inputRef = useRef<HTMLInputElement>(null);

    const startEditing = useCallback(() => {
        setDraft(formatForEdit ? formatForEdit(value) : String(value));
        setEditing(true);
    }, [formatForEdit, value]);

    const commit = useCallback(() => {
        setEditing(false);
        const parsed = parseFloat(draft);
        if (Number.isFinite(parsed)) {
            onCommit(parseInput ? parseInput(parsed) : parsed);
        }
    }, [draft, onCommit, parseInput]);

    useEffect(() => {
        if (editing) {
            inputRef.current?.select();
        }
    }, [editing]);

    if (editing) {
        return (
            <input
                ref={inputRef}
                type="text"
                inputMode="decimal"
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onBlur={commit}
                onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                        commit();
                    } else if (e.key === 'Escape') {
                        setEditing(false);
                    }
                }}
                className={className}
                autoFocus
            />
        );
    }

    return (
        <button type="button" onClick={startEditing} className={className}>
            {displayValue}
        </button>
    );
}

export type ReaderSettingsPanelProps = {
    fontSize: ReaderStyleControl;
    lineSpacing: ReaderStyleControl;
    wordSpacing: ReaderStyleControl;
    leftMargin: ReaderStyleControl;
    rightMargin: ReaderStyleControl;
    topMargin: ReaderStyleControl;
    bottomMargin: ReaderStyleControl;
    hyphenation: ReaderModeControl;
    floatingPunctuation: ReaderModeControl;
    embeddedFonts: ReaderToggleControl;
    onResetBookDefaults: () => void;
    canResetBookDefaults: boolean;
    onResetKoShelfDefaults: () => void;
    canResetKoShelfDefaults: boolean;
    hasDistinctBookDefaults: boolean;
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

function formatPercentSettingValue(value: number): string {
    return `${Math.round(value * 100)}%`;
}

function formatPointSettingValue(value: number): string {
    return `${Math.round(value)}pt`;
}

function formatPixelSettingValue(value: number): string {
    return `${Math.round(value)}px`;
}

type ReaderChoiceOption<T extends string | boolean> = {
    value: T;
    label: string;
};

function ReaderSettingsSection({
    title,
    hasOverrides,
    defaultOpen,
    children,
}: {
    title: string;
    hasOverrides: boolean;
    defaultOpen?: boolean;
    children: ReactNode;
}) {
    const [expanded, setExpanded] = useState(defaultOpen ?? false);

    return (
        <div>
            <button
                type="button"
                onClick={() => setExpanded((prev) => !prev)}
                className="flex items-center justify-between w-full py-1.5 group"
            >
                <div className="flex items-center gap-2">
                    <span className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-dark-400 group-hover:text-gray-700 dark:group-hover:text-dark-200 transition-colors">
                        {title}
                    </span>
                    {hasOverrides && !expanded && (
                        <span className="w-1.5 h-1.5 rounded-full bg-primary-400 dark:bg-primary-300" />
                    )}
                </div>
                <LuChevronDown
                    className={`w-3.5 h-3.5 text-gray-400 dark:text-dark-500 transition-transform duration-200 ${expanded ? 'rotate-180' : ''}`}
                    aria-hidden="true"
                />
            </button>
            <div
                className={`grid transition-[grid-template-rows] duration-200 ease-in-out ${expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'}`}
            >
                <div className="overflow-hidden">
                    <div className="space-y-3 pt-2 pb-1">{children}</div>
                </div>
            </div>
        </div>
    );
}

function ReaderSettingControl({
    icon,
    label,
    value,
    decreaseAriaLabel,
    increaseAriaLabel,
    control,
    formatForEdit,
    parseInput,
}: {
    icon: ReactNode;
    label: string;
    value: string;
    decreaseAriaLabel: string;
    increaseAriaLabel: string;
    control: ReaderStyleControl;
    formatForEdit?: (value: number) => string;
    parseInput?: (input: number) => number;
}) {
    const commitValue = useCallback(
        (raw: number) => control.stepBy(raw - control.value),
        [control],
    );

    return (
        <div className="space-y-3">
            <div className="flex items-center gap-2 text-gray-900 dark:text-white">
                {icon}
                <span className="text-sm font-semibold">{label}</span>
                {control.isOverridden && (
                    <span className="w-1.5 h-1.5 rounded-full bg-primary-400 dark:bg-primary-300" />
                )}
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

                <EditableValue
                    value={control.value}
                    displayValue={value}
                    onCommit={commitValue}
                    formatForEdit={formatForEdit}
                    parseInput={parseInput}
                    className="flex-1 px-3 py-2 rounded-lg border border-gray-200/80 dark:border-dark-700/70 bg-white/85 dark:bg-dark-900/70 text-center text-sm font-medium text-gray-700 dark:text-dark-200 tabular-nums focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                />

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
    isOverridden,
}: {
    icon?: ReactNode;
    label: string;
    value: T;
    options: readonly ReaderChoiceOption<T>[];
    onSelect: (nextValue: T) => void;
    isOverridden?: boolean;
}) {
    return (
        <div className="space-y-2">
            <div className="flex items-center gap-2 text-gray-900 dark:text-white">
                {icon}
                <span className="text-sm font-semibold">{label}</span>
                {isOverridden && (
                    <span className="w-1.5 h-1.5 rounded-full bg-primary-400 dark:bg-primary-300" />
                )}
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

function ReaderCompactControl({
    label,
    value,
    decreaseAriaLabel,
    increaseAriaLabel,
    control,
}: {
    label: string;
    value: string;
    decreaseAriaLabel: string;
    increaseAriaLabel: string;
    control: ReaderStyleControl;
}) {
    const commitValue = useCallback(
        (raw: number) => control.stepBy(raw - control.value),
        [control],
    );

    return (
        <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5">
                <span className="text-sm font-medium text-gray-700 dark:text-dark-200">
                    {label}
                </span>
                {control.isOverridden && (
                    <span className="w-1.5 h-1.5 rounded-full bg-primary-400 dark:bg-primary-300" />
                )}
            </div>
            <div className="flex items-center gap-1.5">
                <button
                    type="button"
                    onClick={control.decrease}
                    className={COMPACT_CONTROL_BUTTON_CLASS}
                    aria-label={decreaseAriaLabel}
                >
                    <LuMinus className="w-3.5 h-3.5" aria-hidden="true" />
                </button>
                <EditableValue
                    value={control.value}
                    displayValue={value}
                    onCommit={commitValue}
                    className="w-11 text-center text-xs font-medium text-gray-700 dark:text-dark-200 tabular-nums rounded-md bg-transparent focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                />
                <button
                    type="button"
                    onClick={control.increase}
                    className={COMPACT_CONTROL_BUTTON_CLASS}
                    aria-label={increaseAriaLabel}
                >
                    <LuPlus className="w-3.5 h-3.5" aria-hidden="true" />
                </button>
            </div>
        </div>
    );
}

export function ReaderSettingsPanel({
    fontSize,
    lineSpacing,
    wordSpacing,
    leftMargin,
    rightMargin,
    topMargin,
    bottomMargin,
    hyphenation,
    floatingPunctuation,
    embeddedFonts,
    onResetBookDefaults,
    canResetBookDefaults,
    onResetKoShelfDefaults,
    canResetKoShelfDefaults,
    hasDistinctBookDefaults,
}: ReaderSettingsPanelProps) {
    const [open, setOpen] = useState(false);
    const buttonRef = useRef<HTMLButtonElement>(null);
    const panelRef = useRef<HTMLDivElement>(null);
    const [position, setPosition] = useState<{
        top: number;
        left: number;
    } | null>(null);
    const panelId = useId();

    const close = useCallback(() => setOpen(false), []);
    const updatePosition = useCallback(() => {
        const button = buttonRef.current;
        const panel = panelRef.current;
        if (!button || !panel) {
            return;
        }

        const result = computeOverlayPosition(
            button.getBoundingClientRect(),
            panel.getBoundingClientRect(),
            window.innerWidth,
            window.innerHeight,
            {
                placements: ['bottom', 'top'],
                alignment: 'end',
                arrowSize: 0,
                gap: PANEL_OFFSET_PX,
            },
        );
        setPosition({ top: result.top, left: result.left });
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
    const displayLineSpacing = formatPercentSettingValue(lineSpacing.value);
    const displayWordSpacing = `${Math.round(wordSpacing.value)}%`;
    const displayLeftMargin = formatPixelSettingValue(leftMargin.value);
    const displayRightMargin = formatPixelSettingValue(rightMargin.value);
    const displayTopMargin = formatPixelSettingValue(topMargin.value);
    const displayBottomMargin = formatPixelSettingValue(bottomMargin.value);

    const readerModeOptions: readonly ReaderChoiceOption<ReaderModeValue>[] = [
        { value: 'auto', label: translation.get('reader-mode.auto') },
        { value: 'on', label: translation.get('reader-mode.on') },
        { value: 'off', label: translation.get('reader-mode.off') },
    ];

    const embeddedFontOptions: readonly ReaderChoiceOption<boolean>[] = [
        { value: true, label: translation.get('reader-mode.on') },
        { value: false, label: translation.get('reader-mode.off') },
    ];

    const marginSettings = [
        {
            key: 'left',
            label: translation.get('reader-left-margin'),
            value: displayLeftMargin,
            decreaseAriaLabel: translation.get(
                'reader-left-margin.decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-left-margin.increase-aria',
            ),
            control: leftMargin,
        },
        {
            key: 'right',
            label: translation.get('reader-right-margin'),
            value: displayRightMargin,
            decreaseAriaLabel: translation.get(
                'reader-right-margin.decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-right-margin.increase-aria',
            ),
            control: rightMargin,
        },
        {
            key: 'top',
            label: translation.get('reader-top-margin'),
            value: displayTopMargin,
            decreaseAriaLabel: translation.get(
                'reader-top-margin.decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-top-margin.increase-aria',
            ),
            control: topMargin,
        },
        {
            key: 'bottom',
            label: translation.get('reader-bottom-margin'),
            value: displayBottomMargin,
            decreaseAriaLabel: translation.get(
                'reader-bottom-margin.decrease-aria',
            ),
            increaseAriaLabel: translation.get(
                'reader-bottom-margin.increase-aria',
            ),
            control: bottomMargin,
        },
    ];

    const hasTypographyOverrides =
        lineSpacing.isOverridden ||
        wordSpacing.isOverridden ||
        hyphenation.isOverridden ||
        floatingPunctuation.isOverridden ||
        embeddedFonts.isOverridden;
    const hasMarginOverrides =
        leftMargin.isOverridden ||
        rightMargin.isOverridden ||
        topMargin.isOverridden ||
        bottomMargin.isOverridden;

    return (
        <>
            <Button
                ref={buttonRef}
                variant="neutral"
                size="icon"
                icon={LuSettings}
                label={translation.get('reader-settings.aria-label')}
                onClick={() => setOpen((prev) => !prev)}
                className={
                    open
                        ? 'bg-gray-200/60 dark:bg-dark-700/60 border-gray-300/80 dark:border-dark-600/70 text-gray-900 dark:text-white'
                        : ''
                }
                aria-expanded={open}
                aria-haspopup="dialog"
                aria-controls={open ? panelId : undefined}
            />

            {open &&
                createPortal(
                    <div
                        id={panelId}
                        ref={panelRef}
                        className="fixed w-80 max-w-[calc(100vw-1.5rem)] max-h-[70vh] overflow-y-auto bg-white/95 dark:bg-dark-900/88 backdrop-blur-xs border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl p-4 z-[100]"
                        style={{
                            top: position?.top ?? 0,
                            left: position?.left ?? 0,
                            visibility: position ? 'visible' : 'hidden',
                        }}
                        role="dialog"
                        aria-label={translation.get(
                            'reader-settings.aria-label',
                        )}
                    >
                        <div className="space-y-2">
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
                                    'reader-font-size.decrease-aria',
                                )}
                                increaseAriaLabel={translation.get(
                                    'reader-font-size.increase-aria',
                                )}
                                control={fontSize}
                            />

                            <hr className="border-gray-200/50 dark:border-dark-700/40" />

                            <ReaderSettingsSection
                                title={translation.get(
                                    'reader-settings.typography',
                                )}
                                hasOverrides={hasTypographyOverrides}
                            >
                                <ReaderSettingControl
                                    icon={
                                        <LuAlignJustify
                                            className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                            aria-hidden="true"
                                        />
                                    }
                                    label={translation.get(
                                        'reader-line-spacing',
                                    )}
                                    value={displayLineSpacing}
                                    decreaseAriaLabel={translation.get(
                                        'reader-line-spacing.decrease-aria',
                                    )}
                                    increaseAriaLabel={translation.get(
                                        'reader-line-spacing.increase-aria',
                                    )}
                                    control={lineSpacing}
                                    formatForEdit={formatLineSpacingForEdit}
                                    parseInput={parseLineSpacingInput}
                                />

                                <ReaderSettingControl
                                    icon={
                                        <LuUnfoldHorizontal
                                            className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                            aria-hidden="true"
                                        />
                                    }
                                    label={translation.get(
                                        'reader-word-spacing',
                                    )}
                                    value={displayWordSpacing}
                                    decreaseAriaLabel={translation.get(
                                        'reader-word-spacing.decrease-aria',
                                    )}
                                    increaseAriaLabel={translation.get(
                                        'reader-word-spacing.increase-aria',
                                    )}
                                    control={wordSpacing}
                                />

                                <ReaderChoiceControl
                                    icon={
                                        <LuType
                                            className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                            aria-hidden="true"
                                        />
                                    }
                                    label={translation.get(
                                        'reader-embedded-fonts',
                                    )}
                                    value={embeddedFonts.value}
                                    options={embeddedFontOptions}
                                    onSelect={embeddedFonts.setValue}
                                    isOverridden={embeddedFonts.isOverridden}
                                />

                                <ReaderChoiceControl
                                    icon={
                                        <LuWrapText
                                            className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                            aria-hidden="true"
                                        />
                                    }
                                    label={translation.get(
                                        'reader-hyphenation',
                                    )}
                                    value={hyphenation.value}
                                    options={readerModeOptions}
                                    onSelect={hyphenation.setValue}
                                    isOverridden={hyphenation.isOverridden}
                                />

                                <ReaderChoiceControl
                                    icon={
                                        <LuQuote
                                            className="w-4 h-4 text-primary-500 dark:text-primary-300"
                                            aria-hidden="true"
                                        />
                                    }
                                    label={translation.get(
                                        'reader-floating-punctuation',
                                    )}
                                    value={floatingPunctuation.value}
                                    options={readerModeOptions}
                                    onSelect={floatingPunctuation.setValue}
                                    isOverridden={
                                        floatingPunctuation.isOverridden
                                    }
                                />
                            </ReaderSettingsSection>

                            <hr className="border-gray-200/50 dark:border-dark-700/40" />

                            <ReaderSettingsSection
                                title={translation.get(
                                    'reader-settings.margins',
                                )}
                                hasOverrides={hasMarginOverrides}
                            >
                                {marginSettings.map((setting) => (
                                    <ReaderCompactControl
                                        key={setting.key}
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
                            </ReaderSettingsSection>

                            <hr className="border-gray-200/50 dark:border-dark-700/40" />

                            <div className="space-y-2">
                                {hasDistinctBookDefaults && (
                                    <button
                                        type="button"
                                        onClick={onResetBookDefaults}
                                        disabled={!canResetBookDefaults}
                                        className="w-full px-3 py-2.5 rounded-lg border border-gray-300/70 dark:border-dark-700/70 bg-white/85 dark:bg-dark-900/70 text-sm font-medium text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                                        aria-label={translation.get(
                                            'reader-reset.book-aria',
                                        )}
                                    >
                                        {translation.get('reader-reset.book')}
                                    </button>
                                )}
                                <button
                                    type="button"
                                    onClick={onResetKoShelfDefaults}
                                    disabled={!canResetKoShelfDefaults}
                                    className="w-full px-3 py-2.5 rounded-lg border border-gray-300/70 dark:border-dark-700/70 bg-white/85 dark:bg-dark-900/70 text-sm font-medium text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                                    aria-label={translation.get(
                                        'reader-reset.defaults-aria',
                                    )}
                                >
                                    {translation.get('reader-reset.defaults')}
                                </button>
                            </div>
                        </div>
                    </div>,
                    document.body,
                )}
        </>
    );
}
