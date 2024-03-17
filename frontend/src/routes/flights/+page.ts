import {loadApiFlights, type FlightListItem, type FlightLocation} from './api';

export interface Data {
    readonly flights: FlightListItem[];
    readonly locations: Record<number, FlightLocation>;
}

export async function load({fetch}): Promise<Data> {
    return await loadApiFlights(fetch);
}
