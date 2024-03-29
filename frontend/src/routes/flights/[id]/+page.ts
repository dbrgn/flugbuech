import {z} from 'zod';

import {loadApiFlight, type Flight} from '../api';

// Dynamic URL, don't prerender
export const prerender = false;

export interface Data {
    readonly flight: Flight;
}

export async function load({fetch, params}): Promise<Data> {
    const id = z.coerce.number().parse(params.id);
    const flight = await loadApiFlight(fetch, id);
    return {flight};
}
