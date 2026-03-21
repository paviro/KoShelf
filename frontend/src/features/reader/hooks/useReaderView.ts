import {
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
    type Dispatch,
    type RefObject,
    type SetStateAction,
} from 'react';
import { useNavigate, useParams, useSearchParams } from 'react-router';

import {
    buildRoutePath,
    detailRouteIdForCollection,
} from '../../../app/routes/route-registry';
import { ApiHttpError, api } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { StorageManager } from '../../../shared/storage-manager';
import type { LibraryAnnotation } from '../../library/api/library-data';
import { useLibraryDetailQuery } from '../../library/hooks/useLibraryQueries';
import type { LibraryCollection } from '../../library/model/library-model';
import { attachHighlightDrawListener } from '../lib/reader-highlight-overlay';
import {
    isReaderFormatSupported,
    normalizeLibraryFormat,
} from '../lib/reader-format-support';
import { resolveHighlightsBySection } from '../lib/reader-highlight-resolution';
import { resolveAnnotationTarget } from '../lib/reader-navigation-resolution';
import { parseKoReaderPosition } from '../lib/reader-position-parser';
import { applyReaderPresentation } from '../lib/reader-theme';
import type {
    FoliateView,
    ReaderHighlightValue,
    ReaderLocation,
    TocEntry,
} from '../model/reader-model';

const READER_POSITION_KEY_PREFIX = 'reader_position_';

export type ReaderActiveNote = {
    note: string;
};

export type UseReaderViewResult = {
    containerRef: RefObject<HTMLDivElement | null>;
    loading: boolean;
    error: unknown | null;
    backHref: string;
    title: string;
    chapterLabel: string;
    chapterHref: string | null;
    currentSectionIndex: number | null;
    toc: TocEntry[];
    highlights: LibraryAnnotation[];
    bookmarks: LibraryAnnotation[];
    activeNote: ReaderActiveNote | null;
    dismissNote: () => void;
    goTo: (target: string | number) => void;
    handleBackClick: (event: React.MouseEvent<HTMLAnchorElement>) => void;
    handlePrev: () => void;
    handleNext: () => void;
};

export function useReaderView(
    collection: LibraryCollection,
    viewRef: RefObject<FoliateView | null>,
    setLocation: Dispatch<SetStateAction<ReaderLocation | null>>,
    scrubSettlingRef: RefObject<boolean>,
    setDragFraction: (value: number | null) => void,
    fontSize: number,
    lineSpacing: number,
): UseReaderViewResult {
    const navigate = useNavigate();
    const params = useParams();
    const [searchParams] = useSearchParams();
    const id = params.id;
    const highlightIndex = searchParams.get('highlight');
    const bookmarkIndex = searchParams.get('bookmark');

    const detailQuery = useLibraryDetailQuery(collection, id);
    const item = detailQuery.data?.item;
    const hasItem = Boolean(item);
    const highlights = useMemo(
        () => detailQuery.data?.highlights ?? [],
        [detailQuery.data?.highlights],
    );
    const bookmarks = useMemo(
        () => detailQuery.data?.bookmarks ?? [],
        [detailQuery.data?.bookmarks],
    );

    const fontSizeRef = useRef(fontSize);
    fontSizeRef.current = fontSize;

    const lineSpacingRef = useRef(lineSpacing);
    lineSpacingRef.current = lineSpacing;

    const containerRef = useRef<HTMLDivElement>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<unknown | null>(null);
    const [location, setLocationState] = useState<ReaderLocation | null>(null);
    const [activeNote, setActiveNote] = useState<ReaderActiveNote | null>(null);
    const [toc, setToc] = useState<TocEntry[]>([]);
    const noteMapRef = useRef(new Map<string, ReaderActiveNote>());

    const dismissNote = useCallback(() => setActiveNote(null), []);

    const supportsReader = isReaderFormatSupported(item?.format);
    const normalizedFormat = supportsReader
        ? normalizeLibraryFormat(item?.format)
        : null;
    const fileHref = supportsReader
        ? api.getItemFileHref(id ?? '', item?.format)
        : null;
    const backHref = id
        ? buildRoutePath(detailRouteIdForCollection(collection), { id })
        : `/${collection}`;

    const closeReaderView = useCallback(() => {
        const currentView = viewRef.current;
        viewRef.current = null;
        if (!currentView) {
            return;
        }

        try {
            currentView.close();
        } catch {
            // Ignore cleanup errors.
        }

        if (currentView.isConnected) {
            currentView.remove();
        }
    }, [viewRef]);

    const handleBackClick = useCallback(
        (event: React.MouseEvent<HTMLAnchorElement>) => {
            event.preventDefault();
            closeReaderView();
            navigate(backHref);
        },
        [backHref, closeReaderView, navigate],
    );

    const handlePrev = useCallback(() => {
        viewRef.current?.prev();
    }, [viewRef]);

    const handleNext = useCallback(() => {
        viewRef.current?.next();
    }, [viewRef]);

    const goTo = useCallback(
        (target: string | number) => {
            viewRef.current?.goTo(target);
        },
        [viewRef],
    );

    useEffect(() => {
        const container = containerRef.current;
        if (!container) {
            return;
        }

        if (detailQuery.isError) {
            setError(
                detailQuery.error ?? new Error('Failed to load book details.'),
            );
            setLoading(false);
            return;
        }

        if (hasItem && !supportsReader) {
            setError(
                new Error(
                    normalizedFormat
                        ? `Unsupported format for in-app reader: ${normalizedFormat.toUpperCase()}`
                        : 'Unsupported format for in-app reader.',
                ),
            );
            setLoading(false);
            return;
        }

        if (hasItem && !fileHref) {
            setError(
                new ApiHttpError('/assets/files', 404, {
                    code: 'book_file_unavailable',
                }),
            );
            setLoading(false);
            return;
        }

        if (!fileHref) {
            return;
        }

        let cancelled = false;
        let currentView: FoliateView | null = null;
        let detachHighlightDrawListener: (() => void) | null = null;
        let showAnnotationListener: EventListener | null = null;

        setLoading(true);
        setError(null);
        setActiveNote(null);
        setToc([]);
        noteMapRef.current.clear();

        const initReader = async () => {
            try {
                const [{ View }, { Overlayer }] = await Promise.all([
                    import('@xincmm/foliate-js/view.js'),
                    import('@xincmm/foliate-js/overlayer.js'),
                ]);
                const renderers = {
                    highlight: Overlayer.highlight,
                    underline: Overlayer.underline,
                };

                if (cancelled) {
                    return;
                }

                const response = await fetch(fileHref);
                if (!response.ok) {
                    throw new ApiHttpError(fileHref, response.status, {
                        code:
                            response.status === 404
                                ? 'book_file_unavailable'
                                : undefined,
                        apiMessage: response.statusText.trim() || undefined,
                    });
                }

                if (cancelled) {
                    return;
                }

                const blob = await response.blob();
                const file = new File(
                    [blob],
                    `book.${normalizedFormat ?? 'epub'}`,
                    {
                        type: blob.type,
                    },
                );

                if (!customElements.get('foliate-view')) {
                    customElements.define('foliate-view', View);
                }

                const view = document.createElement(
                    'foliate-view',
                ) as FoliateView;
                view.style.width = '100%';
                view.style.height = '100%';

                const highlightsBySection = new Map<
                    number,
                    ReaderHighlightValue[]
                >();
                const loadedOverlaySections = new Set<number>();
                const addedHighlightValuesBySection = new Map<
                    number,
                    Set<string>
                >();

                const registerNotes = (
                    sectionHighlights: ReaderHighlightValue[],
                ) => {
                    for (const h of sectionHighlights) {
                        if (h.note) {
                            noteMapRef.current.set(h.value, {
                                note: h.note,
                            });
                        }
                    }
                };

                const addHighlightsForSection = async (
                    sectionIndex: number,
                ) => {
                    const sectionHighlights =
                        highlightsBySection.get(sectionIndex);
                    if (!sectionHighlights || sectionHighlights.length === 0) {
                        return;
                    }

                    let addedValues =
                        addedHighlightValuesBySection.get(sectionIndex);
                    if (!addedValues) {
                        addedValues = new Set<string>();
                        addedHighlightValuesBySection.set(
                            sectionIndex,
                            addedValues,
                        );
                    }

                    const pendingHighlights: ReaderHighlightValue[] = [];
                    for (let i = 0; i < sectionHighlights.length; i += 1) {
                        const highlight = sectionHighlights[i];
                        if (addedValues.has(highlight.value)) {
                            continue;
                        }

                        addedValues.add(highlight.value);
                        pendingHighlights.push(highlight);
                    }

                    if (pendingHighlights.length === 0) {
                        return;
                    }

                    await Promise.all(
                        pendingHighlights.map(async (highlight) => {
                            try {
                                await view.addAnnotation(highlight);
                            } catch {
                                addedValues?.delete(highlight.value);
                            }
                        }),
                    );
                };

                const createOverlayListener = ((e: CustomEvent) => {
                    const detail = e.detail as { index?: number } | undefined;
                    if (typeof detail?.index !== 'number') {
                        return;
                    }

                    addedHighlightValuesBySection.delete(detail.index);

                    loadedOverlaySections.add(detail.index);
                    void addHighlightsForSection(detail.index);
                }) as EventListener;

                view.addEventListener('relocate', ((e: CustomEvent) => {
                    const detail = e.detail;
                    const loc: ReaderLocation = {
                        fraction: detail.fraction ?? 0,
                        tocItem: detail.tocItem ?? null,
                        section: detail.section ?? null,
                    };
                    setLocationState(loc);
                    setLocation(loc);
                    setActiveNote(null);
                    if (scrubSettlingRef.current) {
                        scrubSettlingRef.current = false;
                        setDragFraction(null);
                    }
                    const cfi = view.lastLocation?.cfi;
                    if (cfi && id) {
                        StorageManager.setSessionByKey(
                            `${READER_POSITION_KEY_PREFIX}${id}`,
                            cfi,
                        );
                    }
                }) as EventListener);
                detachHighlightDrawListener = attachHighlightDrawListener(
                    view,
                    renderers,
                );

                showAnnotationListener = ((e: CustomEvent) => {
                    const detail = e.detail as { value?: string } | undefined;
                    if (!detail?.value) {
                        setActiveNote(null);
                        return;
                    }

                    setActiveNote(noteMapRef.current.get(detail.value) ?? null);
                }) as EventListener;
                view.addEventListener(
                    'show-annotation',
                    showAnnotationListener,
                );

                view.addEventListener('create-overlay', createOverlayListener);

                container.appendChild(view);
                currentView = view;
                viewRef.current = view;

                await view.open(file);
                setToc(resolveTocEntries(view));
                applyReaderPresentation(
                    view,
                    fontSizeRef.current,
                    lineSpacingRef.current,
                );

                const hasAnnotationTarget =
                    highlightIndex !== null || bookmarkIndex !== null;
                const savedCfi =
                    !hasAnnotationTarget && id
                        ? StorageManager.getSessionByKey<string>(
                              `${READER_POSITION_KEY_PREFIX}${id}`,
                          )
                        : null;

                if (savedCfi) {
                    await view.init({ lastLocation: savedCfi });
                } else {
                    await view.init({ showTextStart: true });
                }

                if (cancelled) {
                    return;
                }

                const prioritySectionIndexes = buildPrioritySectionIndexes(
                    highlights,
                    bookmarks,
                    highlightIndex,
                    bookmarkIndex,
                    loadedOverlaySections,
                );

                const isDark =
                    document.documentElement.classList.contains('dark');

                const resolvedHighlightsPromise = resolveHighlightsBySection(
                    view,
                    highlights,
                    isDark,
                    {
                        prioritizeSectionIndexes: prioritySectionIndexes,
                        onSectionResolved: async (
                            sectionIndex,
                            sectionHighlights,
                        ) => {
                            if (cancelled) {
                                return;
                            }

                            registerNotes(sectionHighlights);
                            highlightsBySection.set(
                                sectionIndex,
                                sectionHighlights,
                            );
                            if (loadedOverlaySections.has(sectionIndex)) {
                                await addHighlightsForSection(sectionIndex);
                            }
                        },
                    },
                );

                await navigateToAnnotation(
                    view,
                    highlights,
                    bookmarks,
                    highlightIndex,
                    bookmarkIndex,
                );

                if (cancelled) {
                    return;
                }

                setLoading(false);

                const resolvedHighlights = await resolvedHighlightsPromise;

                if (cancelled) {
                    return;
                }

                for (const [
                    sectionIndex,
                    sectionHighlights,
                ] of resolvedHighlights) {
                    registerNotes(sectionHighlights);
                    highlightsBySection.set(sectionIndex, sectionHighlights);
                }

                await Promise.all(
                    Array.from(loadedOverlaySections, (sectionIndex) =>
                        addHighlightsForSection(sectionIndex),
                    ),
                );
            } catch (err) {
                if (!cancelled) {
                    setError(
                        err instanceof Error
                            ? err
                            : new Error('Failed to load book'),
                    );
                    setLoading(false);
                }
            }
        };

        void initReader();

        return () => {
            cancelled = true;
            detachHighlightDrawListener?.();
            if (showAnnotationListener && currentView) {
                currentView.removeEventListener(
                    'show-annotation',
                    showAnnotationListener,
                );
            }
            if (currentView === viewRef.current) {
                closeReaderView();
            }
        };
    }, [
        closeReaderView,
        detailQuery.error,
        bookmarks,
        bookmarkIndex,
        detailQuery.isError,
        fileHref,
        hasItem,
        highlightIndex,
        highlights,
        id,
        normalizedFormat,
        scrubSettlingRef,
        setDragFraction,
        setLocation,
        supportsReader,
        viewRef,
    ]);

    const title = item?.title ?? translation.get('reader-title');
    const chapterLabel = location?.tocItem?.label ?? '';
    const chapterHref = location?.tocItem?.href ?? null;
    const currentSectionIndex =
        typeof location?.section?.current === 'number'
            ? location.section.current
            : null;

    return {
        containerRef,
        loading,
        error,
        backHref,
        title,
        chapterLabel,
        chapterHref,
        currentSectionIndex,
        toc,
        highlights,
        bookmarks,
        activeNote,
        dismissNote,
        goTo,
        handleBackClick,
        handlePrev,
        handleNext,
    };
}

function resolveTargetAnnotation(
    highlights: LibraryAnnotation[],
    bookmarks: LibraryAnnotation[],
    highlightIndex: string | null,
    bookmarkIndex: string | null,
): { annotation: LibraryAnnotation; index: number } | null {
    const targetIndex = bookmarkIndex !== null ? bookmarkIndex : highlightIndex;
    const annotations = bookmarkIndex !== null ? bookmarks : highlights;

    if (targetIndex === null) {
        return null;
    }

    const parsedIndex = Number.parseInt(targetIndex, 10);
    if (
        Number.isNaN(parsedIndex) ||
        parsedIndex < 0 ||
        parsedIndex >= annotations.length
    ) {
        return null;
    }

    return { annotation: annotations[parsedIndex], index: parsedIndex };
}

function resolveTocEntries(view: FoliateView): TocEntry[] {
    const result: TocEntry[] = [];

    const visit = (
        entries: NonNullable<NonNullable<FoliateView['book']>['toc']>,
        depth: number,
    ) => {
        for (const entry of entries) {
            const href =
                typeof entry.href === 'string' ? entry.href.trim() : '';

            if (href !== '') {
                const label =
                    typeof entry.label === 'string' && entry.label.trim() !== ''
                        ? entry.label.trim()
                        : href;
                const sectionIndex = view.resolveNavigation(href)?.index;

                result.push(
                    typeof sectionIndex === 'number'
                        ? { href, label, sectionIndex, depth }
                        : { href, label, depth },
                );
            }

            const subitems = entry.subitems;
            if (Array.isArray(subitems) && subitems.length > 0) {
                visit(subitems, depth + 1);
            }
        }
    };

    const toc = view.book?.toc;
    if (Array.isArray(toc) && toc.length > 0) {
        visit(toc, 0);
    }

    return result;
}

async function navigateToAnnotation(
    view: FoliateView,
    highlights: LibraryAnnotation[],
    bookmarks: LibraryAnnotation[],
    highlightIndex: string | null,
    bookmarkIndex: string | null,
) {
    const resolved = resolveTargetAnnotation(
        highlights,
        bookmarks,
        highlightIndex,
        bookmarkIndex,
    );
    if (!resolved) {
        return;
    }

    const target = await resolveAnnotationTarget(view, resolved.annotation);
    if (target === null) {
        return;
    }

    try {
        await view.goTo(target);
    } catch {
        // Navigation failed, stay at current position.
    }
}

function buildPrioritySectionIndexes(
    highlights: LibraryAnnotation[],
    bookmarks: LibraryAnnotation[],
    highlightIndex: string | null,
    bookmarkIndex: string | null,
    loadedOverlaySections: Set<number>,
): number[] {
    const sectionIndexes = new Set<number>(loadedOverlaySections);
    const resolved = resolveTargetAnnotation(
        highlights,
        bookmarks,
        highlightIndex,
        bookmarkIndex,
    );

    if (resolved?.annotation.pos0) {
        const parsedPos = parseKoReaderPosition(resolved.annotation.pos0);
        if (parsedPos) {
            sectionIndexes.add(parsedPos.spineIndex);
        }
    }

    return Array.from(sectionIndexes);
}
