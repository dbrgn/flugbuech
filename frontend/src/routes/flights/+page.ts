import {loadApiFlights, type FlightListItem, type FlightLocation} from './api';
import {type PageLoadEvent} from './$types';

export interface Data {
    readonly flights: FlightListItem[];
    readonly locations: Record<number, FlightLocation>;
}

export async function load({fetch}: PageLoadEvent): Promise<Data> {
    return await loadApiFlights(fetch);
}
