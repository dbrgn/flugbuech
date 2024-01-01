import {z} from 'zod';

import {loadApiLocation, type Location} from '../api';

// Disable server-side rendering for this page
export const ssr = false;
export function entries() {
    return [{id: '1'}];
}

export interface Data {
    readonly location: Location;
}

export async function load({fetch, params}): Promise<Data> {
    const id = z.coerce.number().parse(params.id);
    const location = await loadApiLocation(fetch, id);
    return {location};
}
