type SettingsFieldProps = {
    label: string;
    htmlFor: string;
    hints?: string[];
    wide?: boolean;
    children: React.ReactNode;
};

export function SettingsField({
    label,
    htmlFor,
    hints,
    wide,
    children,
}: SettingsFieldProps) {
    return (
        <div className="bg-white dark:bg-dark-850/50 border border-gray-200/70 dark:border-dark-700/70 rounded-lg p-4">
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                <div className="space-y-0.5">
                    <label
                        htmlFor={htmlFor}
                        className="block text-base font-semibold text-gray-900 dark:text-white"
                    >
                        {label}
                    </label>
                    {hints?.map((line, i) => (
                        <p
                            key={i}
                            className="text-sm font-medium text-gray-500 dark:text-dark-400"
                        >
                            {line}
                        </p>
                    ))}
                </div>
                <div className={`${wide ? 'sm:w-72' : 'sm:w-56'} shrink-0`}>
                    {children}
                </div>
            </div>
        </div>
    );
}
