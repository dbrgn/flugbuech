import {describe, it, expect} from 'vitest';

import {
    timeToMinutes,
    minutesToTime,
    hmsToTime,
    calculateFlightDuration,
    ymdToDateString,
    isTimeBefore,
} from './time-helpers';

describe('timeToMinutes', () => {
    const testCases = [
        ['00:00', 0],
        ['00:01', 1],
        ['00:59', 59],
        ['01:00', 60],
        ['01:30', 90],
        ['12:00', 720],
        ['23:59', 1439],
    ] as const;

    for (const [time, expected] of testCases) {
        it(`converts ${time} to ${expected} minutes`, () => {
            expect(timeToMinutes(time)).to.equal(expected);
        });
    }
});

describe('minutesToTime', () => {
    const testCases = [
        [0, '00:00'],
        [1, '00:01'],
        [59, '00:59'],
        [60, '01:00'],
        [90, '01:30'],
        [720, '12:00'],
        [1439, '23:59'],
    ] as const;

    for (const [minutes, expected] of testCases) {
        it(`converts ${minutes} minutes to ${expected}`, () => {
            expect(minutesToTime(minutes)).to.equal(expected);
        });
    }
});

describe('hmsToTime', () => {
    it('converts HMS with zero seconds', () => {
        expect(hmsToTime([12, 30, 0])).to.equal('12:30');
    });

    it('converts HMS with seconds < 30 (rounds down)', () => {
        expect(hmsToTime([12, 30, 25])).to.equal('12:30');
    });

    it('converts HMS with seconds >= 30 (rounds up)', () => {
        expect(hmsToTime([12, 30, 35])).to.equal('12:31');
    });

    it('handles minute overflow when rounding seconds', () => {
        expect(hmsToTime([12, 59, 30])).to.equal('13:00');
    });

    it('handles hour overflow when rounding seconds at 23:59', () => {
        expect(hmsToTime([23, 59, 30])).to.equal('24:00');
    });

    it('converts midnight', () => {
        expect(hmsToTime([0, 0, 0])).to.equal('00:00');
    });

    it('converts typical launch time', () => {
        expect(hmsToTime([14, 23, 45])).to.equal('14:24');
    });
});

describe('calculateFlightDuration', () => {
    it('calculates duration for same day flight', () => {
        expect(calculateFlightDuration('12:00', '14:30')).to.equal('+2:30');
    });

    it('calculates duration for short flight', () => {
        expect(calculateFlightDuration('10:15', '10:45')).to.equal('+0:30');
    });

    it('calculates duration for long flight', () => {
        expect(calculateFlightDuration('09:00', '16:30')).to.equal('+7:30');
    });

    it('handles day rollover (midnight crossing)', () => {
        expect(calculateFlightDuration('23:00', '01:00')).to.equal('+2:00');
    });

    it('handles day rollover with minutes', () => {
        expect(calculateFlightDuration('23:45', '00:15')).to.equal('+0:30');
    });

    it('returns undefined when start time is empty', () => {
        expect(calculateFlightDuration('', '14:30')).to.be.undefined;
    });

    it('returns undefined when end time is empty', () => {
        expect(calculateFlightDuration('12:00', '')).to.be.undefined;
    });

    it('returns undefined when both times are empty', () => {
        expect(calculateFlightDuration('', '')).to.be.undefined;
    });

    it('calculates zero duration for same time', () => {
        expect(calculateFlightDuration('12:00', '12:00')).to.equal('+0:00');
    });

    it('calculates one minute duration', () => {
        expect(calculateFlightDuration('12:00', '12:01')).to.equal('+0:01');
    });

    it('formats hours with single digit', () => {
        expect(calculateFlightDuration('10:00', '15:00')).to.equal('+5:00');
    });

    it('formats hours with double digits', () => {
        expect(calculateFlightDuration('08:00', '20:30')).to.equal('+12:30');
    });
});

describe('ymdToDateString', () => {
    const testCases: Array<[[number, number, number], string]> = [
        [[2024, 1, 1], '2024-01-01'],
        [[2024, 12, 31], '2024-12-31'],
        [[2024, 3, 15], '2024-03-15'],
        [[2025, 10, 14], '2025-10-14'],
        [[2000, 2, 29], '2000-02-29'], // Leap year
    ];

    for (const [ymd, expected] of testCases) {
        it(`converts [${ymd.join(', ')}] to ${expected}`, () => {
            expect(ymdToDateString(ymd)).to.equal(expected);
        });
    }
});

describe('isTimeBefore', () => {
    it('returns true when first time is before second', () => {
        expect(isTimeBefore('10:00', '11:00')).to.be.true;
    });

    it('returns true when first time is one minute before', () => {
        expect(isTimeBefore('10:59', '11:00')).to.be.true;
    });

    it('returns false when first time is after second', () => {
        expect(isTimeBefore('11:00', '10:00')).to.be.false;
    });

    it('returns false when times are equal', () => {
        expect(isTimeBefore('10:00', '10:00')).to.be.false;
    });

    it('returns true for midnight vs morning', () => {
        expect(isTimeBefore('00:00', '06:00')).to.be.true;
    });

    it('returns true for morning vs afternoon', () => {
        expect(isTimeBefore('09:30', '14:45')).to.be.true;
    });

    it('returns false for evening vs morning (no day rollover handling)', () => {
        // Note: This function compares times within the same day only
        expect(isTimeBefore('23:00', '01:00')).to.be.false;
    });
});
