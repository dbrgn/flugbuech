import {z} from 'zod';

import {apiPostBlob, extractResponseError} from '$lib/api';
import {AuthenticationError} from '$lib/errors';

const SCHEMA_API_ANALYZE_RESULT = z.object({
    warnings: z.array(z.string()).optional(),
    errors: z.array(z.string()).optional(),
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
    const res = await apiPostBlob('/api/v1/flights/add/csv/import?mode=analyze', blob);
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
    const res = await apiPostBlob('/api/v1/flights/add/csv/import?mode=import', blob);
    switch (res.status) {
        case 200:
            return SCHEMA_API_IMPORT_RESULT.parse(await res.json());
        case 401:
            throw new AuthenticationError();
        default:
            throw new Error(`Could not submit CSV to API: ${await extractResponseError(res)}`);
    }
}
