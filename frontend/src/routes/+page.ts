import {z} from 'zod';

import {PUBLIC_API_URL} from '$env/static/public';

// Prerender this page
export const prerender = true;

export interface GlobalStats {
    /**
     * Number of registered users.
     */
    readonly userCount: number;
    /**
     * Number of registered gliders.
     */
    readonly gliderCount: number;
    /**
     * Total flights in database.
     */
    readonly flightCount: number;
}

const SCHEMA_API_RESPONSE = z.object({
    user_count: z.number(),
    glider_count: z.number(),
    flight_count: z.number(),
});

/**
 * Load global stats from API.
 */
export async function _loadApiStats(): Promise<GlobalStats> {
    const res = await fetch(new URL('global-stats', PUBLIC_API_URL));
    if (res.status !== 200) {
        // TODO: Better error handling
        throw new Error(`Could not reach API: HTTP ${res.status}`);
    }
    const parsed = SCHEMA_API_RESPONSE.parse(await res.json());
    return {
        userCount: parsed.user_count,
        gliderCount: parsed.glider_count,
        flightCount: parsed.flight_count,
    };
}
