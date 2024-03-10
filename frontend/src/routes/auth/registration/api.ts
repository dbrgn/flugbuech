import {apiPost, extractApiError, extractResponseError} from '$lib/api';
import {ensureClientOrServerErrorCode} from '$lib/errors';
import {error} from '@sveltejs/kit';

export type RegistrationResult =
    | {
          readonly success: true;
      }
    | {
          readonly success: false;
          readonly errorReason: string;
          readonly errorDescription: string;
      };

/**
 * Sign up in via API.
 */
export async function apiRegister(
    username: string,
    email: string,
    password: string,
): Promise<RegistrationResult> {
    const res = await apiPost('/api/v1/auth/registration', {
        username,
        email,
        password,
    });
    switch (res.status) {
        case 204:
            return {success: true};
        case 400: {
            try {
                const apiError = await extractApiError(res);
                return {
                    success: false,
                    errorReason: apiError.error.reason,
                    errorDescription: apiError.error.description,
                };
            } catch (e) {
                throw error(
                    ensureClientOrServerErrorCode(res.status),
                    `Could not register: Unknown error response`,
                );
            }
        }
        default:
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not register: ${await extractResponseError(res)}`,
            );
    }
}
