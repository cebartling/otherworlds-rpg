import { execSync } from 'node:child_process';

const ROOT = new URL('..', import.meta.url).pathname;

export default async function globalTeardown() {
  console.log('[acceptance] Stopping docker compose services...');
  try {
    execSync('docker compose down', { cwd: ROOT, stdio: 'pipe', timeout: 30_000 });
  } catch (err) {
    console.error('[acceptance] Warning: docker compose down failed:', err);
  }
  console.log('[acceptance] Teardown complete.');
}
