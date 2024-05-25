/**
 * Returns all keys in the provided object.
 *
 * This is a typed version of {@link Object.keys}.
 */
export function keys<T>(object: {[key in keyof T]: unknown}): (keyof T)[] {
    return Object.keys(object) as (keyof T)[];
}
