import {z} from 'zod';

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

const SCHEMA_API_STATS = z.object({
    user_count: z.number(),
    glider_count: z.number(),
    flight_count: z.number(),
});

/**
 * Load global stats from API.
 */
export async function _loadApiStats(): Promise<GlobalStats> {
    const res = await fetch('/api/v1/global-stats');
    if (res.status !== 200) {
        // TODO: Better error handling
        throw new Error(`Could not reach API: HTTP ${res.status}`);
    }
    const parsed = SCHEMA_API_STATS.parse(await res.json());
    return {
        userCount: parsed.user_count,
        gliderCount: parsed.glider_count,
        flightCount: parsed.flight_count,
    };
}
