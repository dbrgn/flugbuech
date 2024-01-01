import {loadApiLocations, type Location} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export interface Data {
    readonly locations: Location[];
}

export async function load({fetch}): Promise<Data> {
    return {locations: await loadApiLocations(fetch)};
}
