import {apiPost, extractResponseError} from '$lib/api';

export interface LoginResult {
    readonly success: boolean;
}

/**
 * Log in via API.
 */
export async function apiLogin(username: string, password: string): Promise<LoginResult> {
    const res = await apiPost('/api/v1/auth/login', {
        username,
        password,
    });
    switch (res.status) {
        case 204:
            return {success: true};
        case 403:
            return {success: false};
        default:
            throw new Error(`Could not log in: ${await extractResponseError(res)}`);
    }
}
