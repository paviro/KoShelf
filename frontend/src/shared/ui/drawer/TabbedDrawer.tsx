import {
    useEffect,
    useId,
    useRef,
    useState,
    type MouseEvent,
    type ReactNode,
} from 'react';
import { LuX } from 'react-icons/lu';

import { translation } from '../../i18n';

const TRANSITION_DURATION_MS = 300;

export type TabbedDrawerTab<TTabId extends string> = {
    id: TTabId;
    label: string;
    content: ReactNode;
};

type TabbedDrawerProps<TTabId extends string> = {
    open: boolean;
    onClose: () => void;
    ariaLabel: string;
    tabs: readonly TabbedDrawerTab<TTabId>[];
    activeTab: TTabId;
    onTabChange: (tabId: TTabId) => void;
    tabListAriaLabel?: string;
    panelClassName?: string;
};

export function TabbedDrawer<TTabId extends string>({
    open,
    onClose,
    ariaLabel,
    tabs,
    activeTab,
    onTabChange,
    tabListAriaLabel,
    panelClassName,
}: TabbedDrawerProps<TTabId>) {
    const [isMounted, setIsMounted] = useState(false);
    const [isVisible, setIsVisible] = useState(false);
    const backdropRef = useRef<HTMLDivElement | null>(null);
    const idBase = useId();

    const [prevOpen, setPrevOpen] = useState(false);
    if (open !== prevOpen) {
        setPrevOpen(open);
        if (open) {
            setIsMounted(true);
        } else if (isMounted) {
            setIsVisible(false);
        }
    }

    useEffect(() => {
        if (!open && isMounted) {
            const timeoutId = window.setTimeout(() => {
                setIsMounted(false);
            }, TRANSITION_DURATION_MS);
            return () => window.clearTimeout(timeoutId);
        }
    }, [open, isMounted]);

    useEffect(() => {
        if (!open || !isMounted) {
            return;
        }

        if (backdropRef.current) {
            void backdropRef.current.offsetHeight;
        }

        const frameId = window.requestAnimationFrame(() => {
            setIsVisible(true);
        });

        return () => {
            window.cancelAnimationFrame(frameId);
        };
    }, [isMounted, open]);

    useEffect(() => {
        if (!isMounted) {
            return;
        }

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                onClose();
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [isMounted, onClose]);

    if (!isMounted) {
        return null;
    }

    const activeTabIndex = tabs.findIndex((tab) => tab.id === activeTab);
    const resolvedActiveTabIndex =
        activeTabIndex >= 0 ? activeTabIndex : tabs.length > 0 ? 0 : -1;
    const resolvedActiveTab =
        resolvedActiveTabIndex >= 0 ? tabs[resolvedActiveTabIndex] : null;

    const handleBackdropMouseDown = (event: MouseEvent<HTMLDivElement>) => {
        if (event.target === event.currentTarget) {
            onClose();
        }
    };

    return (
        <div
            ref={backdropRef}
            className={`fixed inset-0 z-50 p-2 bg-gray-950/45 dark:bg-black/55 backdrop-blur-[2px] transition-opacity duration-300 ${
                isVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'
            }`}
            onMouseDown={handleBackdropMouseDown}
        >
            <div
                className={`ml-auto h-full w-[32rem] max-w-full bg-white/95 dark:bg-dark-900/88 backdrop-blur-xl border border-gray-200/70 dark:border-dark-600/50 rounded-2xl overflow-hidden shadow-2xl transform transition-transform duration-300 ${
                    panelClassName ?? ''
                } ${isVisible ? 'translate-x-0' : 'translate-x-full'}`}
                onMouseDown={(event) => event.stopPropagation()}
            >
                <div
                    className="flex flex-col h-full min-h-0"
                    role="dialog"
                    aria-modal="true"
                    aria-label={ariaLabel}
                >
                    <div className="shrink-0 px-4 pt-4 pb-3 border-b border-gray-200/70 dark:border-dark-700/50 bg-white/70 dark:bg-dark-900/35 backdrop-blur-xs">
                        <div className="flex items-center justify-between gap-3">
                            <div
                                role="tablist"
                                aria-label={tabListAriaLabel ?? ariaLabel}
                                className="inline-flex items-center gap-1 p-1 rounded-xl bg-gray-100/70 dark:bg-dark-800/60 border border-gray-200/70 dark:border-dark-700/60 shadow-xs"
                            >
                                {tabs.map((tab, index) => {
                                    const isActive =
                                        index === resolvedActiveTabIndex;
                                    const tabId = `${idBase}-tab-${index}`;
                                    const panelId = `${idBase}-panel-${index}`;

                                    return (
                                        <TabButton
                                            key={tab.id}
                                            id={tabId}
                                            panelId={panelId}
                                            active={isActive}
                                            onClick={() => onTabChange(tab.id)}
                                            label={tab.label}
                                        />
                                    );
                                })}
                            </div>
                            <button
                                type="button"
                                className="w-10 h-10 flex items-center justify-center rounded-lg bg-gray-100/50 dark:bg-dark-800/40 border border-gray-300/60 dark:border-dark-700/60 text-gray-500 dark:text-dark-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-200/60 dark:hover:bg-dark-700/60 transition-colors duration-200 backdrop-blur-xs focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50"
                                aria-label={translation.get('close.aria-label')}
                                onClick={onClose}
                            >
                                <LuX className="w-5 h-5" aria-hidden="true" />
                            </button>
                        </div>
                    </div>

                    <div className="flex-1 min-h-0 px-4 py-3">
                        {resolvedActiveTab && (
                            <div
                                id={`${idBase}-panel-${resolvedActiveTabIndex}`}
                                role="tabpanel"
                                aria-labelledby={`${idBase}-tab-${resolvedActiveTabIndex}`}
                                className="h-full overflow-y-auto pr-1"
                                data-tabbed-drawer-scroll-container
                            >
                                {resolvedActiveTab.content}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

type TabButtonProps = {
    id: string;
    panelId: string;
    active: boolean;
    onClick: () => void;
    label: string;
};

function TabButton({ id, panelId, active, onClick, label }: TabButtonProps) {
    return (
        <button
            id={id}
            type="button"
            role="tab"
            aria-selected={active}
            aria-controls={panelId}
            tabIndex={active ? 0 : -1}
            onClick={onClick}
            className={`cursor-pointer px-3.5 py-1.5 text-sm font-medium rounded-lg border transition-colors duration-150 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary-500/50 ${
                active
                    ? 'text-gray-900 dark:text-white bg-white dark:bg-dark-700/70 border-gray-200/80 dark:border-dark-600/70 shadow-xs'
                    : 'text-gray-500 dark:text-dark-300 bg-transparent border-transparent hover:text-gray-700 dark:hover:text-dark-100 hover:bg-white/70 dark:hover:bg-dark-700/45'
            }`}
        >
            {label}
        </button>
    );
}
