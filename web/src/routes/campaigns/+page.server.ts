import type { Actions, PageServerLoad } from './$types';
import { listCampaigns, ingestCampaign } from '$lib/server/api/content';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async () => {
  try {
    const campaigns = await listCampaigns();
    return { campaigns };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  ingest: async ({ request }) => {
    const formData = await request.formData();
    const source = formData.get('source');

    if (!source || typeof source !== 'string' || source.trim().length === 0) {
      return fail(400, { error: 'Campaign source content is required.' });
    }

    let aggregateId: string;
    try {
      const result = await ingestCampaign({ source: source.trim() });
      aggregateId = result.aggregate_id;
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, `/campaigns/${aggregateId}`);
  },
};
