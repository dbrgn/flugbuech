import {z} from 'zod';

/**
 * Common headers that should be used for every fetch API request.
 */
const COMMON_HEADERS = {
    accept: 'application/json, text/plain, */*;q=0.8',
};

/**
 * Schema of an API error response.
 */
const SCHEMA_API_ERROR = z.object({
    error: z.object({
        code: z.number(),
        reason: z.string(),
        description: z.string(),
    }),
});

/**
 * Send a POST request to the API.
 */
export async function apiPost(
    url: string,
    requestBody?: Record<string, unknown>,
    fetchImpl = fetch,
): Promise<Response> {
    return await fetchImpl(url, {
        method: 'POST',
        cache: 'no-cache',
        credentials: 'include',
        headers: {
            'content-type': 'application/json',
            ...COMMON_HEADERS,
        },
        body: requestBody !== undefined ? JSON.stringify(requestBody) : undefined,
    });
}

/**
 * Send a DELETE request to the API.
 */
export async function apiDelete(url: string, fetchImpl = fetch): Promise<Response> {
    return await fetchImpl(url, {
        method: 'DELETE',
        cache: 'no-cache',
        credentials: 'include',
        headers: {
            ...COMMON_HEADERS,
        },
    });
}

/**
 * Extract the error message from an error response.
 */
export async function extractResponseError(response: Response): Promise<string> {
    try {
        const error = SCHEMA_API_ERROR.parse(await response.json());
        return `HTTP ${response.status} (${error.error.reason}): ${error.error.description}`;
    } catch (error) {
        console.warn('Failed to parse API response as error:', error);
        return `HTTP ${response.status}`;
    }
}
