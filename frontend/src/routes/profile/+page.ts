import {loadApiProfile, type Profile} from './api';

export interface Data {
    readonly profile: Profile;
}

export async function load({fetch}): Promise<Data> {
    return {profile: await loadApiProfile(fetch)};
}
