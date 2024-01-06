import {loadApiProfile, type Profile} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export interface Data {
    readonly profile: Profile;
}

export async function load({fetch}): Promise<Data> {
    return {profile: await loadApiProfile(fetch)};
}
