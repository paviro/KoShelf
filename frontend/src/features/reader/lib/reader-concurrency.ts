export async function runWithConcurrency<T>(
    items: T[],
    maxConcurrent: number,
    worker: (item: T) => Promise<void>,
) {
    if (items.length === 0) {
        return;
    }

    const limit = Math.max(1, maxConcurrent);
    let nextIndex = 0;

    const runners = Array.from(
        { length: Math.min(limit, items.length) },
        async () => {
            while (nextIndex < items.length) {
                const currentIndex = nextIndex;
                nextIndex += 1;
                await worker(items[currentIndex]);
            }
        },
    );

    await Promise.all(runners);
}
