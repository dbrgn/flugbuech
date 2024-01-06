/**
 * When redirecting to a specific {@link redirectPath}, ensure that it's a valid path starting with "/", and not a
 * full URL. Otherwise, or if the redirect path is not defined, return the {@link fallback}.
 */
export function sanitizeRedirectPath(
    redirectPath: string | undefined | null,
    fallback: string,
): string {
    return redirectPath?.startsWith('/') ? redirectPath : fallback;
}
