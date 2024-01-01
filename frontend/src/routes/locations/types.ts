import {z} from 'zod';

export const SCHEMA_API_LOCATION = z.object({
    id: z.number(),
    name: z.string(),
    countryCode: z.string(),
    elevation: z.number(),
    coordinates: z
        .object({
            lon: z.number(),
            lat: z.number(),
        })
        .optional(),
    flightCount: z.number(),
});

export type Location = z.infer<typeof SCHEMA_API_LOCATION>;
