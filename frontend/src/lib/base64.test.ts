import {describe, it, expect} from 'vitest';

import {u8aToBase64} from './base64';

describe('base64', () => {
    it('encodes an empty array', () => {
        expect(u8aToBase64(new Uint8Array([]))).to.equal('');
    });

    it('encodes the text "hi"', () => {
        expect(u8aToBase64(new Uint8Array([104, 105]))).to.equal('aGk=');
    });

    it('encodes the bytes [0xff, 0xff]', () => {
        expect(u8aToBase64(new Uint8Array([0xff, 0xff]))).to.equal('//8=');
    });

    it('encodes non-URL-safe by default', () => {
        expect(u8aToBase64(new Uint8Array([0xfe, 0xff, 0xfe]))).to.equal('/v/+');
    });

    it('supports URL-safe encoding', () => {
        expect(u8aToBase64(new Uint8Array([0xfe, 0xff, 0xfe]), {urlSafe: true})).to.equal('_v_-');
    });

    it('supports removal of padding', () => {
        expect(u8aToBase64(new Uint8Array([0xff, 0xff]), {noPad: true})).to.equal('//8');
    });
});
