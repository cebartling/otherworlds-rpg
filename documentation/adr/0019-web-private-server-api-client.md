# ADR-0019: Web Private Server-Side API Client

## Status

Accepted

## Context

The SvelteKit web client communicates with the Rust backend over HTTP. The backend URL and API structure are implementation details that should not be exposed to the browser. SvelteKit's `$lib/server/` module convention enforces server-only imports at build time — any attempt to import a `$lib/server/` module from client-side code produces a build error.

The client needs a typed, consistent way to call backend endpoints with JSON encoding, error parsing, and optional correlation ID forwarding, while keeping the backend URL confined to server-side code.

## Decision

All backend communication is encapsulated in the `$lib/server/api/` module, which is server-only by convention. The module has two layers:

### Core Client (`client.ts`)

A generic `apiFetch<T>()` wrapper handles all HTTP concerns:

```typescript
async function apiFetch<T>(path: string, options: ApiFetchOptions = {}): Promise<T> {
  const url = `${API_BASE_URL}${path}`;
  const headers = { 'Content-Type': 'application/json', 'Accept': 'application/json' };
  if (correlationId) headers['X-Correlation-ID'] = correlationId;
  // ... fetch, error parsing, dev logging
}
```

Key behaviors:
- Prepends `API_BASE_URL` (from environment) to all paths.
- Sets JSON content headers on every request.
- Forwards optional `X-Correlation-ID` headers for tracing.
- Logs requests and responses in dev mode via `import.meta.env.DEV`.
- Parses non-OK responses into a custom `ApiClientError` class.

Three convenience functions are exported: `apiGet<T>`, `apiPost<T>`, `apiDelete<T>`.

### Error Type (`ApiClientError`)

```typescript
export class ApiClientError extends Error {
  public readonly status: number;
  public readonly errorResponse: ErrorResponse;

  constructor(status: number, errorResponse: ErrorResponse) {
    super(`API error ${status}: ${errorResponse.message}`);
    this.name = 'ApiClientError';
    this.status = status;
    this.errorResponse = errorResponse;
  }
}
```

This preserves both the HTTP status code and the structured error body from the backend's `ApiError` responses, enabling precise error mapping in `handleLoadError`.

### Context Endpoint Modules

Each bounded context has a dedicated endpoint module (e.g., `character.ts`, `narrative.ts`, `content.ts`) that exports typed async functions:

```typescript
const BASE = '/api/v1/characters';

export async function listCharacters(): Promise<CharacterSummary[]> {
  return apiGet<CharacterSummary[]>(BASE);
}

export async function createCharacter(request: CreateCharacterRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/create`, request);
}
```

Each module:
- Defines a `BASE` path matching the backend's route prefix (`/api/v1/{context}`).
- Exports one function per backend endpoint.
- Uses TypeScript generics for request/response type safety.
- Delegates all HTTP concerns to the core client.

## Consequences

### Positive

- The backend URL (`API_BASE_URL`) never appears in browser-delivered JavaScript — SvelteKit's `$lib/server/` import restriction enforces this at build time.
- Typed endpoint functions catch request/response shape mismatches at compile time.
- `ApiClientError` carries structured error data, enabling precise mapping to SvelteKit error responses.
- Dev-mode logging provides request/response visibility without production overhead.
- Adding a new context endpoint module requires only a new file following the established pattern.

### Negative

- Every backend endpoint requires a corresponding TypeScript function, which is manual boilerplate (no code generation).
- The `$lib/server/` restriction means client-side components cannot call the API directly — all data must flow through `+page.server.ts` load functions and actions (this is intentional but limits flexibility).
- Type definitions for request/response bodies must be maintained in sync with the backend's Rust types manually.

### Constraints

- All backend communication must go through `$lib/server/api/` modules. No direct `fetch` calls to the backend from `+page.server.ts` or anywhere else.
- Context endpoint modules must follow the `/api/v1/{context}` path convention.
- `ApiClientError` must be used for all error cases — do not throw raw `Error` objects from API calls.
- The `API_BASE_URL` environment variable must be set in all deployment environments.
