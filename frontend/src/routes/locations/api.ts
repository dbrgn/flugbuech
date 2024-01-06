import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';
import {error} from '@sveltejs/kit';
import {AuthenticationError} from '$lib/errors';

const SCHEMA_API_LOCATION = z.object({
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
});

export type Location = z.infer<typeof SCHEMA_API_LOCATION>;

const SCHEMA_API_LOCATIONS_RESPONSE = z.object({
    locations: z.array(SCHEMA_API_LOCATION),
});

/**
 * Load locations from API.
 */
export async function loadApiLocations(fetch: SvelteKitFetch): Promise<Location[]> {
    const res = await fetch('/api/v1/locations');
    if (res.status !== 200) {
        // TODO: Better error handling
        throw new Error(`Could not fetch locations from API: HTTP ${res.status}`);
    }
    const response = SCHEMA_API_LOCATIONS_RESPONSE.parse(await res.json());
    return response.locations;
}

/**
 * Load location from API.
 */
export async function loadApiLocation(fetch: SvelteKitFetch, id: number): Promise<Location> {
    const res = await fetch(`/api/v1/locations/${id}`);
    switch (res.status) {
        case 200:
            return SCHEMA_API_LOCATION.parse(await res.json());
        case 401:
            throw AuthenticationError.redirectToLogin(`/locations/${id}`);
        case 403:
            return error(403, `This is not your location, viewing not allowed`);
        case 404:
            return error(404, `Location with ID ${id} not found`);
        default:
            // TODO: Better error handling
            throw new Error(`Could not fetch location from API: HTTP ${res.status}`);
    }
}
