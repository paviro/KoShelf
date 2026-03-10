export type UnitValuePart = {
    amount: string;
    unit?: string;
};

export function joinUnitValueParts(parts: UnitValuePart[]): string {
    return parts
        .filter((part) => part.amount || part.unit)
        .map((part) => `${part.amount}${part.unit ?? ''}`)
        .join(' ');
}
