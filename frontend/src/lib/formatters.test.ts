import {describe, it, expect} from 'vitest';

import {formatDistance, formatDuration} from './formatters';

describe('formatDuration', () => {
    const testCases = [
        [0, '00:00'],
        [60 * 7, '00:07'],
        [60 * 59, '00:59'],
        [60 * 60, '01:00'],
        [60 * 61, '01:01'],
        [60 * 642, '10:42'],
        [3600 * 99 + 3599, '99:59'],
        [3600 * 99 + 3600, '100:00'],
    ] as const;

    for (const [seconds, expected] of testCases) {
        it(`formats ${seconds} seconds`, () => {
            expect(formatDuration(seconds)).to.equal(expected);
        });
    }
});

describe('formatDistance', () => {
    const testCases = [
        [0, '0 km'],
        [3, '3 km'],
        [3.1, '3.1 km'],
        [3.45, '3.45 km'],
        [3.4521, '3.45 km'],
    ] as const;

    for (const [km, expected] of testCases) {
        it(`formats ${km} km`, () => {
            expect(formatDistance(km)).to.equal(expected);
        });
    }
});
