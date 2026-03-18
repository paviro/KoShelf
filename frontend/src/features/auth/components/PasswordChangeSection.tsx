import { useCallback, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';

import { api, isApiHttpError } from '../../../shared/api';
import { translation } from '../../../shared/i18n';
import { SettingsField } from '../../settings/components/SettingsField';
import { SettingsInput } from '../../settings/components/SettingsInput';

function resolvePasswordChangeError(error: unknown): string {
    if (
        isApiHttpError(error) &&
        error.status === 400 &&
        error.code === 'invalid_credentials'
    ) {
        return translation.get('incorrect-password');
    }

    if (
        isApiHttpError(error) &&
        error.status === 400 &&
        error.code === 'invalid_query' &&
        error.apiMessage?.toLowerCase().includes('at least 8')
    ) {
        return translation.get('password-too-short');
    }

    if (isApiHttpError(error)) {
        return error.apiMessage ?? translation.get('error-state.title');
    }

    return translation.get('error-state.connection-title');
}

export function PasswordChangeSection() {
    const queryClient = useQueryClient();
    const [currentPassword, setCurrentPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [pending, setPending] = useState(false);
    const [feedback, setFeedback] = useState<{
        type: 'success' | 'error';
        message: string;
    } | null>(null);

    const handleSubmit = useCallback(
        async (event: React.FormEvent<HTMLFormElement>) => {
            event.preventDefault();

            if (pending) {
                return;
            }

            if (newPassword.length < 8) {
                setFeedback({
                    type: 'error',
                    message: translation.get('password-too-short'),
                });
                return;
            }

            if (newPassword !== confirmPassword) {
                setFeedback({
                    type: 'error',
                    message: translation.get('password-mismatch'),
                });
                return;
            }

            setPending(true);
            setFeedback(null);

            try {
                await api.changePassword(currentPassword, newPassword);
                setCurrentPassword('');
                setNewPassword('');
                setConfirmPassword('');
                setFeedback({
                    type: 'success',
                    message: translation.get('password-changed'),
                });
                void queryClient.invalidateQueries({
                    queryKey: ['auth', 'sessions'],
                });
            } catch (error) {
                setFeedback({
                    type: 'error',
                    message: resolvePasswordChangeError(error),
                });
            } finally {
                setPending(false);
            }
        },
        [confirmPassword, currentPassword, newPassword, pending, queryClient],
    );

    return (
        <form onSubmit={handleSubmit} className="space-y-4">
            <SettingsField
                label={translation.get('current-password')}
                htmlFor="settings-current-password"
                wide
            >
                <SettingsInput
                    id="settings-current-password"
                    type="password"
                    autoComplete="current-password"
                    placeholder={translation.get(
                        'current-password-placeholder',
                    )}
                    value={currentPassword}
                    onChange={(event) => {
                        setCurrentPassword(event.target.value);
                        setFeedback(null);
                    }}
                    disabled={pending}
                />
            </SettingsField>

            <SettingsField
                label={translation.get('new-password')}
                htmlFor="settings-new-password"
                hints={[translation.get('new-password-hint')]}
                wide
            >
                <SettingsInput
                    id="settings-new-password"
                    type="password"
                    autoComplete="new-password"
                    placeholder={translation.get('new-password-placeholder')}
                    value={newPassword}
                    onChange={(event) => {
                        setNewPassword(event.target.value);
                        setFeedback(null);
                    }}
                    disabled={pending}
                />
            </SettingsField>

            <SettingsField
                label={translation.get('confirm-password')}
                htmlFor="settings-confirm-password"
                wide
            >
                <SettingsInput
                    id="settings-confirm-password"
                    type="password"
                    autoComplete="new-password"
                    placeholder={translation.get(
                        'confirm-password-placeholder',
                    )}
                    value={confirmPassword}
                    onChange={(event) => {
                        setConfirmPassword(event.target.value);
                        setFeedback(null);
                    }}
                    disabled={pending}
                />
            </SettingsField>

            {feedback ? (
                <p
                    className={`text-sm px-3 py-2 rounded-lg border ${
                        feedback.type === 'success'
                            ? 'border-emerald-300/70 dark:border-emerald-500/40 bg-emerald-50/80 dark:bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                            : 'border-red-200/80 dark:border-red-500/40 bg-red-50/80 dark:bg-red-500/10 text-red-700 dark:text-red-300'
                    }`}
                >
                    {feedback.message}
                </p>
            ) : null}

            <SettingsField
                label={translation.get('change-password')}
                htmlFor="settings-change-password-submit"
            >
                <button
                    id="settings-change-password-submit"
                    type="submit"
                    className="w-full inline-flex items-center justify-center rounded-lg px-4 py-2.5 text-sm font-medium bg-primary-600 text-white hover:bg-primary-500 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
                    disabled={
                        pending ||
                        currentPassword.length === 0 ||
                        newPassword.length === 0 ||
                        confirmPassword.length === 0
                    }
                >
                    {translation.get('change-password')}
                </button>
            </SettingsField>
        </form>
    );
}
