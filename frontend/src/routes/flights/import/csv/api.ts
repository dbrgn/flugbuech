import {z} from 'zod';

import {apiPostBlob, extractResponseError} from '$lib/api';
import {AuthenticationError} from '$lib/errors';
import {ensureXContestTracktype} from '$lib/xcontest';

/**
 * List of all possible CSV validation message fields.
 */
const API_MESSAGE_FIELDS = [
    'number',
    'glider-id',
    'launch-at',
    'landing-at',
    'launch-time',
    'landing-time',
    'xcontest-tracktype',
    'xcontest-url',
] as const;

const SCHEMA_API_CSV_FLIGHT_PREVIEW = z.object({
    csvRow: z.number(),
    number: z.number().optional(),
    gliderId: z.number().optional(),
    launchAt: z.number().optional(),
    landingAt: z.number().optional(),
    launchTime: z
        .string()
        .transform((datetime) => new Date(datetime))
        .optional(),
    landingTime: z
        .string()
        .transform((datetime) => new Date(datetime))
        .optional(),
    trackDistance: z.number().optional(),
    xcontestTracktype: z.string().transform(ensureXContestTracktype).optional(),
    xcontestDistance: z.number().optional(),
    xcontestUrl: z.string().optional(),
    comment: z.string().optional(),
    videoUrl: z.string().optional(),
    hikeandfly: z.boolean(),
});

const SCHEMA_API_MESSAGE = z.object({
    csvRow: z.number().optional(),
    field: z.enum(API_MESSAGE_FIELDS).optional(),
    message: z.string(),
});

/**
 * A CSV validation message (either a warning or an error).
 */
type ApiMessage = z.infer<typeof SCHEMA_API_MESSAGE>;

interface GroupedApiMessages {
    readonly warnings: ApiMessage[];
    readonly errors: ApiMessage[];
}

export const NO_ROW = Symbol('no-row');
export const NO_FIELD = Symbol('no-field');

/**
 * All messages first grouped by row, then by field, and then by type.
 */
interface MessagesByRowAndField {
    [NO_ROW]: GroupedApiMessages;
    [row: number]: {
        [field in (typeof API_MESSAGE_FIELDS)[number] | typeof NO_FIELD]: GroupedApiMessages;
    };
}

export function defaultMessageGroup(): GroupedApiMessages {
    return {warnings: [], errors: []};
}

function defaultRowRecord(): MessagesByRowAndField[number] {
    const record: Record<string, GroupedApiMessages> = {[NO_FIELD]: defaultMessageGroup()};
    for (const key of API_MESSAGE_FIELDS) {
        record[key] = defaultMessageGroup();
    }
    return record as MessagesByRowAndField[number];
}

const SCHEMA_API_ANALYZE_RESULT = z
    .object({
        warnings: z.array(SCHEMA_API_MESSAGE),
        errors: z.array(SCHEMA_API_MESSAGE),
        flights: z.array(SCHEMA_API_CSV_FLIGHT_PREVIEW),
    })
    .transform((result) => {
        // Aggregate warnings and errors into a structore easier to use in the frontend
        const messagesByRowAndField: MessagesByRowAndField = {[NO_ROW]: defaultMessageGroup()};

        function processMessage(message: ApiMessage, type: 'warnings' | 'errors') {
            // If message is not associated with any row, add it to the NO_ROW group
            if (message.csvRow === undefined) {
                messagesByRowAndField[NO_ROW][type].push(message);
                return;
            }

            // Ensure that a row record exists
            const rowRecord = messagesByRowAndField[message.csvRow] ?? defaultRowRecord();
            messagesByRowAndField[message.csvRow] = rowRecord;

            // Push message
            console.log(`Row ${type} record for field ${message.field}`, rowRecord);
            rowRecord[message.field ?? NO_FIELD][type].push(message);
        }

        for (const warning of result.warnings) {
            processMessage(warning, 'warnings');
        }
        for (const error of result.errors) {
            processMessage(error, 'errors');
        }

        return {...result, messagesByRowAndField};
    });

const SCHEMA_API_IMPORT_RESULT = z.object({
    success: z.boolean(),
});

export type CsvAnalyzeResult = z.infer<typeof SCHEMA_API_ANALYZE_RESULT>;
export type CsvImportResult = z.infer<typeof SCHEMA_API_IMPORT_RESULT>;

/**
 * Analyze CSV file through API.
 */
export async function analyzeCsv(blob: Blob): Promise<CsvAnalyzeResult> {
    const res = await apiPostBlob('/api/v1/flights/add/import_csv?mode=analyze', blob);
    switch (res.status) {
        case 200:
            return SCHEMA_API_ANALYZE_RESULT.parse(await res.json());
        case 401:
            throw new AuthenticationError();
        default:
            throw new Error(`Could not submit CSV to API: ${await extractResponseError(res)}`);
    }
}

/**
 * Import CSV file through API.
 */
export async function importCsv(blob: Blob): Promise<CsvImportResult> {
    const res = await apiPostBlob('/api/v1/flights/add/import_csv?mode=import', blob);
    switch (res.status) {
        case 200:
            return SCHEMA_API_IMPORT_RESULT.parse(await res.json());
        case 401:
            throw new AuthenticationError();
        default:
            throw new Error(`Could not submit CSV to API: ${await extractResponseError(res)}`);
    }
}
