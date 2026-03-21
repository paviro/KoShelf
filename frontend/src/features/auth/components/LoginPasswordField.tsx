import { LuLock } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';

type LoginPasswordFieldProps = {
    password: string;
    disabled: boolean;
    onPasswordChange: (value: string) => void;
};

export function LoginPasswordField({
    password,
    disabled,
    onPasswordChange,
}: LoginPasswordFieldProps) {
    return (
        <div className="space-y-2">
            <label
                htmlFor="login-password"
                className="block text-sm font-medium text-gray-900 dark:text-white"
            >
                {translation.get('login.password')}
            </label>
            <div className="relative">
                <LuLock
                    className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400 dark:text-dark-400"
                    aria-hidden="true"
                />
                <input
                    id="login-password"
                    type="password"
                    autoComplete="current-password"
                    value={password}
                    onChange={(event) => onPasswordChange(event.target.value)}
                    className="w-full bg-gray-50 dark:bg-dark-800/70 border border-gray-300/70 dark:border-dark-700 rounded-lg pl-10 pr-3 py-2.5 text-gray-900 dark:text-white focus:outline-hidden focus:ring-2 focus:ring-primary-500/60 disabled:opacity-60 disabled:cursor-not-allowed"
                    disabled={disabled}
                    required
                />
            </div>
        </div>
    );
}
