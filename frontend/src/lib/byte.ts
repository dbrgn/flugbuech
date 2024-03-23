// prettier-ignore
const HEX_LOOKUP_TABLE = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
] as const;

/**
 * Convert an Uint8Array to a 0-padded lowercase hex string.
 *
 * @param array Array to convert
 * @returns String as hex
 */
export function bytesToHex(array: Uint8Array): string {
    return array.reduce(
        (parts, value) => parts + HEX_LOOKUP_TABLE[value >>> 4] + HEX_LOOKUP_TABLE[value & 0x0f],
        '',
    );
}
