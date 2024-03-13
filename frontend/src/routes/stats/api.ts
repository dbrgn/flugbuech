import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';
import {error} from '@sveltejs/kit';
import {extractResponseError} from '$lib/api';
import {AuthenticationError, ensureClientOrServerErrorCode} from '$lib/errors';

import {SCHEMA_API_LOCATION} from '../locations/api';

const SCHEMA_API_DISTANCE = z.object({
    track: z.number().nonnegative(),
    scored: z.number().nonnegative(),
});

const SCHEMA_API_STATS = z.object({
    launchLocations: z.array(SCHEMA_API_LOCATION),
    landingLocations: z.array(SCHEMA_API_LOCATION),
    yearlyStats: z.record(
        z.string().transform((key) => parseInt(key)),
        z.object({
            flightCount: z.number().nonnegative(),
            hikeandflyCount: z.number().nonnegative(),
            flightSeconds: z.number().nonnegative(),
            distance: SCHEMA_API_DISTANCE,
            distanceTrackIncomplete: z.boolean(),
            distanceScoredIncomplete: z.boolean(),
        }),
    ),
    flightCountTotal: z.number().nonnegative(),
    hikeandflyCountTotal: z.number().nonnegative(),
    flightTimeTotal: z.number().nonnegative(),
    flightDistanceTotal: SCHEMA_API_DISTANCE,
    flightsWithoutLaunchTime: z.number().nonnegative(),
});

export type Stats = z.infer<typeof SCHEMA_API_STATS>;

/**
 * Load stats from API.
 */
export async function loadApiStats(fetch: SvelteKitFetch): Promise<Stats> {
    const res = await fetch('/api/v1/stats');
    switch (res.status) {
        case 200:
            return SCHEMA_API_STATS.parse(await res.json());
        case 401:
            throw AuthenticationError.redirectToLogin(`/stats/`);
        default:
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not fetch stats from API: ${await extractResponseError(res)}`,
            );
    }
}
