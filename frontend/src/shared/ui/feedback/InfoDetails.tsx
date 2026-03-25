import { LuInfo } from 'react-icons/lu';

type InfoDetailsProps = {
    question: string;
    answer: string;
};

export function InfoDetails({ question, answer }: InfoDetailsProps) {
    return (
        <div className="max-w-md mx-auto mt-8 w-full">
            <details className="group bg-white dark:bg-dark-800/40 border border-gray-200/60 dark:border-dark-700/60 rounded-lg overflow-hidden transition-all duration-300 open:shadow-xs">
                <summary className="flex items-center justify-between p-4 cursor-pointer select-none text-sm font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-dark-700/50 transition-colors list-none [&::-webkit-details-marker]:hidden">
                    <div className="flex items-center gap-3">
                        <LuInfo
                            className="w-5 h-5 text-gray-400 dark:text-gray-500 shrink-0"
                            aria-hidden
                        />
                        <span>{question}</span>
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
                    <p>{answer}</p>
                </div>
            </details>
        </div>
    );
}
