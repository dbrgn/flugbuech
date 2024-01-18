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
