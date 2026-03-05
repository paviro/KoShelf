import { LuInfo, LuScrollText } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';

type RecapEmptyStateProps = {
    hasYearContext: boolean;
};

export function RecapEmptyState({ hasYearContext }: RecapEmptyStateProps) {
    return (
        <section className="page-centered-state flex-col text-center">
            <div className="flex flex-col items-center justify-center">
                <div className="relative mb-8">
                    <div className="absolute inset-0 w-32 h-32 bg-gradient-to-br from-purple-500/20 to-pink-500/20 rounded-full blur-2xl"></div>
                    <div className="relative w-24 h-24 bg-gradient-to-br from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center shadow-2xl">
                        <LuScrollText
                            className="w-12 h-12 text-white"
                            aria-hidden
                        />
                    </div>
                </div>
                <h3 className="text-2xl md:text-3xl font-bold text-gray-900 dark:text-white mb-4">
                    {translation.get('recap-empty.nothing-here')}
                </h3>
                <p className="text-lg text-gray-600 dark:text-dark-300 max-w-2xl leading-relaxed">
                    {hasYearContext
                        ? translation.get('recap-empty.try-switching')
                        : translation.get('recap-empty.finish-reading')}
                </p>
            </div>

            <div className="max-w-md mx-auto mt-8 w-full">
                <details className="group bg-white dark:bg-dark-800/40 border border-gray-200/60 dark:border-dark-700/60 rounded-lg overflow-hidden transition-all duration-300 open:shadow-sm">
                    <summary className="flex items-center justify-between p-4 cursor-pointer select-none text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-dark-700/50 transition-colors list-none [&::-webkit-details-marker]:hidden">
                        <div className="flex items-center gap-3">
                            <LuInfo className="w-5 h-5 text-gray-400 dark:text-gray-500 flex-shrink-0" />
                            <span>
                                {translation.get('recap-empty.info-question')}
                            </span>
                        </div>
                        <svg
                            className="w-4 h-4 text-gray-400 transform transition-transform duration-200 group-open:rotate-180 ml-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth="2"
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </summary>
                    <div className="px-4 pb-4 pt-4 text-left text-sm text-gray-500 dark:text-gray-400 ml-8 leading-relaxed">
                        <p>{translation.get('recap-empty.info-answer')}</p>
                    </div>
                </details>
            </div>
        </section>
    );
}
