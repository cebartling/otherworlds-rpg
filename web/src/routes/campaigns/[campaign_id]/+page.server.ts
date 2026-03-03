import type { Actions, PageServerLoad } from './$types';
import {
  getCampaign,
  validateCampaign,
  compileCampaign,
  archiveCampaign,
} from '$lib/server/api/content';
import { handleLoadError } from '$lib/server/api/errors';
import { redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const campaign = await getCampaign(params.campaign_id);
    return { campaign };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  validate: async ({ params }) => {
    try {
      await validateCampaign({ campaign_id: params.campaign_id });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'validate', success: true };
  },

  compile: async ({ params }) => {
    try {
      await compileCampaign({ campaign_id: params.campaign_id });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'compile', success: true };
  },

  archive: async ({ params }) => {
    try {
      await archiveCampaign(params.campaign_id);
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/campaigns');
  },
};
