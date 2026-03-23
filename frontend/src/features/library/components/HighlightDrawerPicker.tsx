import {
    LuCheck,
    LuHighlighter,
    LuStrikethrough,
    LuType,
    LuUnderline,
} from 'react-icons/lu';
import type { IconType } from 'react-icons';

import { translation } from '../../../shared/i18n';
import { OverlayPicker } from './OverlayPicker';

const DRAWERS: ReadonlyArray<{
    name: string;
    labelKey: string;
    icon: IconType;
    sampleClass: string;
}> = [
    {
        name: 'lighten',
        labelKey: 'highlight-drawer.lighten',
        icon: LuHighlighter,
        sampleClass: 'bg-amber-300/60 dark:bg-amber-400/40',
    },
    {
        name: 'underscore',
        labelKey: 'highlight-drawer.underscore',
        icon: LuUnderline,
        sampleClass: 'underline underline-offset-2 decoration-2 decoration-current',
    },
    {
        name: 'strikeout',
        labelKey: 'highlight-drawer.strikeout',
        icon: LuStrikethrough,
        sampleClass: 'line-through decoration-2 decoration-current',
    },
    {
        name: 'invert',
        labelKey: 'highlight-drawer.invert',
        icon: LuType,
        sampleClass: 'bg-gray-800 text-white dark:bg-white dark:text-gray-900 px-1 rounded-sm',
    },
];

export const DRAWER_ICONS: Record<string, IconType> = {
    lighten: LuHighlighter,
    underscore: LuUnderline,
    strikeout: LuStrikethrough,
    invert: LuType,
};

type HighlightDrawerPickerProps = {
    anchorRef: React.RefObject<HTMLElement | null>;
    currentDrawer: string;
    onSelect: (drawer: string) => void;
    onClose: () => void;
};

export function HighlightDrawerPicker({
    anchorRef,
    currentDrawer,
    onSelect,
    onClose,
}: HighlightDrawerPickerProps) {
    return (
        <OverlayPicker anchorRef={anchorRef} onClose={onClose}>
            <div className="flex flex-col gap-0.5">
                {DRAWERS.map((drawer) => {
                    const isSelected = drawer.name === currentDrawer;
                    const Icon = drawer.icon;

                    return (
                        <button
                            key={drawer.name}
                            type="button"
                            onClick={() => onSelect(drawer.name)}
                            className={`flex items-center gap-2.5 px-3 py-1.5 rounded-lg text-sm transition-colors ${
                                isSelected
                                    ? 'bg-primary-500/15 text-primary-700 dark:text-primary-300'
                                    : 'text-gray-700 dark:text-dark-200 hover:bg-gray-100 dark:hover:bg-dark-700/60'
                            }`}
                        >
                            <Icon className="w-4 h-4 shrink-0" aria-hidden="true" />
                            <span className={drawer.sampleClass}>
                                {translation.get(drawer.labelKey)}
                            </span>
                            {isSelected && (
                                <LuCheck className="w-3.5 h-3.5 ml-auto text-primary-600 dark:text-primary-400" />
                            )}
                        </button>
                    );
                })}
            </div>
        </OverlayPicker>
    );
}
