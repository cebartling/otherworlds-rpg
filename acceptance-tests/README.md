# Acceptance Tests

End-to-end browser tests for the Otherworlds RPG web client, driven by [Playwright](https://playwright.dev/) and run with [Bun](https://bun.sh/).

## Prerequisites

- **Bun** — runtime and package manager ([install guide](https://bun.sh/docs/installation))
- **Docker & Docker Compose** — PostgreSQL and the Rust API server run in containers
- **Node.js** (v18+) — SvelteKit dev server
- **Rust toolchain** — the API container builds from source via `docker compose up api --build`

## Environment Setup

### 1. Install Bun

```bash
curl -fsSL https://bun.sh/install | bash
```

Verify the installation:

```bash
bun --version
```

### 2. Install dependencies

```bash
cd acceptance-tests
bun install
```

This installs Playwright and its type definitions per `package.json`.

### 3. Install Playwright browsers

```bash
bunx playwright install chromium
```

Playwright needs a local Chromium binary. This only needs to be run once (or after Playwright version upgrades).

### 4. Install web client dependencies

The SvelteKit dev server must be able to start. If you haven't already:

```bash
cd ../web
npm install
```

### 5. Verify Docker is running

The global setup starts PostgreSQL and the API server via Docker Compose. Make sure the Docker daemon is running:

```bash
docker info
```

## Running Tests

```bash
cd acceptance-tests
bun run test               # headless Chromium
bun run test:ui            # interactive Playwright UI
```

The Playwright config handles infrastructure automatically:

1. **Global setup** starts PostgreSQL via `docker compose`, runs migrations, truncates stale data, builds and starts the Rust API server, and waits for its `/health` endpoint.
2. **Web server** starts the SvelteKit dev server on `http://localhost:5173` (reuses an existing one outside CI).
3. **Global teardown** stops all Docker Compose services.

## Configuration

| Setting | Value |
|---|---|
| Browser | Chromium (Desktop Chrome) |
| Base URL | `http://localhost:5173` |
| Parallelism | Sequential (`workers: 1`) |
| Retries | 2 in CI, 0 locally |
| Trace | Captured on first retry |
| Reporter | HTML (`playwright-report/`) |

See `playwright.config.ts` for full details.

## Directory Structure

```
acceptance-tests/
├── campaigns/
│   └── campaign-pipeline.test.ts   # Campaign pipeline test suite
├── fixtures/
│   └── the-lost-temple.md          # Valid campaign markdown fixture
├── global-setup.ts                 # Docker + DB migrations + API startup
├── global-teardown.ts              # Docker Compose teardown
├── playwright.config.ts
├── package.json
└── README.md
```

## Test Suites

### Campaign Pipeline (`campaigns/campaign-pipeline.test.ts`)

Tests the full content pipeline lifecycle: ingest, validate, compile, archive. Every test ingests a fresh campaign so tests are independent and can run in any order.

**Shared helpers:**

- `ingestCampaign(page, source)` — Opens the ingest form, uploads a `.md` file via the file input, submits, waits for redirect to the campaign detail page, and returns the new campaign UUID.
- `pipelineStepCircle(page, stepNumber)` — Locates the numbered circle element in the Content Pipeline section (1 = Ingested, 2 = Validated, 3 = Compiled).

**Fixtures:**

| Constant | Source | Purpose |
|---|---|---|
| `VALID_CAMPAIGN_SOURCE` | `fixtures/the-lost-temple.md` | Well-formed campaign with YAML front-matter, 4 scenes, 2 NPCs |
| `INVALID_CAMPAIGN_SOURCE` | Inline string | Plain text with no front-matter or scenes (triggers validation errors) |

**Style assertions:**

Pipeline step and badge state is verified via inline `style` attributes. Two regex patterns match green (active) and gray (inactive) states, accounting for both SSR hex values and client-side RGB equivalents.

#### Test Cases

##### 1. Ingest campaign via file upload

Uploads a valid campaign markdown file and verifies the resulting detail page:

- Redirects to `/campaigns/<uuid>`
- Heading displays the campaign ID prefix
- Pipeline step 1 (Ingested) is green; steps 2 and 3 are gray
- Version Details shows v1 with a non-empty hash
- Validate button is enabled; Compile button is disabled

##### 2. Validate ingested campaign

Ingests a campaign, then clicks Validate:

- Success message "Campaign validated successfully." appears
- Pipeline steps 1 and 2 are green; step 3 is gray
- Validate button is disabled; Compile button is enabled

##### 3. Compile validated campaign

Ingests a campaign, validates it, then clicks Compile:

- Success message "Campaign compiled successfully." appears
- All three pipeline steps are green
- Both Validate and Compile buttons are disabled

##### 4. Archive campaign removes from list

Ingests a campaign, clicks Archive, then confirms:

- Redirects to `/campaigns`
- The archived campaign no longer appears in the list

##### 5. Campaigns list shows ingested campaign with correct badge state

Ingests a campaign, then navigates back to the campaigns list:

- Campaign card is visible with a link to its detail page
- Ingested badge is green
- Validated and Compiled badges are gray

##### 6. Validation error displays inline

Ingests an invalid campaign (no front-matter), then clicks Validate:

- Stays on the campaign detail page (no SvelteKit error page)
- Inline error block is visible with a non-empty error message
- Success message is not visible
- Pipeline step 1 remains green; step 2 remains gray

## Writing New Tests

1. Create a new `.test.ts` file under a descriptive directory (e.g., `sessions/session-management.test.ts`).
2. Import from `@playwright/test` and reuse the pattern of shared helpers for repeated setup steps.
3. Each test should be self-contained — create its own data rather than depending on another test's side effects.
4. Use `waitUntil: 'networkidle'` on `page.goto` for pages that load data from the API.
5. Prefer role-based selectors (`getByRole`, `getByText`) over CSS selectors where possible.
6. Run `bun run test` to verify all tests pass before committing.

## Troubleshooting

### Playwright browsers not found

```
Error: browserType.launch: Executable doesn't exist
```

Run `bunx playwright install chromium` to download the browser binary.

### PostgreSQL fails to start

Check that Docker is running (`docker info`) and that port 5432 is not already in use by a local PostgreSQL instance.

### API health check times out

The global setup waits up to 60 seconds for the API at `http://localhost:3000/health`. If the Rust build is slow on first run, try building the API image ahead of time:

```bash
cd ..
docker compose build api
```

### SvelteKit dev server won't start

Make sure web client dependencies are installed (`cd ../web && npm install`). Playwright starts the dev server automatically but won't install npm packages for you.

### Tests pass locally but fail in CI

CI uses `retries: 2` and does not reuse existing servers. Check that no test relies on data from a previous test — each test should create its own campaign via the `ingestCampaign` helper.
