import {z} from 'zod';

export const SCHEMA_DATETIME_STRING = z
    .string()
    .transform((dateTimeString) => new Date(dateTimeString));
