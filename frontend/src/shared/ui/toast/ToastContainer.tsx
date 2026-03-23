import { useToastStore } from './toast-store';
import { ToastItem } from './ToastItem';

export function ToastContainer() {
    const toasts = useToastStore();

    return (
        <div
            aria-live="polite"
            aria-relevant="additions removals"
            className="fixed z-[60] top-4 right-4 sm:top-auto sm:bottom-6 sm:right-6 flex flex-col sm:flex-col-reverse items-end gap-2 pointer-events-none"
        >
            {toasts.map((toast) => (
                <div key={toast.id} className="pointer-events-auto w-full sm:w-auto">
                    <ToastItem toast={toast} />
                </div>
            ))}
        </div>
    );
}
