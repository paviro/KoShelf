import {
    useCallback,
    useEffect,
    useMemo,
    useState,
    type DragEvent,
} from 'react';
import type { IconType } from 'react-icons';
import { HiOutlineBookOpen } from 'react-icons/hi2';
import {
    LuBan,
    LuBookOpen,
    LuCheck,
    LuFileText,
    LuImage,
    LuImageUp,
    LuLoaderCircle,
    LuPencil,
    LuRotateCcw,
} from 'react-icons/lu';

import type { MetadataField } from '../../../shared/api-client';
import type { LibraryDetailItem } from '../../../shared/contracts';
import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';
import { ModalShell } from '../../../shared/ui/modal/ModalShell';
import { CacheNoticeModal } from './CacheNoticeModal';
import { useCustomEdit } from '../api/library-mutations';
import { useCacheNotice } from '../hooks/useCacheNotice';
import { useEditSnapshot } from '../hooks/useEditSnapshot';
import { usePendingEdit } from '../hooks/usePendingEdit';
import {
    buildMetadataPatch,
    canEditCustom,
    isMetadataOverride,
    isValidSeriesIndex,
} from '../lib/custom-edit';
import type { LibraryCollection } from '../model/library-model';

type Tab = 'status' | 'metadata' | 'cover';
/** `undefined` keeps the current cover, `null` restores the original. */
type CoverDraft = File | null | undefined;

const TABS: ReadonlyArray<{ value: Tab; labelKey: string; icon: IconType }> = [
    { value: 'status', labelKey: 'custom-tab-status', icon: LuBookOpen },
    { value: 'metadata', labelKey: 'custom-metadata', icon: LuFileText },
    { value: 'cover', labelKey: 'custom-cover', icon: LuImage },
];

const STATUS_OPTIONS: ReadonlyArray<{
    value: string;
    labelKey: string;
    active: string;
    iconContainer: string;
    iconClassName: string;
    icon: IconType;
}> = [
    {
        value: 'reading',
        labelKey: 'status.reading-short',
        active: 'bg-primary-50 dark:bg-primary-500/10 border-primary-300 dark:border-primary-500/30 text-primary-700 dark:text-primary-300',
        iconContainer:
            'bg-primary-500/20 dark:bg-linear-to-br dark:from-primary-500 dark:to-primary-600',
        iconClassName: 'text-primary-600 dark:text-white',
        icon: HiOutlineBookOpen,
    },
    {
        value: 'complete',
        labelKey: 'status.completed-short',
        active: 'bg-green-50 dark:bg-green-500/10 border-green-300 dark:border-green-500/30 text-green-700 dark:text-green-300',
        iconContainer:
            'bg-green-500/20 dark:bg-linear-to-br dark:from-green-500 dark:to-green-600',
        iconClassName: 'text-green-600 dark:text-white',
        icon: LuCheck,
    },
    {
        value: 'abandoned',
        labelKey: 'status.abandoned',
        active: 'bg-red-50 dark:bg-red-500/10 border-red-300 dark:border-red-500/30 text-red-700 dark:text-red-300',
        iconContainer:
            'bg-red-500/20 dark:bg-linear-to-br dark:from-red-500 dark:to-red-600',
        iconClassName: 'text-red-600 dark:text-white',
        icon: LuBan,
    },
];

/** Metadata form layout on a six-column grid; `rows` > 1 renders a textarea. */
const FIELDS: ReadonlyArray<{
    field: MetadataField;
    labelKey: string;
    span: string;
    rows: number;
}> = [
    {
        field: 'title',
        labelKey: 'custom-field-title',
        span: 'sm:col-span-6',
        rows: 1,
    },
    {
        field: 'series',
        labelKey: 'custom-field-series',
        span: 'sm:col-span-2',
        rows: 1,
    },
    {
        field: 'series_index',
        labelKey: 'custom-field-series_index',
        span: 'sm:col-span-2',
        rows: 1,
    },
    {
        field: 'language',
        labelKey: 'custom-field-language',
        span: 'sm:col-span-2',
        rows: 1,
    },
    {
        field: 'authors',
        labelKey: 'custom-field-authors',
        span: 'sm:col-span-3',
        rows: 3,
    },
    {
        field: 'keywords',
        labelKey: 'custom-field-keywords',
        span: 'sm:col-span-3',
        rows: 3,
    },
    {
        field: 'description',
        labelKey: 'custom-field-description',
        span: 'sm:col-span-6',
        rows: 7,
    },
];

const REASON_LABELS: Record<string, string> = {
    writeback_disabled: 'custom-reason-writeback_disabled',
    archived: 'custom-reason-archived',
    inaccessible_custom_files: 'custom-reason-inaccessible_custom_files',
};

const COVER_ACCEPT =
    '.jpg,.jpeg,.png,.gif,.webp,image/jpeg,image/png,image/gif,image/webp';

/** Default narrow width of the status tab, matching other small modals. */
const STATUS_WIDTH = 448;
const WIDE_WIDTH = '56rem';
/** Horizontal padding of the header around the tab bar (`px-6`). */
const HEADER_PADDING = 48;

const TAB_CLASS =
    'flex flex-1 items-center justify-center gap-1.5 px-3 py-1.5 rounded-md whitespace-nowrap text-sm font-medium';

// `pr-9` keeps room for the inline reset button so text never reflows.
const INPUT_CLASS =
    'w-full rounded-lg border bg-gray-50 dark:bg-dark-800 text-gray-900 dark:text-white pl-3 pr-9 py-2 text-sm leading-relaxed transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-500 disabled:opacity-60';

type LibraryEditModalProps = {
    open: boolean;
    onClose: () => void;
    item: LibraryDetailItem;
    collection: LibraryCollection;
    canChangeStatus: boolean;
    onStatusChange?: (status: string) => void;
};

export function LibraryEditModal({
    open,
    onClose,
    item,
    collection,
    canChangeStatus,
    onStatusChange,
}: LibraryEditModalProps) {
    const edit = item.custom_edit ?? null;
    const { value: snapshot, stale } = useEditSnapshot(
        edit,
        edit ? `${edit.metadata.revision}:${edit.cover.revision}` : null,
        open,
    );
    const [selectedTab, setSelectedTab] = useState<Tab | null>(null);
    /** Typed text per touched field; override state is derived from it. */
    const [drafts, setDrafts] = useState<
        Partial<Record<MetadataField, string>>
    >({});
    const [cover, setCover] = useState<CoverDraft>(undefined);
    const [dragging, setDragging] = useState(false);
    const [tabsWidth, setTabsWidth] = useState(0);
    const [contentHeight, setContentHeight] = useState<number>();
    const { pending, run } = usePendingEdit();
    const notice = useCacheNotice();
    const mutation = useCustomEdit(item.id, collection);

    // Measure the tab labels at their natural size so the status tab can grow
    // just enough for longer translations instead of wrapping.
    // Track the content's natural height so tab switches can animate the card;
    // an `auto` height cannot transition.
    const measureContent = useCallback((element: HTMLDivElement | null) => {
        if (!element) return;
        const measure = () => setContentHeight(element.offsetHeight);
        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(element);
        return () => observer.disconnect();
    }, []);

    const measureTabs = useCallback((element: HTMLDivElement | null) => {
        if (!element) return;
        const measure = () => setTabsWidth(element.offsetWidth);
        measure();
        const observer = new ResizeObserver(measure);
        observer.observe(element);
        return () => observer.disconnect();
    }, []);

    const preview = useMemo(
        () => (cover ? URL.createObjectURL(cover) : null),
        [cover],
    );
    useEffect(
        () => () => {
            if (preview) URL.revokeObjectURL(preview);
        },
        [preview],
    );

    const customEditable = canEditCustom(snapshot);
    const tabs = TABS.filter(({ value }) =>
        value === 'status' ? canChangeStatus : customEditable,
    );
    const tab =
        tabs.find(({ value }) => value === selectedTab)?.value ??
        tabs[0]?.value;

    const close = () => {
        if (pending) return;
        setSelectedTab(null);
        setDrafts({});
        setCover(undefined);
        onClose();
    };

    const capability =
        snapshot && tab !== 'status' && tab ? snapshot[tab] : null;
    const locked = pending || stale || !capability?.available;
    const baseText = (field: MetadataField) => snapshot?.base[field] ?? '';
    const fieldText = (field: MetadataField) =>
        drafts[field] ?? snapshot?.overrides[field] ?? baseText(field);
    const isOverridden = (field: MetadataField) =>
        isMetadataOverride(field, fieldText(field), baseText(field));
    const patch = snapshot ? buildMetadataPatch(snapshot, drafts) : {};
    // Fields that will still carry an override after saving the draft.
    const overriddenFields = FIELDS.filter(({ field }) => isOverridden(field));
    const metadataResetPending =
        Object.keys(patch).length > 0 && overriddenFields.length === 0;
    const resetField = (field: MetadataField) =>
        setDrafts((d) => ({ ...d, [field]: baseText(field) }));
    const resetMetadata = () =>
        setDrafts(
            Object.fromEntries(
                FIELDS.map(({ field }) => [field, baseText(field)]),
            ),
        );
    // Both tabs' drafts are saved together, so each pending part needs its own
    // capability rather than the active tab's.
    const metadataChanged = Object.keys(patch).length > 0;
    const seriesIndexInvalid = !isValidSeriesIndex(fieldText('series_index'));
    const coverChanged = cover !== undefined;
    const canSave =
        !pending &&
        !stale &&
        !!snapshot &&
        (metadataChanged || coverChanged) &&
        (!metadataChanged ||
            (snapshot.metadata.available && !seriesIndexInvalid)) &&
        (!coverChanged || snapshot.cover.available);

    const save = () => {
        if (!snapshot || !canSave) return;
        void run(
            () =>
                mutation.mutateAsync({
                    ...(metadataChanged && {
                        metadata: {
                            patch,
                            revision: snapshot.metadata.revision,
                        },
                    }),
                    ...(cover !== undefined && {
                        cover: {
                            file: cover,
                            revision: snapshot.cover.revision,
                        },
                    }),
                }),
            () => {
                close();
                notice.show();
            },
        );
    };

    const selectStatus = (status: string) => {
        if (status !== item.status) onStatusChange?.(status);
        close();
    };

    const dropCover = (event: DragEvent<HTMLLabelElement>) => {
        event.preventDefault();
        setDragging(false);
        const file = event.dataTransfer.files[0];
        if (!locked && file?.type.startsWith('image/')) setCover(file);
    };

    return (
        <>
            <ModalShell
                open={open}
                onClose={close}
                closeDisabled={pending}
                containerClassName="z-[60]"
                // Animate between explicit max-widths; `auto` widths cannot transition.
                cardStyle={{
                    height: contentHeight,
                    maxWidth:
                        tab === 'status'
                            ? Math.max(STATUS_WIDTH, tabsWidth + HEADER_PADDING)
                            : WIDE_WIDTH,
                }}
                cardClassName={`bg-white/95 dark:bg-dark-900/95 border border-gray-200/70 dark:border-dark-600/50 rounded-2xl shadow-2xl overflow-hidden`}
            >
                <div
                    ref={measureContent}
                    className="flex flex-col max-h-[92vh]"
                >
                    <header className="shrink-0 flex flex-col gap-4 px-6 pt-6 pb-4 border-b border-gray-200/50 dark:border-dark-700/50">
                        <div className="flex items-center gap-3 min-w-0 pr-8">
                            <div className="w-10 h-10 rounded-full bg-primary-100 dark:bg-primary-500/20 flex items-center justify-center shrink-0">
                                <LuPencil className="w-5 h-5 text-primary-600 dark:text-primary-400" />
                            </div>
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                                {translation.get('custom-edit-title')}
                            </h2>
                        </div>

                        {tabs.length > 1 && (
                            <div
                                role="tablist"
                                className="flex overflow-x-auto rounded-lg bg-gray-100 dark:bg-dark-800 p-1"
                            >
                                {tabs.map(({ value, labelKey, icon: Icon }) => {
                                    const active = value === tab;
                                    return (
                                        <button
                                            key={value}
                                            type="button"
                                            role="tab"
                                            aria-selected={active}
                                            aria-label={translation.get(
                                                labelKey,
                                            )}
                                            title={translation.get(labelKey)}
                                            disabled={pending}
                                            onClick={() =>
                                                setSelectedTab(value)
                                            }
                                            className={`${TAB_CLASS} transition-colors ${
                                                active
                                                    ? 'bg-white dark:bg-dark-700 text-gray-900 dark:text-white shadow-xs'
                                                    : 'text-gray-500 dark:text-dark-400 hover:text-gray-900 dark:hover:text-white'
                                            }`}
                                        >
                                            <Icon
                                                className="w-4 h-4"
                                                aria-hidden="true"
                                            />
                                            {/* Icon-only on small screens. */}
                                            <span className="hidden sm:inline">
                                                {translation.get(labelKey)}
                                            </span>
                                        </button>
                                    );
                                })}
                            </div>
                        )}
                        {tabs.length > 1 && (
                            <div
                                ref={measureTabs}
                                aria-hidden="true"
                                className="invisible absolute left-0 top-0 flex w-max p-1 pointer-events-none"
                            >
                                {tabs.map(({ value, labelKey, icon: Icon }) => (
                                    <span key={value} className={TAB_CLASS}>
                                        <Icon className="w-4 h-4" />
                                        {translation.get(labelKey)}
                                    </span>
                                ))}
                            </div>
                        )}
                    </header>

                    <div className="min-h-0 overflow-y-auto px-6 py-5 space-y-4">
                        {tab !== 'status' && stale && (
                            <p
                                role="alert"
                                className="rounded-lg bg-amber-50 dark:bg-amber-500/10 border border-amber-200 dark:border-amber-500/30 px-4 py-3 text-sm font-medium text-amber-700 dark:text-amber-300"
                            >
                                {translation.get('custom-edit-stale')}
                            </p>
                        )}

                        {capability && !capability.available && (
                            <div
                                role="status"
                                className="rounded-lg bg-gray-50 dark:bg-dark-800 border border-gray-200/70 dark:border-dark-700/70 px-4 py-3 text-sm font-medium text-gray-600 dark:text-dark-300"
                            >
                                <p>
                                    {translation.get('custom-edit-unavailable')}
                                    :{' '}
                                    {capability.reasons
                                        .map((reason) =>
                                            translation.get(
                                                REASON_LABELS[reason] ??
                                                    'custom-edit-unavailable',
                                            ),
                                        )
                                        .join(', ')}
                                </p>
                            </div>
                        )}

                        {tab === 'status' && (
                            <div className="flex flex-col gap-2">
                                {STATUS_OPTIONS.map((opt) => {
                                    const isActive = item.status === opt.value;
                                    return (
                                        <button
                                            key={opt.value}
                                            type="button"
                                            onClick={() =>
                                                selectStatus(opt.value)
                                            }
                                            aria-pressed={isActive}
                                            className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-medium border transition-all duration-200 ${
                                                isActive
                                                    ? opt.active
                                                    : 'bg-gray-50 dark:bg-dark-800 border-gray-200 dark:border-dark-700 text-gray-700 dark:text-dark-300 hover:bg-gray-100 dark:hover:bg-dark-700'
                                            }`}
                                        >
                                            <div
                                                className={`w-8 h-8 rounded-lg flex items-center justify-center shrink-0 ${opt.iconContainer}`}
                                            >
                                                <opt.icon
                                                    className={`w-5 h-5 ${opt.iconClassName}`}
                                                    aria-hidden="true"
                                                />
                                            </div>
                                            <span className="flex-1 text-left">
                                                {translation.get(opt.labelKey)}
                                            </span>
                                            {isActive && (
                                                <LuCheck
                                                    className="w-4 h-4 shrink-0"
                                                    aria-hidden="true"
                                                />
                                            )}
                                        </button>
                                    );
                                })}
                            </div>
                        )}

                        {tab === 'metadata' && snapshot && (
                            <div className="grid grid-cols-1 sm:grid-cols-6 gap-x-4 gap-y-4">
                                {FIELDS.map(
                                    ({ field, labelKey, span, rows }) => {
                                        const overridden = isOverridden(field);
                                        const value = fieldText(field);
                                        const invalid =
                                            field === 'series_index' &&
                                            seriesIndexInvalid;
                                        const inputProps = {
                                            id: `custom-${field}`,
                                            className: `${INPUT_CLASS} ${
                                                invalid
                                                    ? 'border-red-400 dark:border-red-500/60'
                                                    : overridden
                                                      ? 'border-primary-300 dark:border-primary-500/50'
                                                      : 'border-gray-200 dark:border-dark-700'
                                            }`,
                                            disabled: locked,
                                            'aria-invalid':
                                                invalid || undefined,
                                            value,
                                            onChange: (event: {
                                                target: { value: string };
                                            }) =>
                                                setDrafts((d) => ({
                                                    ...d,
                                                    [field]: event.target.value,
                                                })),
                                        };
                                        return (
                                            <div key={field} className={span}>
                                                <label
                                                    htmlFor={`custom-${field}`}
                                                    className="flex items-center gap-1.5 mb-1.5 text-sm font-medium text-gray-700 dark:text-dark-200"
                                                >
                                                    {translation.get(labelKey)}
                                                    {overridden && (
                                                        <span
                                                            className="w-1.5 h-1.5 rounded-full bg-primary-500 shrink-0"
                                                            title={translation.get(
                                                                'custom-override',
                                                            )}
                                                        >
                                                            <span className="sr-only">
                                                                {translation.get(
                                                                    'custom-override',
                                                                )}
                                                            </span>
                                                        </span>
                                                    )}
                                                </label>
                                                <div className="relative">
                                                    {rows > 1 ? (
                                                        <textarea
                                                            {...inputProps}
                                                            rows={rows}
                                                            className={`${inputProps.className} resize-y block`}
                                                        />
                                                    ) : (
                                                        <input
                                                            type="text"
                                                            inputMode={
                                                                field ===
                                                                'series_index'
                                                                    ? 'decimal'
                                                                    : undefined
                                                            }
                                                            {...inputProps}
                                                        />
                                                    )}
                                                    {overridden && !locked && (
                                                        <button
                                                            type="button"
                                                            aria-label={translation.get(
                                                                'custom-reset',
                                                            )}
                                                            title={translation.get(
                                                                'custom-reset',
                                                            )}
                                                            onClick={() =>
                                                                resetField(
                                                                    field,
                                                                )
                                                            }
                                                            className="absolute top-1.5 right-1.5 p-1 rounded-md text-gray-400 dark:text-dark-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-gray-200/60 dark:hover:bg-dark-700 transition-colors"
                                                        >
                                                            <LuRotateCcw
                                                                className="w-4 h-4"
                                                                aria-hidden="true"
                                                            />
                                                        </button>
                                                    )}
                                                </div>
                                            </div>
                                        );
                                    },
                                )}
                                <p className="sm:col-span-6 text-xs text-gray-500 dark:text-dark-400">
                                    {seriesIndexInvalid
                                        ? translation.get(
                                              'custom-series-index-invalid',
                                          )
                                        : metadataResetPending
                                          ? translation.get(
                                                'custom-metadata-reset-pending',
                                            )
                                          : translation.get(
                                                'custom-lists-help',
                                            )}
                                </p>
                            </div>
                        )}

                        {tab === 'cover' && (
                            <div className="flex flex-col sm:flex-row gap-6">
                                <div className="w-48 sm:w-56 shrink-0 mx-auto sm:mx-0">
                                    <div className="rounded-xl overflow-hidden border border-gray-200/70 dark:border-dark-700/70 bg-gray-50 dark:bg-dark-800 shadow-lg dark:shadow-none">
                                        <img
                                            src={preview ?? item.cover_url}
                                            className={`w-full h-auto transition-opacity ${cover === null ? 'opacity-40' : ''}`}
                                            alt={translation.get(
                                                'custom-cover-preview',
                                            )}
                                        />
                                    </div>
                                </div>
                                <div className="flex-1 flex flex-col gap-3">
                                    <label
                                        onDragOver={(event) => {
                                            event.preventDefault();
                                            if (!locked) setDragging(true);
                                        }}
                                        onDragLeave={() => setDragging(false)}
                                        onDrop={dropCover}
                                        className={`flex-1 min-h-40 flex flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed px-6 py-8 text-center transition-colors ${
                                            locked
                                                ? 'opacity-60 cursor-not-allowed border-gray-200 dark:border-dark-700'
                                                : dragging
                                                  ? 'cursor-pointer border-primary-500 bg-primary-50 dark:bg-primary-500/10'
                                                  : 'cursor-pointer border-gray-300 dark:border-dark-600 hover:border-primary-400 hover:bg-gray-50 dark:hover:bg-dark-800'
                                        }`}
                                    >
                                        <LuImageUp
                                            className="w-8 h-8 text-gray-400 dark:text-dark-400"
                                            aria-hidden="true"
                                        />
                                        <span className="text-sm font-medium text-gray-700 dark:text-dark-200">
                                            {cover
                                                ? cover.name
                                                : translation.get(
                                                      'custom-cover-choose',
                                                  )}
                                        </span>
                                        <span className="text-xs text-gray-500 dark:text-dark-400">
                                            JPG, PNG, GIF, WebP
                                        </span>
                                        <input
                                            type="file"
                                            className="sr-only"
                                            aria-label={translation.get(
                                                'custom-cover',
                                            )}
                                            accept={COVER_ACCEPT}
                                            disabled={locked}
                                            onChange={(event) => {
                                                const file =
                                                    event.target.files?.[0];
                                                if (file) setCover(file);
                                                event.target.value = '';
                                            }}
                                        />
                                    </label>
                                    {cover === null && (
                                        <p className="text-sm text-gray-500 dark:text-dark-400">
                                            {translation.get(
                                                'custom-cover-reset-pending',
                                            )}
                                        </p>
                                    )}
                                </div>
                            </div>
                        )}

                        {tab !== 'status' &&
                            snapshot &&
                            snapshot.issues.length > 0 && (
                                <details className="text-xs text-gray-500 dark:text-dark-400">
                                    <summary className="cursor-pointer select-none font-medium">
                                        {translation.get('custom-issues')}
                                    </summary>
                                    <ul className="mt-2 list-disc pl-5 break-all">
                                        {snapshot.issues.map((issue) => (
                                            <li key={issue}>{issue}</li>
                                        ))}
                                    </ul>
                                </details>
                            )}
                    </div>

                    {tab !== 'status' && (
                        <footer className="shrink-0 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 px-6 py-3 border-t border-gray-200/50 dark:border-dark-700/50 bg-gray-50/50 dark:bg-dark-900/30">
                            <div className="flex flex-col sm:block">
                                {tab === 'metadata' && (
                                    <Button
                                        variant="neutral"
                                        icon={LuRotateCcw}
                                        disabled={
                                            locked ||
                                            overriddenFields.length === 0
                                        }
                                        onClick={resetMetadata}
                                    >
                                        {translation.get(
                                            'custom-reset-original',
                                        )}
                                    </Button>
                                )}
                                {tab === 'cover' && (
                                    <Button
                                        variant="neutral"
                                        icon={LuRotateCcw}
                                        disabled={
                                            locked ||
                                            cover === null ||
                                            (cover === undefined &&
                                                !snapshot?.has_custom_cover)
                                        }
                                        // Without a custom cover, restoring only
                                        // discards the selected file.
                                        onClick={() =>
                                            setCover(
                                                snapshot?.has_custom_cover
                                                    ? null
                                                    : undefined,
                                            )
                                        }
                                    >
                                        {translation.get(
                                            'custom-reset-original',
                                        )}
                                    </Button>
                                )}
                            </div>
                            {/* Stacked full-width on small screens, inline on larger ones. */}
                            <div className="flex items-center gap-2">
                                <Button
                                    color="secondary"
                                    className="flex-1 sm:flex-none"
                                    disabled={pending}
                                    onClick={close}
                                >
                                    {translation.get('cancel')}
                                </Button>
                                <Button
                                    className="flex-1 sm:flex-none"
                                    disabled={!canSave}
                                    aria-busy={pending}
                                    onClick={save}
                                >
                                    {pending && (
                                        <LuLoaderCircle
                                            className="w-4 h-4 animate-spin"
                                            aria-hidden="true"
                                        />
                                    )}
                                    <span role="status" aria-live="polite">
                                        {translation.get(
                                            pending ? 'custom-saving' : 'save',
                                        )}
                                    </span>
                                </Button>
                            </div>
                        </footer>
                    )}
                </div>
            </ModalShell>
            <CacheNoticeModal open={notice.noticeOpen} onClose={notice.close} />
        </>
    );
}
