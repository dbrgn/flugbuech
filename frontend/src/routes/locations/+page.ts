import {PUBLIC_API_URL} from '$env/static/public';

import {z} from 'zod';

// Disable server-side rendering for this page
export const ssr = false;

const SCHEMA_API_LOCATIONS = z.array(
    z.object({
        id: z.number(),
        name: z.string(),
        countryCode: z.string(),
        elevation: z.number(),
        coordinates: z
            .object({
                lon: z.number(),
                lat: z.number(),
            })
            .optional(),
        flightCount: z.number(),
    }),
);

type Locations = z.infer<typeof SCHEMA_API_LOCATIONS>;

/**
 * Load locations from API.
 */
async function loadApiLocations(): Promise<Locations> {
    const res = await fetch(new URL('locations', PUBLIC_API_URL), {credentials: 'include'});
    if (res.status !== 200) {
        // TODO: Better error handling
        throw new Error(`Could not fetch locations from API: HTTP ${res.status}`);
    }
    return SCHEMA_API_LOCATIONS.parse(await res.json());
}

export interface Data {
    readonly locations: Locations;
}

export async function load(): Promise<Data> {
    return {locations: await loadApiLocations()};
}
