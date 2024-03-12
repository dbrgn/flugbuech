import {redirect, type NumericRange} from '@sveltejs/kit';

import {getLoginUrl} from '$lib/auth';

export class AuthenticationError extends Error {
    public readonly redirectUrl;

    constructor(redirectUrl?: string) {
        super('Not logged in');
        this.redirectUrl = getLoginUrl(redirectUrl);
    }

    /**
     * Helper method: Throw an error that redirects to the login page.
     */
    public static redirectToLogin(redirectUrl?: string): never {
        throw redirect(302, getLoginUrl(redirectUrl));
    }
}

/**
 * Ensure that the {@link code} is between 400 and 599. If it's outside that range, set it to
 * {@link fallback}.
 */
export function ensureClientOrServerErrorCode(
    code: number,
    fallback: NumericRange<400, 599> = 500,
): NumericRange<400, 599> {
    return code >= 400 && code < 600 ? (code as NumericRange<400, 599>) : fallback;
}
