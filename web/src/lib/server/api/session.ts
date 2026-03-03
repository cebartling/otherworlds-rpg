/**
 * Server-side API client for the Session & Progress bounded context.
 *
 * Routes are nested under /api/v1/sessions on the backend.
 */

import type {
  BranchTimelineRequest,
  CampaignRunSummary,
  CampaignRunView,
  CommandResponseWithAggregate,
  CreateCheckpointRequest,
  StartCampaignRunRequest,
} from '$lib/types';

import { apiDelete, apiGet, apiPost } from './client';

const BASE = '/api/v1/sessions';

export async function listCampaignRuns(): Promise<CampaignRunSummary[]> {
  return apiGet<CampaignRunSummary[]>(BASE);
}

export async function getCampaignRun(runId: string): Promise<CampaignRunView> {
  return apiGet<CampaignRunView>(`${BASE}/${runId}`);
}

export async function startCampaignRun(
  request: StartCampaignRunRequest,
): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/start-campaign-run`, request);
}

export async function createCheckpoint(
  request: CreateCheckpointRequest,
): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/create-checkpoint`, request);
}

export async function branchTimeline(
  request: BranchTimelineRequest,
): Promise<CommandResponseWithAggregate> {
  return apiPost<CommandResponseWithAggregate>(`${BASE}/branch-timeline`, request);
}

export async function archiveCampaignRun(
  runId: string,
): Promise<CommandResponseWithAggregate> {
  return apiDelete<CommandResponseWithAggregate>(`${BASE}/${runId}`);
}
