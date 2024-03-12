import {loadApiGliders, type GlidersInfo} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export type Data = GlidersInfo;

export async function load({fetch}): Promise<Data> {
    return await loadApiGliders(fetch);
}
