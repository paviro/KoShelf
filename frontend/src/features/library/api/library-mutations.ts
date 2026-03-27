import { useMutation, useQueryClient } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type {
    UpdateAnnotationPayload,
    UpdateItemPayload,
} from '../../../shared/api-client';
import { translation } from '../../../shared/i18n';
import { addToast } from '../../../shared/ui/toast';
import type { LibraryDetailData } from '../api/library-data';
import type { LibraryCollection } from '../model/library-model';
import {
    libraryDetailQueryKey,
    libraryListQueryKey,
} from '../hooks/useLibraryQueries';
import {
    applyAnnotationDeletion,
    applyAnnotationUpdate,
    applyItemUpdate,
    patchDetailCache,
} from '../lib/optimistic-cache';

type RollbackContext = { previous?: LibraryDetailData };

export function useUpdateItem(itemId: string, collection: LibraryCollection) {
    const queryClient = useQueryClient();
    const detailKey = libraryDetailQueryKey(collection, itemId);

    return useMutation<void, Error, UpdateItemPayload, RollbackContext>({
        mutationFn: (payload) => api.updateItem(itemId, payload),

        onMutate: async (payload) => {
            await queryClient.cancelQueries({ queryKey: detailKey });
            const previous = patchDetailCache(queryClient, detailKey, (data) =>
                applyItemUpdate(data, payload),
            );
            return { previous };
        },

        onError: (error, _payload, context) => {
            console.error('[useUpdateItem] Failed to update item:', error);
            addToast('error', translation.get('toast-update-item-error'));
            if (context?.previous) {
                queryClient.setQueryData(detailKey, context.previous);
            }
        },

        onSettled: () => {
            void queryClient.invalidateQueries({ queryKey: detailKey });
            void queryClient.invalidateQueries({
                queryKey: libraryListQueryKey(collection),
            });
        },
    });
}

type AnnotationVars = {
    annotationId: string;
    payload: UpdateAnnotationPayload;
};

export function useUpdateAnnotation(
    itemId: string,
    collection: LibraryCollection,
) {
    const queryClient = useQueryClient();
    const detailKey = libraryDetailQueryKey(collection, itemId);

    return useMutation<void, Error, AnnotationVars, RollbackContext>({
        mutationFn: (vars) =>
            api.updateAnnotation(itemId, vars.annotationId, vars.payload),

        onMutate: async (vars) => {
            await queryClient.cancelQueries({ queryKey: detailKey });
            const previous = patchDetailCache(queryClient, detailKey, (data) =>
                applyAnnotationUpdate(data, vars.annotationId, vars.payload),
            );
            return { previous };
        },

        onError: (error, _vars, context) => {
            console.error(
                '[useUpdateAnnotation] Failed to update annotation:',
                error,
            );
            addToast('error', translation.get('toast-update-annotation-error'));
            if (context?.previous) {
                queryClient.setQueryData(detailKey, context.previous);
            }
        },

        onSettled: () => {
            void queryClient.invalidateQueries({ queryKey: detailKey });
        },
    });
}

export function useDeleteAnnotation(
    itemId: string,
    collection: LibraryCollection,
) {
    const queryClient = useQueryClient();
    const detailKey = libraryDetailQueryKey(collection, itemId);

    return useMutation<void, Error, string, RollbackContext>({
        mutationFn: (annotationId) =>
            api.deleteAnnotation(itemId, annotationId),

        onMutate: async (annotationId) => {
            await queryClient.cancelQueries({ queryKey: detailKey });
            const previous = patchDetailCache(queryClient, detailKey, (data) =>
                applyAnnotationDeletion(data, annotationId),
            );
            return { previous };
        },

        onError: (error, _annotationId, context) => {
            console.error(
                '[useDeleteAnnotation] Failed to delete annotation:',
                error,
            );
            addToast('error', translation.get('toast-delete-annotation-error'));
            if (context?.previous) {
                queryClient.setQueryData(detailKey, context.previous);
            }
        },

        onSettled: () => {
            void queryClient.invalidateQueries({ queryKey: detailKey });
            void queryClient.invalidateQueries({
                queryKey: libraryListQueryKey(collection),
            });
            void queryClient.invalidateQueries({
                queryKey: ['page-activity', itemId],
            });
        },
    });
}
