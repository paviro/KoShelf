type AnnotationReaderTarget = 'highlight' | 'bookmark';

export function annotationReaderHref(
    readerBaseHref: string | null | undefined,
    target: AnnotationReaderTarget,
    index: number,
): string | undefined {
    if (!readerBaseHref) {
        return undefined;
    }

    const searchParams = new URLSearchParams({
        [target]: String(index),
    });
    return `${readerBaseHref}?${searchParams.toString()}`;
}
