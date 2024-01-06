import {loadApiGliders, type Glider} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export interface Data {
    readonly gliders: Glider[];
}

export async function load({fetch}): Promise<Data> {
    return {gliders: await loadApiGliders(fetch)};
}
