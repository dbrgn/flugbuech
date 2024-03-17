import {loadApiGliders, type GlidersInfo} from './api';

export type Data = GlidersInfo;

export async function load({fetch}): Promise<Data> {
    return await loadApiGliders(fetch);
}
