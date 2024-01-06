import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';
import {error} from '@sveltejs/kit';
import {extractResponseError} from '$lib/api';
import {AuthenticationError} from '$lib/errors';

const SCHEMA_API_PROFILE = z.object({
    username: z.string(),
    email: z.string(),
    signedUp: z
        .string()
        .transform((datestring) => new Date(datestring))
        .optional(),
});

export type Profile = z.infer<typeof SCHEMA_API_PROFILE>;

/**
 * Load profile from API.
 */
export async function loadApiProfile(fetch: SvelteKitFetch): Promise<Profile> {
    const res = await fetch('/api/v1/profile');
    switch (res.status) {
        case 200:
            return SCHEMA_API_PROFILE.parse(await res.json());
        case 401:
            throw AuthenticationError.redirectToLogin(`/profile`);
        case 403:
            return error(403, `This is not your location, viewing not allowed`);
        default:
            // TODO: Better error handling
            throw new Error(`Could not fetch profile from API: ${extractResponseError(res)}`);
    }
}
