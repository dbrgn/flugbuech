export function randomBytes(buf: Uint8Array): Uint8Array {
    crypto.getRandomValues(buf);
    return buf;
}
