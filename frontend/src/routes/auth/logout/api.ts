import {error} from '@sveltejs/kit';

import type {SvelteKitFetch} from '$lib';
import {apiPost, extractResponseError} from '$lib/api';
import {ensureClientOrServerErrorCode} from '$lib/errors';

/**
 * Log out via API.
 */
export async function apiLogout(fetch: SvelteKitFetch): Promise<void> {
    const res = await apiPost('/api/v1/auth/logout', undefined, fetch);
    switch (res.status) {
        case 204:
            return;
        default:
            throw error(
                ensureClientOrServerErrorCode(res.status),
                `Could not log out: ${await extractResponseError(res)}`,
            );
    }
}
