import {loadApiGliders, type GlidersInfo} from './api';
import {type PageLoadEvent} from './$types';

export type Data = GlidersInfo;

export async function load({fetch}: PageLoadEvent): Promise<Data> {
    return await loadApiGliders(fetch);
}
