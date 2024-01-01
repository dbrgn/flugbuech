import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';

import {SCHEMA_API_LOCATION, type Location} from './types';

// Disable server-side rendering for this page
export const ssr = false;

const SCHEMA_API_LOCATIONS_RESPONSE = z.object({
    locations: z.array(SCHEMA_API_LOCATION),
});

/**
 * Load locations from API.
 */
async function loadApiLocations(fetch: SvelteKitFetch): Promise<Location[]> {
    const res = await fetch('/api/v1/locations');
    if (res.status !== 200) {
        // TODO: Better error handling
        throw new Error(`Could not fetch locations from API: HTTP ${res.status}`);
    }
    const response = SCHEMA_API_LOCATIONS_RESPONSE.parse(await res.json());
    return response.locations;
}

export interface Data {
    readonly locations: Location[];
}

export async function load({fetch}): Promise<Data> {
    return {locations: await loadApiLocations(fetch)};
}
