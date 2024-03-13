import {unreachable} from './assert';

export type XContestTracktype = 'free_flight' | 'flat_triangle' | 'fai_triangle';

export function isXContestTracktype(value: string): value is XContestTracktype {
    return ['free_flight', 'flat_triangle', 'fai_triangle'].includes(value);
}

export function ensureXContestTracktype(value: string): XContestTracktype {
    if (isXContestTracktype(value)) {
        return value;
    }
    throw new Error(`String "${value}" is not a valid XContest track type`);
}

export function tracktypeName(tracktype: XContestTracktype): string {
    switch (tracktype) {
        case 'free_flight':
            return 'Free Flight';
        case 'flat_triangle':
            return 'Flat Triangle';
        case 'fai_triangle':
            return 'FAI Triangle';
        default:
            return unreachable(tracktype);
    }
}
