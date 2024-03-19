import {error} from '@sveltejs/kit';

import {apiPost, extractApiError, extractResponseError, type ApiSuccessOrError} from '$lib/api';
import {ensureClientOrServerErrorCode} from '$lib/errors';

/**
 * Change password via API.
 */
export async function apiChangePassword(
    currentPassword: string,
    newPassword: string,
): Promise<ApiSuccessOrError> {
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
