import {z} from 'zod';

import {loadApiGliders, type Glider} from '../../api';
import {error} from '@sveltejs/kit';

// Disable server-side rendering for this page
export const ssr = false;
export function entries() {
    return [{id: '1'}];
}

export interface Data {
    readonly glider: Glider;
}

export async function load({fetch, params}): Promise<Data> {
    const id = z.coerce.number().parse(params.id);
    // TODO: Add endpoint to fetch glider by ID
    const gliders = await loadApiGliders(fetch);
    const glider = gliders.filter((glider) => glider.id === id)[0];
    if (glider === undefined) {
        throw error(404, `Glider with ID ${id} not found`);
    }
    return {glider};
}
