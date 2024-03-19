import {z} from 'zod';

import {loadApiGliders, type Glider} from '../../../gliders/api';
import {loadApiLocations} from '../../../locations/api';
import {loadApiFlights, type FlightLocation, loadApiFlight, type Flight} from '../../api';

// Dynamic URL, don't prerender
export const prerender = false;

export interface Data {
    /**
     * The flight that is being edited.
     */
    readonly flight: Flight;
    /**
     * A sorted array of flight numbers associated with existing flights.
     */
    readonly existingFlightNumbers: number[];
    /**
     * The user's gliders.
     */
    readonly gliders: Glider[];
    /**
     * The user's locations.
     */
    readonly locations: FlightLocation[];
}

export async function load({fetch, params}): Promise<Data> {
    // Load flight to be edited
    const id = z.coerce.number().parse(params.id);
    const flight = await loadApiFlight(fetch, id);

    // TODO: Dedicated endpoint for the values we need
    const flights = await loadApiFlights(fetch);
    const gliders = await loadApiGliders(fetch);
    const locations = await loadApiLocations(fetch);
    const existingFlightNumbers = flights.flights
        .map((f) => f.number)
        .filter((n): n is number => n !== undefined)
        .sort();

    return {
        flight,
        existingFlightNumbers,
        gliders: gliders.gliders,
        locations,
    };
}
