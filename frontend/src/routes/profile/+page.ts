import {loadApiProfile, type Profile} from './api';
import {type PageLoadEvent} from './$types';

export interface Data {
    readonly profile: Profile;
}

export async function load({fetch}: PageLoadEvent): Promise<Data> {
    return {profile: await loadApiProfile(fetch)};
}
