import {initializeI18n} from '$lib/i18n';
import {addFlash, refreshLoginState} from '$lib/stores';

import {goto} from '$app/navigation';

import {apiLogout} from './api';
import {type PageLoadEvent} from './$types';

export async function load({fetch}: PageLoadEvent): Promise<void> {
    await apiLogout(fetch);

    const {i18n} = initializeI18n();

    // Logout successful! Add flash.
    addFlash({
        message: i18n.t('auth.prose--logged-out'),
        severity: 'success',
        icon: 'fa-circle-check',
    });

    // Refresh login state
    refreshLoginState();

    // Redirect to home
    await goto('/', {invalidateAll: true, replaceState: true});
}
