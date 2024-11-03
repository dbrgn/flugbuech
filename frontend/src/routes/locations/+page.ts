import {loadApiLocations, type Location} from './api';
import {type PageLoadEvent} from './$types';

export interface Data {
    readonly locations: Location[];
}

export async function load({fetch}: PageLoadEvent): Promise<Data> {
    return {locations: await loadApiLocations(fetch)};
}
