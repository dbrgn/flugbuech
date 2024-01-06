import {describe, it, expect} from 'vitest';

import {formatDuration} from './time';

describe('time', () => {
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
