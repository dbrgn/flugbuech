/**
 * Wraps a function with additional dependencies, so that Svelte will re-evaluate it if any of the
 * given dependencies change.
 *
 * Note: Calls to this function must always be in a reactive block (`$: reactive(...)`).)
 */
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function reactive<TReturn>(fn: () => TReturn, dependencies: unknown[]): TReturn {
    return fn();
}
