import { PageContent } from '../features/layout/PageContent';
import { PageHeader } from '../features/layout/PageHeader';

type PlaceholderPageProps = {
    title: string;
};

export function PlaceholderPage({ title }: PlaceholderPageProps) {
    return (
        <>
            <PageHeader title={title} />
            <PageContent className="space-y-6 md:space-y-8">
                <section className="bg-white dark:bg-dark-850/50 rounded-lg p-6 border border-gray-200/30 dark:border-dark-700/70">
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                        {title}
                    </h2>
                    <p className="text-sm text-gray-500 dark:text-dark-300">
                        React route placeholder. This screen is queued for migration.
                    </p>
                </section>
            </PageContent>
        </>
    );
}
