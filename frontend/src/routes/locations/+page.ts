import {loadApiLocations, type Location} from './api';

export interface Data {
    readonly locations: Location[];
}

export async function load({fetch}): Promise<Data> {
    return {locations: await loadApiLocations(fetch)};
}
