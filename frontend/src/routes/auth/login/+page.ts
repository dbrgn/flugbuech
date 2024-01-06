import {getCookiesMap} from '$lib/cookies';
import {redirect} from '@sveltejs/kit';

// Disable server-side rendering for this page
export const ssr = false;

export async function load(): Promise<void> {
    const cookies = getCookiesMap(document.cookie);
    // Note: The user_id cookie is HTTP only, so we cannot fetch it.
    //       We're using the user_name cookie as proxy for determining auth state.
    if (cookies['user_name'] !== undefined) {
        // User is already logged in, redirect to profile
        // TODO: If redirect parameter is set, use it
        throw redirect(302, '/profile');
    }
}
