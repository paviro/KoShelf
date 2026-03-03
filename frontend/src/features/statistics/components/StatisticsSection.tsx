import type { ReactNode } from 'react';

import { translation } from '../../../shared/i18n';
import type { SectionName } from '../model/statistics-model';

type ToggleButtonProps = {
    visible: boolean;
    onClick: () => void;
};

function ToggleButton({ visible, onClick }: ToggleButtonProps) {
    return (
        <button
            data-section-toggle-button
            onClick={onClick}
            className="flex items-center justify-center sm:space-x-2 w-10 sm:w-auto sm:px-3 h-10 bg-gray-100/50 dark:bg-dark-800/50 border border-gray-300/50 dark:border-dark-700/50 rounded-lg hover:bg-gray-200/50 dark:hover:bg-dark-700/50 transition-colors backdrop-blur-sm"
        >
            <span className="hidden sm:inline text-sm text-gray-600 dark:text-dark-300">
                {visible ? translation.get('toggle.hide') : translation.get('toggle.show')}
            </span>
            <svg
                className="w-4 h-4 text-gray-600 dark:text-gray-300 transform transition-transform duration-200"
                style={{ transform: visible ? 'rotate(0deg)' : 'rotate(-90deg)' }}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth="2"
                    d="M19 9l-7 7-7-7"
                ></path>
            </svg>
        </button>
    );
}

type StatisticsSectionProps = {
    sectionName: SectionName;
    accentClass: string;
    title: string;
    visible: boolean;
    onToggle: () => void;
    controls?: ReactNode;
    children: ReactNode;
};

export function StatisticsSection({
    sectionName,
    accentClass,
    title,
    visible,
    onToggle,
    controls,
    children,
}: StatisticsSectionProps) {
    return (
        <section data-name={sectionName} data-default-visible="true">
            <div className="flex items-center justify-between mb-4 md:mb-6 pb-4 border-b border-gray-200/50 dark:border-dark-700/50">
                <div className="flex items-center space-x-3">
                    <div className={`w-2 h-6 md:h-8 rounded-full ${accentClass}`}></div>
                    <h2 className="text-xl md:text-2xl font-bold text-gray-900 dark:text-white">
                        {title}
                    </h2>
                </div>
                <div className="flex items-center space-x-3">
                    {controls}
                    <ToggleButton visible={visible} onClick={onToggle} />
                </div>
            </div>

            <div id={`${sectionName}Container`} className={visible ? '' : 'hidden'}>
                {children}
            </div>
        </section>
    );
}
