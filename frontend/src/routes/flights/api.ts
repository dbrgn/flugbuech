import type {SvelteKitFetch} from '$lib';
import {z} from 'zod';
import {error} from '@sveltejs/kit';
import {AuthenticationError, ensureClientOrServerErrorCode} from '$lib/errors';
import {extractResponseError} from '$lib/api';
import {SCHEMA_DATETIME_STRING} from '$lib/zod-helpers';
import {ensureXContestTracktype} from '$lib/xcontest';

const SCHEMA_API_FLIGHT_LOCATION = z.object({
    id: z.number(),
    name: z.string(),
    countryCode: z.string(),
    elevation: z.number(),
});

export type FlightLocation = z.infer<typeof SCHEMA_API_FLIGHT_LOCATION>;

const SCHEMA_API_FLIGHT_LIST_ITEM = z.object({
    id: z.number(),
    number: z.number().optional(),
    gliderName: z.string().optional(),
    launchAt: z.number().optional(),
    landingAt: z.number().optional(),
    launchTime: SCHEMA_DATETIME_STRING.optional(),
    landingTime: SCHEMA_DATETIME_STRING.optional(),
    durationSeconds: z.number().optional(),
    trackDistance: z.number().optional(),
    xcontestTracktype: z.string().transform(ensureXContestTracktype).optional(),
    xcontestDistance: z.number().optional(),
    xcontestUrl: z.string().optional(),
    comment: z.string().optional(),
    videoUrl: z.string().optional(),
    hikeandfly: z.boolean(),
    hasIgc: z.boolean(),
});

export type FlightListItem = z.infer<typeof SCHEMA_API_FLIGHT_LIST_ITEM>;

const SCHEMA_API_FLIGHTS_RESPONSE = z.object({
    flights: z.array(SCHEMA_API_FLIGHT_LIST_ITEM),
    locations: z.record(z.coerce.number(), SCHEMA_API_FLIGHT_LOCATION),
});

export type Flights = z.infer<typeof SCHEMA_API_FLIGHTS_RESPONSE>;

/**
 * Load flights from API.
 */
export async function loadApiFlights(fetch: SvelteKitFetch): Promise<Flights> {
    const res = await fetch('/api/v1/flights');
    switch (res.status) {
        case 200:
            return SCHEMA_API_FLIGHTS_RESPONSE.parse(await res.json());
        case 401:
            throw AuthenticationError.redirectToLogin(`/flights/`);
        default: {
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not fetch flights from API: ${await extractResponseError(res)}`,
            );
        }
    }
}
const SCHEMA_API_FLIGHT = z.object({
    id: z.number(),
    number: z.number().optional(),
    gliderName: z.string().optional(),
    launchAt: SCHEMA_API_FLIGHT_LOCATION.optional(),
    landingAt: SCHEMA_API_FLIGHT_LOCATION.optional(),
    launchTime: SCHEMA_DATETIME_STRING.optional(),
    landingTime: SCHEMA_DATETIME_STRING.optional(),
    durationSeconds: z.number().optional(),
    trackDistance: z.number().optional(),
    xcontestTracktype: z.string().transform(ensureXContestTracktype).optional(),
    xcontestDistance: z.number().optional(),
    xcontestUrl: z.string().optional(),
    comment: z.string().optional(),
    videoUrl: z.string().optional(),
    hikeandfly: z.boolean(),
    hasIgc: z.boolean(),
});

export type Flight = z.infer<typeof SCHEMA_API_FLIGHT>;

/**
 * Load flight from API.
 */
export async function loadApiFlight(fetch: SvelteKitFetch, id: number): Promise<Flight> {
    const res = await fetch(`/api/v1/flights/${id}`);
    switch (res.status) {
        case 200:
            return SCHEMA_API_FLIGHT.parse(await res.json());
        case 401:
            throw AuthenticationError.redirectToLogin(`/flights/${id}`);
        case 403:
            return error(403, `This is not your flight, viewing not allowed`);
        case 404:
            return error(404, `Flight with ID ${id} not found`);
        default:
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not fetch flight from API: ${await extractResponseError(res)}`,
            );
    }
}
