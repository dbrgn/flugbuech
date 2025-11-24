/**
 * Time utility functions for flight data processing
 */

/**
 * Convert HH:MM time string to total minutes since midnight
 * @param time Time in "HH:MM" format
 * @returns Total minutes since midnight
 */
export function timeToMinutes(time: string): number {
    const [hours, minutes] = time.split(':').map((v) => parseInt(v, 10));
    return hours * 60 + minutes;
}

/**
 * Convert minutes since midnight to HH:MM time string
 * @param minutes Total minutes since midnight
 * @returns Time in "HH:MM" format
 */
export function minutesToTime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}`;
}

/**
 * Convert HMS array (from IGC data) to HH:MM time string
 * @param hms Tuple of [hours, minutes, seconds]
 * @returns Time in "HH:MM" format (seconds are rounded into minutes)
 */
export function hmsToTime(hms: [number, number, number]): string {
    let hours = hms[0];
    let minutes = Math.round(hms[1] + hms[2] / 60);
    if (minutes === 60) {
        minutes = 0;
        hours += 1;
    }
    return minutesToTime(hours * 60 + minutes);
}

/**
 * Calculate duration between two times (handles day rollover)
 * @param startTime Launch time in "HH:MM" format
 * @param endTime Landing time in "HH:MM" format
 * @returns Duration string in "+H:MM" format, or undefined if either time is empty
 */
export function calculateFlightDuration(startTime: string, endTime: string): string | undefined {
    if (startTime === '' || endTime === '') {
        return undefined;
    }

    const start = timeToMinutes(startTime);
    const end = timeToMinutes(endTime);
    let duration = end - start;

    // Handle day rollover (e.g., 23:00 to 01:00)
    if (duration < 0) {
        duration += 1440; // 24 hours in minutes
    }

    const hours = Math.floor(duration / 60);
    const minutes = (duration % 60).toString().padStart(2, '0');
    return `+${hours}:${minutes}`;
}

/**
 * Convert YMD array to ISO date string (YYYY-MM-DD)
 * @param ymd Tuple of [year, month, day]
 * @returns ISO date string
 */
export function ymdToDateString(ymd: [number, number, number]): string {
    const [year, month, day] = ymd;
    const m = month.toString().padStart(2, '0');
    const d = day.toString().padStart(2, '0');
    return `${year}-${m}-${d}`;
}

/**
 * Check if one time is before another
 * @param time1 First time in "HH:MM" format
 * @param time2 Second time in "HH:MM" format
 * @returns True if time1 is before time2
 */
export function isTimeBefore(time1: string, time2: string): boolean {
    return timeToMinutes(time1) < timeToMinutes(time2);
}
