// Licensing: This file is originally based on `src/common/utils/object.ts` as part of Threema
// Desktop (https://github.com/threema-ch/threema-desktop/), which is released under the AGPLv3
// license.

/**
 * Returns all keys in the provided object.
 *
 * This is a typed version of {@link Object.keys}.
 */
export function keys<T>(object: {[key in keyof T]: unknown}): (keyof T)[] {
    return Object.keys(object) as (keyof T)[];
}
