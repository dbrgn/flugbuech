/**
 * Return a record of local cookies.
 *
 * Note: This function cannot currently deal with cookie values containing a `=`. Improve this?
 */
export function getCookiesMap(): Record<string, string> {
    return document.cookie
        .split(';')
        .map((cookieString) => cookieString.trim().split('='))
        .reduce((acc: Record<string, string>, curr: string[]) => {
            acc[curr[0]] = curr[1];
            return acc;
        }, {});
}
