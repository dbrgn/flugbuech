/**
 * Format a duration (in seconds) as "hh:mm".
 *
 * Example: formatDuration(3780) === "01:03"
 */
export function formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600)
        .toString()
        .padStart(2, '0');
    const minutes = (Math.floor(seconds / 60) % 60).toString().padStart(2, '0');
    return `${hours}:${minutes}`;
}

/**
 * Format a date as "YYYY-mm-dd".
 */
export function formatDate(date: Date): string {
    return date.toISOString().slice(0, 10);
}

/**
 * Format a time as "hh:mm".
 */
export function formatTime(time: Date): string {
    return time.toISOString().slice(11, 16);
}

/**
 * Format a distance in km.
 *
 * Example: formatDistance(13.37123) === "13.37 km"
 */
export function formatDistance(km: number): string {
    return `${km.toFixed(2).replace(/\.?0*$/g, '')} km`;
}
