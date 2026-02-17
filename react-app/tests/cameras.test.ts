import { describe, expect, test } from '@jest/globals';
import { cameras } from '../src/index';

// test if cameras array is populated
describe('camera module', () => {
    test('check that camera array is non-empty', () => {
        expect(cameras).not.toHaveLength(0);
    });
});