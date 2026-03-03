import { API_BASE_URL } from '$env/static/private';
import type { Handle } from '@sveltejs/kit';

// Validate required environment variables on startup.
// This runs once when the server-side module is first loaded.
if (!API_BASE_URL) {
  throw new Error(
    'Missing required environment variable: API_BASE_URL. ' +
    'Set it in web/.env (e.g. API_BASE_URL=http://localhost:3000).'
  );
}

export const handle: Handle = async ({ event, resolve }) => {
  return resolve(event);
};
