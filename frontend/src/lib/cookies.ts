/**
 * Return a record of local cookies.
 *
 * Pass in `document.cookie` when using this function in the browser.
 *
 * Note: This function cannot currently deal with cookie values containing a `=`. Improve this?
 */
export function getCookiesMap(cookies: string): Record<string, string> {
    return cookies
        .split(';')
        .filter((cookieString) => cookieString.length > 0)
        .map((cookieString) => cookieString.trim().split('='))
        .reduce((acc: Record<string, string>, curr: string[]) => {
            acc[curr[0]] = curr[1];
            return acc;
        }, {});
}
