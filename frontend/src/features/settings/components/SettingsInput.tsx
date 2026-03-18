const inputClassName =
    'w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg px-3 py-2.5 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-dark-400 focus:outline-hidden focus:ring-2 focus:ring-primary-500/60 disabled:opacity-60 disabled:cursor-not-allowed';

type SettingsInputProps = Omit<
    React.InputHTMLAttributes<HTMLInputElement>,
    'className'
>;

export function SettingsInput(props: SettingsInputProps) {
    return <input {...props} className={inputClassName} />;
}
