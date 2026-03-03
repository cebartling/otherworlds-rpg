/**
 * Server-side API client for the Rules & Resolution bounded context.
 *
 * Routes are nested under /api/v1/rules on the backend.
 */

import type {
  CommandResponse,
  DeclareIntentRequest,
  ProduceEffectsRequest,
  ResolutionSummary,
  ResolutionView,
  ResolveCheckRequest,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/rules';

export async function listResolutions(): Promise<ResolutionSummary[]> {
  return apiGet<ResolutionSummary[]>(BASE);
}

export async function getResolution(resolutionId: string): Promise<ResolutionView> {
  return apiGet<ResolutionView>(`${BASE}/${resolutionId}`);
}

export async function declareIntent(request: DeclareIntentRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/declare-intent`, request);
}

export async function resolveCheck(request: ResolveCheckRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/resolve-check`, request);
}

export async function produceEffects(request: ProduceEffectsRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/produce-effects`, request);
}

export async function archiveResolution(resolutionId: string): Promise<CommandResponse> {
  return apiDelete<CommandResponse>(`${BASE}/${resolutionId}`);
}
