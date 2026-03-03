import { execSync } from 'node:child_process';

const ROOT = new URL('..', import.meta.url).pathname;

function run(cmd: string) {
  execSync(cmd, { cwd: ROOT, stdio: 'pipe', timeout: 120_000 });
}

function waitForHealth(url: string, retries = 30, intervalMs = 2000) {
  for (let i = 0; i < retries; i++) {
    try {
      execSync(`curl -sf ${url}`, { stdio: 'pipe', timeout: 5000 });
      return;
    } catch {
      if (i === retries - 1) {
        throw new Error(`Health check failed after ${retries} attempts: ${url}`);
      }
      execSync(`sleep ${intervalMs / 1000}`);
    }
  }
}

export default async function globalSetup() {
  console.log('[acceptance] Starting postgres...');
  run('docker compose up postgres -d');

  console.log('[acceptance] Waiting for postgres...');
  for (let i = 0; i < 30; i++) {
    try {
      run('docker compose exec postgres pg_isready -U otherworlds');
      break;
    } catch {
      if (i === 29) throw new Error('Postgres not ready after 60s');
      execSync('sleep 2');
    }
  }

  console.log('[acceptance] Running database migrations...');
  run(
    'docker compose exec -T postgres psql -U otherworlds -d otherworlds -f /dev/stdin < backend/migrations/20250227000001_create_domain_events.sql',
  );
  run(
    'docker compose exec -T postgres psql -U otherworlds -d otherworlds -f /dev/stdin < backend/migrations/20260301000001_add_event_type_index.sql',
  );

  console.log('[acceptance] Building and starting API...');
  run('docker compose up api -d --build');

  console.log('[acceptance] Waiting for API health check...');
  waitForHealth('http://localhost:3000/health');

  console.log('[acceptance] Infrastructure ready.');
}
