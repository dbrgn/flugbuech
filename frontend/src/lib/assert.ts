// Licensing: This file is originally based on `src/common/utils/assert.ts` as part of Threema
// Desktop (https://github.com/threema-ch/threema-desktop/), which is released under the AGPLv3
// license.

/**
 * Assert a condition.
 *
 * @param condition The condition that must be `true` or otherwise this function will throw an
 *   error.
 * @param message Additional metadata that will be added to the error in case condition is `false`.
 * @throws {Error} If the condition is false.
 */
export function assert(condition: boolean, message?: string): asserts condition {
    if (!condition) {
        throw new Error(`Assertion failed, message: ${message}`);
    }
}

/**
 * Mark a section as asserted to be unreachable. This is not checked by the type system, so should
 * be avoided for matching.
 *
 * Use this in unreachable places without explicit matching, e.g. when chaining with the `??`
 * operator (`contacts.get('foo') ?? assert` in case the caller definitely knows that the contact
 * 'foo' exists).
 *
 * @throws {Error} Always.
 */
export function assertUnreachable(message: string): never {
    throw new Error(`Asserted unreachable code section: ${message}`);
}

/**
 * Unreachable code section. This variant is safe because it is checked by the type system.
 *
 * Use this in unreachable places, e.g. the default branch of a switch that should be exhaustive.
 * Will raise a compile error if considered reachable.
 *
 * @throws {Error} Always.
 */
export function unreachable(value: never, error?: Error): never {
    throw error ?? new Error('Unreachable code section!');
}

/**
 * Expect that a value exists. Return it if it exists and throw if it doesn't.
 */
export function unwrap<T>(value: T | null | undefined, message?: string): T {
    assert(value !== undefined && value !== null, message);
    return value;
}

/**
 * Ensure a caught error is an actual `Error` instance.
 */
export function ensureError(error: unknown): Error {
    if (error instanceof Error) {
        return error;
    }
    return new Error(`${error}`);
}
