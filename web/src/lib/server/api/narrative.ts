/**
 * Server-side API client for the Narrative Orchestration bounded context.
 *
 * Routes are nested under /api/v1/narrative on the backend.
 */

import type {
  AdvanceBeatRequest,
  CommandResponse,
  EnterSceneRequest,
  NarrativeSessionSummary,
  NarrativeSessionView,
  PresentChoiceRequest,
  SelectChoiceRequest,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/narrative';

export async function listNarrativeSessions(): Promise<NarrativeSessionSummary[]> {
  return apiGet<NarrativeSessionSummary[]>(BASE);
}

export async function getNarrativeSession(sessionId: string): Promise<NarrativeSessionView> {
  return apiGet<NarrativeSessionView>(`${BASE}/${sessionId}`);
}

export async function advanceBeat(request: AdvanceBeatRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/advance-beat`, request);
}

export async function presentChoice(request: PresentChoiceRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/present-choice`, request);
}

export async function enterScene(request: EnterSceneRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/enter-scene`, request);
}

export async function selectChoice(request: SelectChoiceRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/select-choice`, request);
}

export async function archiveNarrativeSession(sessionId: string): Promise<CommandResponse> {
  return apiDelete<CommandResponse>(`${BASE}/${sessionId}`);
}
