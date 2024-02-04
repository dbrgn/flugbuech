import {loadApiGliders, type Gliders} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export type Data = Gliders;

export async function load({fetch}): Promise<Data> {
    return await loadApiGliders(fetch);
}
