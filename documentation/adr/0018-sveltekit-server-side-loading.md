# ADR-0018: SvelteKit Server-Side Data Loading and Form Actions

## Status

Accepted

## Context

The SvelteKit web client needs a strategy for fetching data from the backend and handling mutations. SvelteKit supports multiple patterns: client-side `fetch` in `onMount`, server-side `load` functions in `+page.server.ts`, and form `actions` for mutations. Client-side loading requires loading spinners, error boundaries, and duplicates fetch logic across components. Server-side loading renders pages with data already present, aligning with SvelteKit's strengths.

The backend is the single source of truth (event-sourced, append-only). The web client should not maintain its own cache or optimistic state — it should always reflect what the server reports.

## Decision

All data fetching and mutations flow through `+page.server.ts` files using SvelteKit's server-side `load` functions and form `actions`.

### Data Loading

Every route's `+page.server.ts` exports a `load` function that fetches data before the page renders:

```typescript
export const load: PageServerLoad = async () => {
  try {
    const characters = await listCharacters();
    return { characters };
  } catch (err) {
    handleLoadError(err);
  }
};
```

- `load` runs on the server before any HTML is sent to the client.
- No client-side loading spinners are needed for initial page data.
- Errors are mapped via `handleLoadError`, which translates `ApiClientError` into SvelteKit's `error()` responses with appropriate HTTP status codes and messages.

### Mutations via Form Actions

State-changing operations use named form `actions` with `use:enhance` for progressive enhancement:

```typescript
export const actions: Actions = {
  create: async ({ request }) => {
    const formData = await request.formData();
    const name = formData.get('name');

    if (!name || typeof name !== 'string' || name.trim().length === 0) {
      return fail(400, { error: 'Character name is required.' });
    }

    try {
      await createCharacter({ name: name.trim() });
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/characters');
  },
};
```

- Mutations POST form data to named actions.
- Successful mutations redirect with HTTP 303 (POST-Redirect-GET), which triggers a fresh `load` call and ensures the page displays current server state.
- Validation errors return `fail()` with error data for the form to display.
- API errors in actions use the same `handleLoadError` mapping as `load` functions.

### No Client-Side Cache

The web client does not implement optimistic updates, client-side caching, or local state management for server data. Every page load fetches fresh data from the backend. This is a deliberate choice: the backend's event-sourced model means the server is always the authoritative source, and caching would introduce stale-data bugs without meaningful performance benefit for the current scale.

## Consequences

### Positive

- Pages render with data already present — no flash of loading state on initial navigation.
- POST-Redirect-GET pattern ensures the displayed data always reflects the server's current state.
- Progressive enhancement via `use:enhance` means forms work without JavaScript, then enhance with JS when available.
- Consistent error handling: `handleLoadError` maps API errors to SvelteKit errors in one place.
- No client-side state management library needed (no Redux, no stores for server data).

### Negative

- Every navigation fetches fresh data from the backend, which adds latency compared to a client-side cache.
- No optimistic updates means the UI waits for the server round-trip before reflecting mutations.
- Form actions require `formData` parsing, which is more verbose than JSON request bodies for complex forms.

### Constraints

- All data fetching must happen in `+page.server.ts` `load` functions, not in client-side `onMount` or `+page.ts`.
- All mutations must use form `actions`, not client-side `fetch` to API endpoints.
- Successful mutations must redirect with 303 to ensure fresh data on the resulting page.
- `handleLoadError` must be used for all API error handling in both `load` and `actions`.
