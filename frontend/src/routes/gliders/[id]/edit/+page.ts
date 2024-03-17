import {error} from '@sveltejs/kit';
import {z} from 'zod';

import {loadApiGliders, type Glider} from '../../api';

// Disable server-side rendering for this page
export const ssr = false;

// Dynamic URL, don't prerender
export const prerender = false;

export interface Data {
    readonly glider: Glider;
}

export async function load({fetch, params}): Promise<Data> {
    const id = z.coerce.number().parse(params.id);
    // TODO: Add endpoint to fetch glider by ID
    const info = await loadApiGliders(fetch);
    const glider = info.gliders.filter((glider) => glider.id === id)[0];
    if (glider === undefined) {
        throw error(404, `Glider with ID ${id} not found`);
    }
    return {glider};
}
