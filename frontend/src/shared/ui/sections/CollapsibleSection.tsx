import type { ReactNode } from 'react';
import { LuChevronDown } from 'react-icons/lu';

import { translation } from '../../i18n';
import { Button } from '../button/Button';

type ToggleButtonProps = {
    visible: boolean;
    onClick: () => void;
};

function ToggleButton({ visible, onClick }: ToggleButtonProps) {
    return (
        <Button
            variant="neutral"
            size="icon"
            className="sm:w-auto sm:px-3 sm:space-x-2"
            data-section-toggle-button
            onClick={onClick}
        >
            <span className="hidden sm:inline text-sm font-medium text-gray-600 dark:text-dark-300">
                {visible
                    ? translation.get('toggle.hide')
                    : translation.get('toggle.show')}
            </span>
            <LuChevronDown
                className="w-4 h-4 text-gray-600 dark:text-gray-300 transform transition-transform duration-200"
                style={{
                    transform: visible ? 'rotate(0deg)' : 'rotate(-90deg)',
                }}
                aria-hidden="true"
            />
        </Button>
    );
}

type CollapsibleSectionProps = {
    sectionKey: string;
    accentClass: string;
    title: string;
    titleBadge?: ReactNode;
    defaultVisible?: boolean;
    visible: boolean;
    onToggle: () => void;
    controls?: ReactNode;
    controlsClassName?: string;
    contentClassName?: string;
    children: ReactNode;
};

export function CollapsibleSection({
    sectionKey,
    accentClass,
    title,
    titleBadge,
    defaultVisible = true,
    visible,
    onToggle,
    controls,
    controlsClassName = 'space-x-3',
    contentClassName,
    children,
}: CollapsibleSectionProps) {
    const containerClassName = [contentClassName, visible ? '' : 'hidden']
        .filter(Boolean)
        .join(' ');

    return (
        <section
            data-name={sectionKey}
            data-default-visible={defaultVisible ? 'true' : 'false'}
        >
            <div className="flex items-center justify-between mb-4 md:mb-6 pb-4 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className="flex items-center space-x-3">
                    <div
                        className={`w-2 h-6 md:h-8 rounded-full ${accentClass}`}
                    ></div>
                    <h2 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                        {title}
                    </h2>
                    {titleBadge}
                </div>
                <div className={`flex items-center ${controlsClassName}`}>
                    {controls}
                    <ToggleButton visible={visible} onClick={onToggle} />
                </div>
            </div>

            <div id={`${sectionKey}Container`} className={containerClassName}>
                {children}
            </div>
        </section>
    );
}
