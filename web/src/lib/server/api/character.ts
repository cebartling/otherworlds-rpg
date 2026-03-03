/**
 * Server-side API client for the Character Management bounded context.
 *
 * Routes are nested under /api/v1/characters on the backend.
 */

import type {
  AwardExperienceRequest,
  CharacterSummary,
  CharacterView,
  CommandResponse,
  CreateCharacterRequest,
  ModifyAttributeRequest,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/characters';

export async function listCharacters(): Promise<CharacterSummary[]> {
  return apiGet<CharacterSummary[]>(BASE);
}

export async function getCharacter(characterId: string): Promise<CharacterView> {
  return apiGet<CharacterView>(`${BASE}/${characterId}`);
}

export async function createCharacter(request: CreateCharacterRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/create`, request);
}

export async function modifyAttribute(request: ModifyAttributeRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/modify-attribute`, request);
}

export async function awardExperience(request: AwardExperienceRequest): Promise<CommandResponse> {
  return apiPost<CommandResponse>(`${BASE}/award-experience`, request);
}

export async function archiveCharacter(characterId: string): Promise<CommandResponse> {
  return apiDelete<CommandResponse>(`${BASE}/${characterId}`);
}
