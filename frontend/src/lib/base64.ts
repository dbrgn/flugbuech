// Licensing: This file is originally based on `src/common/utils/assert.ts` as part of Threema
// Desktop (https://github.com/threema-ch/threema-desktop/), which is released under the AGPLv3
// license.

export interface Base64Options {
    /**
     * If set to true, then the Base64 encoding will be URL safe according to RFC 4648 (with `+` and
     * `/` replaced by `-` and `_`).
     */
    readonly urlSafe?: boolean;
    /**
     * If set to true, then padding (`=`) will be stripped.
     */
    readonly noPad?: boolean;
}

/**
 * Encode a Uint8Array to a base 64 string.
 */
export function u8aToBase64(array: Uint8Array, options?: Base64Options): string {
    let base64 = btoa(Array.from(array, (byte) => String.fromCharCode(byte)).join(''));
    if (options?.urlSafe === true) {
        base64 = base64.replaceAll('+', '-').replaceAll('/', '_');
    }
    if (options?.noPad === true) {
        base64 = base64.replace(/=+$/, '');
    }
    return base64;
}
