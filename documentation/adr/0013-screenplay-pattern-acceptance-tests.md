# ADR-0013: Screenplay Pattern for Acceptance Tests

## Status

Accepted

## Context

The Playwright acceptance tests use flat helper functions (`ingestCampaign`, `pipelineStepCircle`) that mix navigation, form interaction, and state queries in a single abstraction layer. As the test suite grows beyond the campaign pipeline, this approach will lead to duplicated Playwright mechanics across test files and helpers that are hard to compose or reuse independently.

We need a test architecture that:
- Separates business intent from browser mechanics
- Makes tests read like user stories
- Allows new test suites to reuse interactions and queries without copy-pasting
- Keeps Playwright coupling in a single layer so driver changes are isolated

## Decision

Adopt the **Screenplay pattern** with a lightweight custom implementation (no external library). The pattern introduces five concepts:

- **Actor** — represents a user persona, holds Abilities, orchestrates activities via `attemptsTo()` and `asks()`
- **Ability** — wraps infrastructure; `BrowseTheWeb` wraps the Playwright `Page`
- **Interaction** — single atomic browser action (Navigate, Click, UploadFile, WaitForUrl)
- **Task** — high-level business action composed of Interactions (IngestCampaign, ValidateCampaign, etc.)
- **Question** — state query returning a typed value (ThePipelineStep, TheButtonState, etc.)

The core framework (interfaces, Actor class, BrowseTheWeb ability) fits in ~50 lines. No npm dependencies are added. Tasks delegate to `actor.attemptsTo(...)` with Interactions; only Interactions and Questions access the Playwright Page directly.

Structure lives under `acceptance-tests/screenplay/` with subdirectories: `core/`, `interactions/`, `tasks/`, `questions/`, and a barrel `index.ts`.

## Consequences

- **Easier**: Tests read as business language (`actor.attemptsTo(IngestCampaign.withSource(md))`). New bounded context test suites reuse existing Interactions and Questions. Playwright mechanics are isolated — changing selectors or wait strategies touches one file, not every test.
- **Harder**: More files than flat helpers (16 new files for the initial setup). Contributors must learn the Screenplay vocabulary. Simple one-off assertions may feel over-abstracted if wrapped prematurely — the convention is to leave truly one-off Playwright assertions inline until a pattern recurs.
- **No new dependencies**: The implementation is pure TypeScript with no external Screenplay library.
