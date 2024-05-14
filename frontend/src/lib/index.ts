import type {RequestEvent} from '@sveltejs/kit';

export type SvelteKitFetch = RequestEvent['fetch'];

export function notUndefined<T>(value: T | undefined): value is T {
    return value !== undefined;
}
