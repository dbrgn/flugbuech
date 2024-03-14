import {goto} from '$app/navigation';

import type {LoginState} from './stores';

export function getLoginUrl(redirectUrl?: string): string {
    return redirectUrl !== undefined
        ? `/auth/login?redirect=${encodeURI(redirectUrl)}`
        : `/auth/login`;
}

export function requireLogin(loginState: LoginState | undefined, redirectUrl: string): void {
    if (loginState?.username === undefined) {
        // Not logged in
        goto(getLoginUrl(redirectUrl));
    }
}
