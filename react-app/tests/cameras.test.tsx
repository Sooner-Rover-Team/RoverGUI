/*
import { describe, expect, test } from 'vitest';
import { cameras } from '../src/main';

// test if cameras array is populated
describe('camera module', () => {
    test('check that camera array is non-empty', () => {
        expect(cameras).not.toHaveLength(0);
    });
});
*/

import { describe, expect, test, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor } from '@testing-library/react';
import App from '../src/App';

// Tests that the App fetches the camera list from /stream/cameras on mount.
describe('camera module', () => {
    beforeEach(() => {
        // Replace global fetch with a mock that returns a fake list of cameras.
        vi.stubGlobal('fetch', vi.fn(async () =>
            new Response(JSON.stringify(['/dev/video0', '/dev/video1']), {
                status: 200,
                headers: { 'Content-Type': 'application/json' },
            })
        ));
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    test('fetches the camera list from /stream/cameras', async () => {
        render(<App />);
        await waitFor(() => {
            expect(fetch).toHaveBeenCalledWith('/stream/cameras');
        });
    });
});