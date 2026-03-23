import { translation } from '../../../shared/i18n';

export function StatisticsEmptyState() {
    return (
        <div
            id="dynamicEmptyState"
            className="page-centered-state flex-col text-center"
        >
            <div className="flex flex-col items-center justify-center">
                <div className="relative mb-8">
                    <div className="absolute inset-0 w-32 h-32 bg-linear-to-br from-green-500/20 to-emerald-500/20 rounded-full blur-2xl"></div>
                    <div className="relative w-24 h-24 bg-linear-to-br from-green-500 to-emerald-500 rounded-2xl flex items-center justify-center shadow-2xl">
                        <svg
                            className="w-12 h-12 text-white"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            strokeWidth="1.5"
                            aria-hidden
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z"
                            />
                        </svg>
                    </div>
                </div>
                <h3 className="text-2xl md:text-3xl font-bold text-gray-900 dark:text-white mb-4">
                    {translation.get('stats-empty.nothing-here')}
                </h3>
                <p className="text-lg text-gray-600 dark:text-dark-300 max-w-2xl leading-relaxed whitespace-pre-line">
                    {translation.get('stats-empty.start-reading')}
                </p>
            </div>

            <div className="max-w-md mx-auto mt-8 w-full">
                <details className="group bg-white dark:bg-dark-800/40 border border-gray-200/60 dark:border-dark-700/60 rounded-lg overflow-hidden transition-all duration-300 open:shadow-xs">
                    <summary className="flex items-center justify-between p-4 cursor-pointer select-none text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-dark-700/50 transition-colors list-none [&::-webkit-details-marker]:hidden">
                        <div className="flex items-center gap-3">
                            <svg
                                className="w-5 h-5 text-gray-400 dark:text-gray-500 shrink-0"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                                aria-hidden
                            >
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth="2"
                                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                ></path>
                            </svg>
                            <span>
                                {translation.get('stats-empty.info-question')}
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
                    <div className="px-4 pb-4 pt-4 text-left text-sm font-medium text-gray-500 dark:text-gray-400 ml-8 leading-relaxed">
                        <p>{translation.get('stats-empty.info-answer')}</p>
                    </div>
                </details>
            </div>
        </div>
    );
}
