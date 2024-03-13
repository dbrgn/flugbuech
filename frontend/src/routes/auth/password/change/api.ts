import {apiPost, extractApiError, extractResponseError} from '$lib/api';
import {ensureClientOrServerErrorCode} from '$lib/errors';
import {error} from '@sveltejs/kit';

export type PasswordChangeResult =
    | {
          readonly success: true;
      }
    | {
          readonly success: false;
          readonly errorReason: string;
          readonly errorDescription: string;
      };

/**
 * Change password via API.
 */
export async function apiChangePassword(
    currentPassword: string,
    newPassword: string,
): Promise<PasswordChangeResult> {
    const res = await apiPost('/api/v1/auth/password/change', {
        currentPassword,
        newPassword,
    });
    switch (res.status) {
        case 204:
            return {success: true};
        case 422: {
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
                    `Could not change password: Unknown error response`,
                );
            }
        }
        default:
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not change password: ${await extractResponseError(res)}`,
            );
    }
}
