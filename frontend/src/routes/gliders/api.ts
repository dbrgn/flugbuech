import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';
import {error} from '@sveltejs/kit';
import {AuthenticationError, ensureClientOrServerErrorCode} from '$lib/errors';
import {extractResponseError} from '$lib/api';

const SCHEMA_API_GLIDER = z.object({
    id: z.number(),
    manufacturer: z.string(),
    model: z.string(),
    since: z.string().optional(),
    until: z.string().optional(),
    source: z.string().optional(),
    cost: z.number().optional(),
    comment: z.string().optional(),
    stats: z.object({
        flights: z.number(),
        seconds: z.number(),
        secondsComplete: z.boolean(),
    }),
});

export type Glider = z.infer<typeof SCHEMA_API_GLIDER>;

const SCHEMA_API_GLIDERS_RESPONSE = z.object({
    gliders: z.array(SCHEMA_API_GLIDER),
});

/**
 * Load gliders from API.
 */
export async function loadApiGliders(fetch: SvelteKitFetch): Promise<Glider[]> {
    const res = await fetch('/api/v1/gliders');
    switch (res.status) {
        case 200: {
            const response = SCHEMA_API_GLIDERS_RESPONSE.parse(await res.json());
            return response.gliders;
        }
        case 401:
            throw AuthenticationError.redirectToLogin(`/gliders/`);
        default: {
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not fetch gliders from API: ${await extractResponseError(res)}`,
            );
        }
    }
}
