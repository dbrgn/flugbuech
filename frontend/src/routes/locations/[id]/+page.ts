import {z} from 'zod';
import type {SvelteKitFetch} from '$lib';

import {SCHEMA_API_LOCATION, type Location} from '../types';
import {error} from '@sveltejs/kit';
import {goto} from '$app/navigation';

// Disable server-side rendering for this page
export const ssr = false;
export function entries() {
    return [{id: '1'}];
}

/**
 * Load location from API.
 */
async function loadApiLocation(fetch: SvelteKitFetch, id: number): Promise<Location> {
    const res = await fetch(`/api/v1/locations/${id}`);
    switch (res.status) {
        case 200:
            break;
        case 401:
            goto(`/auth/login?redirect=${encodeURI(`/locations/${id}`)}`);
            break;
        case 403:
            return error(403, `This is not your location, viewing not allowed`);
        case 404:
            return error(404, `Location with ID ${id} not found`);
        default:
            // TODO: Better error handling
            throw new Error(`Could not fetch location from API: HTTP ${res.status}`);
    }
    return SCHEMA_API_LOCATION.parse(await res.json());
}

export interface Data {
    readonly location: Location;
}

export async function load({fetch, params}): Promise<Data> {
    const id = z.coerce.number().parse(params.id);
    const location = await loadApiLocation(fetch, id);
    return {location};
}
