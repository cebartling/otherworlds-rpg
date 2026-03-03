/**
 * Server-side API client for the Content Authoring bounded context.
 *
 * Routes are nested under /api/v1/content on the backend.
 */

import type {
  CampaignSummary,
  CampaignView,
  CommandResponseWithAggregate,
  CompileCampaignRequest,
  IngestCampaignRequest,
  ValidateCampaignRequest,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/content';

export async function listCampaigns(): Promise<CampaignSummary[]> {
  return apiGet<CampaignSummary[]>(BASE);
}

export async function getCampaign(campaignId: string): Promise<CampaignView> {
  return apiGet<CampaignView>(`${BASE}/${campaignId}`);
}

export async function ingestCampaign(
  request: IngestCampaignRequest,
): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/ingest-campaign`, request);
}

export async function validateCampaign(
  request: ValidateCampaignRequest,
): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/validate-campaign`, request);
}

export async function compileCampaign(
  request: CompileCampaignRequest,
): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/compile-campaign`, request);
}

export async function archiveCampaign(
  campaignId: string,
): Promise<CommandResponseWithAggregate> {
  return apiDelete<CommandResponseWithAggregate>(`${BASE}/${campaignId}`);
}
