/**
 * Format a duration (in seconds) as "hh:mm".
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
