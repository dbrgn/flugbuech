import {loadApiFlights, type FlightListItem, type FlightLocation} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export interface Data {
    readonly flights: FlightListItem[];
    readonly locations: Record<number, FlightLocation>;
}

export async function load({fetch}): Promise<Data> {
    return await loadApiFlights(fetch);
}
