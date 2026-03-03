/**
 * Server-side API client for the World State bounded context.
 *
 * Routes are nested under /api/v1/world on the backend.
 */

import type {
  ApplyEffectRequest,
  CommandResponse,
  SetFlagRequest,
  UpdateDispositionRequest,
  WorldSnapshotSummary,
  WorldSnapshotView,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/world';

export async function listWorldSnapshots(): Promise<WorldSnapshotSummary[]> {
  return apiGet<WorldSnapshotSummary[]>(BASE);
}

export async function getWorldSnapshot(worldId: string): Promise<WorldSnapshotView> {
  return apiGet<WorldSnapshotView>(`${BASE}/${worldId}`);
}

export async function applyEffect(request: ApplyEffectRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/apply-effect`, request);
}

export async function setFlag(request: SetFlagRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/set-flag`, request);
}

export async function updateDisposition(
  request: UpdateDispositionRequest,
): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/update-disposition`, request);
}

export async function archiveWorldSnapshot(worldId: string): Promise<CommandResponse> {
  return apiDelete<CommandResponse>(`${BASE}/${worldId}`);
}
