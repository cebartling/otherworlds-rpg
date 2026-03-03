import { redirect } from '@sveltejs/kit';
import { listCampaignRuns, startCampaignRun } from '$lib/server/api/session';
import { handleLoadError } from '$lib/server/api/errors';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
  try {
    const runs = await listCampaignRuns();
    return { runs };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  start: async ({ request }) => {
    const formData = await request.formData();
    const campaignId = formData.get('campaign_id');

    if (!campaignId || typeof campaignId !== 'string' || campaignId.trim() === '') {
      return { success: false, error: 'Campaign ID is required.' };
    }

    try {
      const result = await startCampaignRun({ campaign_id: campaignId.trim() });
      redirect(303, `/sessions/${result.aggregate_id}`);
    } catch (err) {
      handleLoadError(err);
    }
  },
};
