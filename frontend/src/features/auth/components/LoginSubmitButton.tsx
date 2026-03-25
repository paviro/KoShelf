import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';

type LoginSubmitButtonProps = {
    disabled: boolean;
};

export function LoginSubmitButton({ disabled }: LoginSubmitButtonProps) {
    return (
        <Button
            variant="gradient"
            size="sm"
            fullWidth
            type="submit"
            disabled={disabled}
        >
            {translation.get('login.submit')}
        </Button>
    );
}
