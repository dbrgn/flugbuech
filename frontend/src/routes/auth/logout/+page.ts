import {addFlash, refreshLoginState} from '$lib/stores';

import {goto} from '$app/navigation';

import {apiLogout} from './api';

export async function load({fetch}): Promise<void> {
    await apiLogout(fetch);

    // Logout successful! Add flash.
    addFlash({
        message: 'Logged out, goodbye and happy landings!',
        severity: 'success',
        icon: 'fa-circle-check',
    });

    // Refresh login state
    refreshLoginState();

    // Redirect to home
    await goto('/', {invalidateAll: true, replaceState: true});
}
