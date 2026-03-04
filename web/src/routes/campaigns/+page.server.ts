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
    const file = formData.get('campaign-file');

    if (!file || !(file instanceof File) || file.size === 0) {
      return fail(400, { error: 'A .md file is required.' });
    }

    if (!file.name.endsWith('.md')) {
      return fail(400, { error: 'Only .md (Markdown) files are accepted.' });
    }

    const source = await file.text();

    if (source.trim().length === 0) {
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
