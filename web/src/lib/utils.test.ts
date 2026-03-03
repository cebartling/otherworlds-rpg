import { describe, it, expect } from 'vitest';
import { formatUuidShort, formatUuidDisplay } from './utils';

describe('formatUuidShort', () => {
  it('returns the first segment of a UUID', () => {
    const uuid = '550e8400-e29b-41d4-a716-446655440000';
    expect(formatUuidShort(uuid)).toBe('550e8400');
  });

  it('handles a different UUID', () => {
    const uuid = 'a1b2c3d4-e5f6-7890-abcd-ef1234567890';
    expect(formatUuidShort(uuid)).toBe('a1b2c3d4');
  });
});

describe('formatUuidDisplay', () => {
  it('returns the first segment with ellipsis', () => {
    const uuid = '550e8400-e29b-41d4-a716-446655440000';
    expect(formatUuidDisplay(uuid)).toBe('550e8400...');
  });
});
