import {z} from 'zod';

import {apiPostBlob, extractResponseError} from '$lib/api';
import {AuthenticationError} from '$lib/errors';
import {ensureXContestTracktype} from '$lib/xcontest';

const SCHEMA_API_CSV_FLIGHT_PREVIEW = z.object({
    number: z.number().optional(),
    gliderId: z.number().optional(),
    launchAt: z.number().optional(),
    landingAt: z.number().optional(),
    launchTime: z.string().optional(),
    landingTime: z.string().optional(),
    trackDistance: z.number().optional(),
    xcontestTracktype: z.string().transform(ensureXContestTracktype).optional(),
    xcontestDistance: z.number().optional(),
    xcontestUrl: z.string().optional(),
    comment: z.string().optional(),
    videoUrl: z.string().optional(),
    hikeandfly: z.boolean(),
});

const SCHEMA_API_ANALYZE_RESULT = z.object({
    warnings: z.array(z.string()),
    errors: z.array(z.string()),
    flights: z.array(SCHEMA_API_CSV_FLIGHT_PREVIEW),
});

const SCHEMA_API_IMPORT_RESULT = z.object({
    success: z.boolean(),
});

type CsvAnalyzeResult = z.infer<typeof SCHEMA_API_ANALYZE_RESULT>;
type CsvImportResult = z.infer<typeof SCHEMA_API_IMPORT_RESULT>;

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
