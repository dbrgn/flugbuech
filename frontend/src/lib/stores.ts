import {writable} from 'svelte/store';
import {getCookiesMap} from './cookies';

// Flashes

export interface Flash {
    readonly message: string;
    readonly severity: 'info' | 'success' | 'warning' | 'error';
    readonly icon?: `fa-${string}`;
}

export const flashes = writable<Flash[]>([]);

export function addFlash(flash: Flash): void {
    flashes.update((f) => [...f, flash]);
}

// Login state

interface LoginState {
    /**
     * The username indicates (with some certainty) whether or not we are logged in.
     */
    readonly username: string | undefined;
}

/**
 * Store containing global user login state
 */
export const loginState = writable<LoginState | undefined>();

/**
 * Refresh the login state based on cookies.
 */
export function refreshLoginState() {
    const cookies = getCookiesMap(document.cookie);
    loginState.set({username: cookies['user_name']});
}
