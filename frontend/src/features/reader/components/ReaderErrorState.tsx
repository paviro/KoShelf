import { Link } from 'react-router';

import { translation } from '../../../shared/i18n';

type ReaderErrorStateProps = {
    error: string;
    backHref: string;
    onBackClick: (event: React.MouseEvent<HTMLAnchorElement>) => void;
};

export function ReaderErrorState({
    error,
    backHref,
    onBackClick,
}: ReaderErrorStateProps) {
    return (
        <div className="absolute inset-0 flex items-center justify-center p-8">
            <div className="text-center">
                <p className="text-lg text-red-500 dark:text-red-400 mb-4">
                    {translation.get('reader-error')}
                </p>
                <p className="text-sm text-gray-500 dark:text-dark-400 mb-6">
                    {error}
                </p>
                <Link
                    to={backHref}
                    onClick={onBackClick}
                    className="inline-flex items-center px-4 py-2 rounded-lg text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors"
                >
                    {translation.get('go-back.aria-label')}
                </Link>
            </div>
        </div>
    );
}
