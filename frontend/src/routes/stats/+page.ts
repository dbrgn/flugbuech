import {loadApiStats, type Stats} from './api';
import {type PageLoadEvent} from './$types';

export type Data = Stats;

export async function load({fetch}: PageLoadEvent): Promise<Data> {
    return await loadApiStats(fetch);
}
