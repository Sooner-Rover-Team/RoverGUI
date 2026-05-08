/*
import { test } from 'vitest';
import { render } from '@testing-library/react';
import App from './App';

test('renders without crashing', () => {
  render(<App />);
});
*/

import { test, expect, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import App from './App';

test('renders camera select label', async () => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() =>
      Promise.resolve({
        json: () => Promise.resolve(['Camera 1', 'Camera 2']),
      } as Response)
    )
  );

  render(<App />);

  expect(screen.getByLabelText(/select camera/i)).toBeInTheDocument();

  await waitFor(() => {
    expect(screen.getByRole('option', { name: 'Camera 1' })).toBeInTheDocument();
  });
});