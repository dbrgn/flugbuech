import {z} from 'zod';

export const SCHEMA_API_ERROR = z.object({
    error: z.object({
        code: z.number(),
        reason: z.string(),
        description: z.string(),
    }),
});
