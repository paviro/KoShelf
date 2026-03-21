import { translation } from '../../../shared/i18n';

type LoginSubmitButtonProps = {
    disabled: boolean;
};

export function LoginSubmitButton({ disabled }: LoginSubmitButtonProps) {
    return (
        <button
            type="submit"
            className="w-full inline-flex items-center justify-center rounded-lg px-4 py-2.5 text-sm font-semibold text-white bg-linear-to-r from-primary-600 to-primary-500 hover:from-primary-500 hover:to-primary-400 disabled:opacity-60 disabled:cursor-not-allowed transition-all shadow-lg shadow-primary-500/20"
            disabled={disabled}
        >
            {translation.get('login.submit')}
        </button>
    );
}
