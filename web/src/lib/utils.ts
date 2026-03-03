/**
 * Formats a UUID for display by showing only the first 8 characters.
 * Example: "550e8400-e29b-41d4-a716-446655440000" -> "550e8400"
 */
export function formatUuidShort(uuid: string): string {
  return uuid.split('-')[0];
}

/**
 * Formats a UUID for display with ellipsis.
 * Example: "550e8400-e29b-41d4-a716-446655440000" -> "550e8400..."
 */
export function formatUuidDisplay(uuid: string): string {
  return `${formatUuidShort(uuid)}...`;
}
