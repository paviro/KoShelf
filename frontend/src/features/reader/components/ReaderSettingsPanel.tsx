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
import {
    DEFAULT_READER_FONT_SIZE,
    DEFAULT_READER_LINE_SPACING,
} from '../lib/reader-theme';

const HEADER_ICON_BUTTON_CLASS =
    'flex items-center justify-center w-10 h-10 p-2.5 bg-gray-100/50 dark:bg-dark-800/10 border border-gray-300/50 dark:border-dark-700/50 rounded-lg text-gray-600 dark:text-gray-300 hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors duration-200 backdrop-blur-xs focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const PANEL_CONTROL_BUTTON_CLASS =
    'flex items-center justify-center w-9 h-9 rounded-lg border border-gray-300/60 dark:border-dark-700/60 bg-white/80 dark:bg-dark-900/60 text-gray-600 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/70 transition-colors duration-200 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50';

const PANEL_OFFSET_PX = 8;

type ReaderSettingsPanelProps = {
    fontSize: number;
    onFontIncrease: () => void;
    onFontDecrease: () => void;
    lineSpacing: number;
    onLineSpacingIncrease: () => void;
    onLineSpacingDecrease: () => void;
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

type ReaderSettingControlProps = {
    icon: ReactNode;
    label: string;
    value: string;
    decreaseAriaLabel: string;
    increaseAriaLabel: string;
    onDecrease: () => void;
    onIncrease: () => void;
};

function ReaderSettingControl({
    icon,
    label,
    value,
    decreaseAriaLabel,
    increaseAriaLabel,
    onDecrease,
    onIncrease,
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
                    onClick={onDecrease}
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
                    onClick={onIncrease}
                    className={PANEL_CONTROL_BUTTON_CLASS}
                    aria-label={increaseAriaLabel}
                >
                    <LuPlus className="w-4 h-4" aria-hidden="true" />
                </button>
            </div>
        </div>
    );
}

export function ReaderSettingsPanel({
    fontSize,
    onFontIncrease,
    onFontDecrease,
    lineSpacing,
    onLineSpacingIncrease,
    onLineSpacingDecrease,
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

    const displayScale = formatRelativeSettingValue(
        fontSize,
        DEFAULT_READER_FONT_SIZE,
    );
    const displayLineSpacing = formatRelativeSettingValue(
        lineSpacing,
        DEFAULT_READER_LINE_SPACING,
    );

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
                        className="fixed w-72 bg-white/95 dark:bg-dark-900/88 backdrop-blur-xs border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl p-4 z-[100]"
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
                                value={displayScale}
                                decreaseAriaLabel={translation.get(
                                    'reader-font-size-decrease-aria',
                                )}
                                increaseAriaLabel={translation.get(
                                    'reader-font-size-increase-aria',
                                )}
                                onDecrease={onFontDecrease}
                                onIncrease={onFontIncrease}
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
                                onDecrease={onLineSpacingDecrease}
                                onIncrease={onLineSpacingIncrease}
                            />
                        </div>
                    </div>,
                    document.body,
                )}
        </>
    );
}
