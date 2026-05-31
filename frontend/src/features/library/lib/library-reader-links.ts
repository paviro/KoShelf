type AnnotationReaderTarget = 'highlight' | 'bookmark';

export function annotationReaderHref(
    readerBaseHref: string | null | undefined,
    target: AnnotationReaderTarget,
    annotationId: string,
): string | undefined {
    if (!readerBaseHref) {
        return undefined;
    }

    const searchParams = new URLSearchParams({
        [target]: annotationId,
    });
    return `${readerBaseHref}?${searchParams.toString()}`;
}
