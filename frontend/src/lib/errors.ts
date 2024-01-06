import {redirect} from '@sveltejs/kit';

export class AuthenticationError extends Error {
    public readonly redirectUrl;

    constructor(redirectUrl?: string) {
        super('Not logged in');
        this.redirectUrl = AuthenticationError.getLoginUrl(redirectUrl);
    }

    private static getLoginUrl(redirectUrl?: string): string {
        return redirectUrl !== undefined
            ? `/auth/login?redirect=${encodeURI(redirectUrl)}`
            : `/auth/login`;
    }

    /**
     * Helper method: Throw an error that redirects to the login page.
     */
    public static redirectToLogin(redirectUrl?: string): never {
        throw redirect(302, AuthenticationError.getLoginUrl(redirectUrl));
    }
}
