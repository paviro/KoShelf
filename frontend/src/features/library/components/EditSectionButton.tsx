import { LuPencil } from 'react-icons/lu';

import { translation } from '../../../shared/i18n';
import { Button } from '../../../shared/ui/button/Button';

type EditSectionButtonProps = {
    editing: boolean;
    onToggle: () => void;
};

export function EditSectionButton({
    editing,
    onToggle,
}: EditSectionButtonProps) {
    return (
        <Button
            variant="neutral"
            size="icon"
            icon={LuPencil}
            label={translation.get('edit.aria-label')}
            onClick={onToggle}
            active={editing}
        />
    );
}
