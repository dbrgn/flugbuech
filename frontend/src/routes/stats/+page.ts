import {loadApiStats, type Stats} from './api';

// Disable server-side rendering for this page
export const ssr = false;

export type Data = Stats;

export async function load({fetch}): Promise<Data> {
    return await loadApiStats(fetch);
}
