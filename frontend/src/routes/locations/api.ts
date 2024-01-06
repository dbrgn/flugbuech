import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';
import {error} from '@sveltejs/kit';
import {AuthenticationError, ensureClientOrServerErrorCode} from '$lib/errors';
import {extractResponseError} from '$lib/api';

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
    switch (res.status) {
        case 200: {
            const response = SCHEMA_API_LOCATIONS_RESPONSE.parse(await res.json());
            return response.locations;
        }
        case 401:
            throw AuthenticationError.redirectToLogin(`/locations/`);
        default: {
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not fetch locations from API: ${await extractResponseError(res)}`,
            );
        }
    }
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
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not fetch location from API: ${await extractResponseError(res)}`,
            );
    }
}
