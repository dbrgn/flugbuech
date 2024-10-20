import {goto} from '$app/navigation';

import type {LoginState} from './stores';

export function getLoginUrl(redirectUrl?: string): string {
    return redirectUrl !== undefined
        ? `/auth/login?redirect=${encodeURI(redirectUrl)}`
        : `/auth/login`;
}

export async function requireLogin(
    loginState: LoginState | undefined,
    redirectUrl: string,
): Promise<void> {
    if (loginState?.username === undefined) {
        // Not logged in
        await goto(getLoginUrl(redirectUrl));
    }
}
