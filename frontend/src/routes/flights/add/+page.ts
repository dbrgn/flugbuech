import {loadApiGliders, type Glider} from '../../gliders/api';
import {loadApiFlights} from '../api';

// Disable server-side rendering for this page
export const ssr = false;

export interface Data {
    /**
     * A sorted array of flight numbers associated with existing flights.
     */
    readonly existingFlightNumbers: number[];
    /**
     * The user's gliders.
     */
    readonly gliders: Glider[];
    /**
     * The user's last used glider ID.
     */
    readonly lastGliderId?: number;
}

export async function load({fetch}): Promise<Data> {
    // TODO: Dedicated endpoint for the values we need
    // TODO: Fetch last glider ID
    const flights = await loadApiFlights(fetch);
    const gliders = await loadApiGliders(fetch);

    const existingFlightNumbers = flights.flights
        .map((flight) => flight.number)
        .filter((n): n is number => n !== undefined)
        .sort();

    return {
        existingFlightNumbers,
        gliders: gliders.gliders,
        lastGliderId: gliders.lastGliderId,
    };
}
