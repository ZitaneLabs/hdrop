/** Unset crypto settings use the minimum; explicit values must be decimal integers. */
export function cryptoInteger(value: string | undefined, name: string, min: number, max: number): number {
    const raw = value ?? String(min)
    const number = Number(raw)
    if (!/^\d+$/.test(raw) || !Number.isSafeInteger(number) || number < min || number > max) {
        throw new RangeError(`${name} must be an integer between ${min} and ${max}`)
    }
    return number
}
