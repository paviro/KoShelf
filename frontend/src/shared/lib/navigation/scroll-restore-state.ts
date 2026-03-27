let activeRestoreCount = 0;

export function beginScrollRestore(): void {
    activeRestoreCount += 1;
}

export function endScrollRestore(): void {
    if (activeRestoreCount > 0) {
        activeRestoreCount -= 1;
    }
}

export function isScrollRestoringNow(): boolean {
    return activeRestoreCount > 0;
}
