import {describe, it, expect} from 'vitest';

import {getCookiesMap} from './cookies';

describe('cookies', () => {
    it('can deal with empty cookies', () => {
        expect(getCookiesMap('')).to.be.empty;
    });

    it('can parse a single cookie', () => {
        expect(getCookiesMap('a=b')).to.deep.equal({a: 'b'});
    });

    it('can parse multiple cookies', () => {
        expect(getCookiesMap('hello=world;something=asdf;true=false')).to.deep.equal({
            hello: 'world',
            something: 'asdf',
            true: 'false',
        });
    });
});
