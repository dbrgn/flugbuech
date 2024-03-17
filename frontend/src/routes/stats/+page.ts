import {loadApiStats, type Stats} from './api';

export type Data = Stats;

export async function load({fetch}): Promise<Data> {
    return await loadApiStats(fetch);
}
