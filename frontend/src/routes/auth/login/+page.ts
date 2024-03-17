import {redirect} from '@sveltejs/kit';

import {getCookiesMap} from '$lib/cookies';
import {sanitizeRedirectPath} from '$lib/urls';

export async function load({url}): Promise<void> {
    const cookies = getCookiesMap(document.cookie);
    // Note: The user_id cookie is HTTP only, so we cannot fetch it.
    //       We're using the user_name cookie as proxy for determining auth state.
    if (cookies['user_name'] !== undefined) {
        // User is already logged in, redirect to profile (or to user-defined page)
        throw redirect(302, sanitizeRedirectPath(url.searchParams.get('redirect'), '/profile'));
    }
}
