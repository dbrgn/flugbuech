import {getCookiesMap} from '$lib/cookies';

// Enable prerendering and disable SSR
export const prerender = true;
export const ssr = false;

// Always use trailing slash
export const trailingSlash = 'always';

interface LayoutData {
    /**
     * The username indicates (with some certainty) whether or not we are logged in.
     */
    readonly username: string | undefined;
}

export async function load(): Promise<LayoutData> {
    const cookies = getCookiesMap(document.cookie);

    return {
        username: cookies['user_name'],
    };
}
