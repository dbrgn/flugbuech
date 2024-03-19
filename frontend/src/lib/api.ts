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
 * An API error response.
 */
export type ApiError = z.infer<typeof SCHEMA_API_ERROR>;

/**
 * Send a JSON POST request to the API.
 */
export async function apiPost(
    url: string,
    requestBody: Record<string, unknown> | undefined,
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
 * Send a Blob POST request to the API.
 */
export async function apiPostBlob(
    url: string,
    requestBody: Blob,
    fetchImpl = fetch,
): Promise<Response> {
    return await fetchImpl(url, {
        method: 'POST',
        cache: 'no-cache',
        credentials: 'include',
        headers: {
            'content-type': 'application/octet-stream',
            ...COMMON_HEADERS,
        },
        body: requestBody,
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
 * Extract the {@link ApiError} from an error response.
 */
export async function extractApiError(response: Response): Promise<ApiError> {
    return SCHEMA_API_ERROR.parse(await response.json());
}

/**
 * Extract an error message from an error response.
 */
export async function extractResponseError(response: Response): Promise<string> {
    try {
        const error = await extractApiError(response);
        return `HTTP ${response.status} (${error.error.reason}): ${error.error.description}`;
    } catch (error) {
        console.warn('Failed to parse API response as error:', error);
        return `HTTP ${response.status}`;
    }
}

/**
 * A type containing a success flag, and - in case of an error - the error details.
 */
export type ApiSuccessOrError =
    | {
          readonly success: true;
      }
    | {
          readonly success: false;
          readonly errorReason: string;
          readonly errorDescription: string;
      };
