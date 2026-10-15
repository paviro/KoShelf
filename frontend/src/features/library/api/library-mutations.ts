import { useMutation, useQueryClient } from '@tanstack/react-query';

import { api } from '../../../shared/api';
import type {
    MetadataPatch,
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
    applyCustomMetadataUpdate,
    applyItemUpdate,
    patchDetailCache,
} from '../lib/optimistic-cache';

type RollbackContext = { previous?: LibraryDetailData };

export function useUpdateItem(itemId: string, collection: LibraryCollection) {
    const queryClient = useQueryClient();
    const detailKey = libraryDetailQueryKey(collection, itemId);

    return useMutation<
        void,
        Error,
        { payload: UpdateItemPayload; revision: string },
        RollbackContext
    >({
        mutationFn: ({ payload, revision }) =>
            api.updateItem(itemId, payload, revision),

        onMutate: async ({ payload }) => {
            await queryClient.cancelQueries({ queryKey: detailKey });
            const previous = patchDetailCache(queryClient, detailKey, (data) =>
                applyItemUpdate(data, payload),
            );
            return { previous };
        },

        onError: (error, _payload, context) => {
            console.error('[useUpdateItem] Failed to update item:', error);
            addToast('error', translation.get('toast-update-item-error'), {
                subtitle: translation.get('toast-update-item-error.subtitle'),
            });
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
    revision: string;
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
            api.updateAnnotation(
                itemId,
                vars.annotationId,
                vars.payload,
                vars.revision,
            ),

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
            addToast(
                'error',
                translation.get('toast-update-annotation-error'),
                {
                    subtitle: translation.get(
                        'toast-update-annotation-error.subtitle',
                    ),
                },
            );
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

    return useMutation<
        void,
        Error,
        { annotationId: string; revision: string },
        RollbackContext
    >({
        mutationFn: ({ annotationId, revision }) =>
            api.deleteAnnotation(itemId, annotationId, revision),

        onMutate: async ({ annotationId }) => {
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
            addToast(
                'error',
                translation.get('toast-delete-annotation-error'),
                {
                    subtitle: translation.get(
                        'toast-delete-annotation-error.subtitle',
                    ),
                },
            );
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

type CustomEditVars = {
    metadata?: { patch: MetadataPatch; revision: string };
    /** `null` restores the original cover. */
    cover?: { file: File | null; revision: string };
};

export function useCustomEdit(itemId: string, collection: LibraryCollection) {
    const queryClient = useQueryClient();
    const detailKey = libraryDetailQueryKey(collection, itemId);

    return useMutation<void, Error, CustomEditVars, RollbackContext>({
        // The cover goes first: an invalid image is rejected before anything
        // is written, so it never leaves the metadata half-saved. Each file
        // has its own revision, so saving one doesn't invalidate the other.
        mutationFn: async ({ metadata, cover }) => {
            if (cover) {
                await (cover.file
                    ? api.uploadCover(itemId, cover.file, cover.revision)
                    : api.resetCover(itemId, cover.revision));
            }
            if (metadata) {
                await api.updateMetadata(
                    itemId,
                    metadata.patch,
                    metadata.revision,
                );
            }
        },

        onMutate: async ({ metadata }) => {
            await queryClient.cancelQueries({ queryKey: detailKey });
            const previous = metadata
                ? patchDetailCache(queryClient, detailKey, (data) =>
                      applyCustomMetadataUpdate(data, metadata.patch),
                  )
                : queryClient.getQueryData<LibraryDetailData>(detailKey);
            return { previous };
        },

        onError: (error, _vars, context) => {
            console.error('[useCustomEdit] Failed to update item:', error);
            addToast('error', translation.get('toast-update-item-error'), {
                subtitle: translation.get('toast-update-item-error.subtitle'),
            });
            if (context?.previous) {
                queryClient.setQueryData(detailKey, context.previous);
            }
        },

        onSettled: () => {
            // One source may affect multiple formats and collections.
            void queryClient.invalidateQueries({
                queryKey: ['library-detail'],
            });
            void queryClient.invalidateQueries({ queryKey: ['library-list'] });
        },
    });
}
