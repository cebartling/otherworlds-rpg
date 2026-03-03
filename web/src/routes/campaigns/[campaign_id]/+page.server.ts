import type { Actions, PageServerLoad } from './$types';
import {
  getCampaign,
  validateCampaign,
  compileCampaign,
  archiveCampaign,
} from '$lib/server/api/content';
import { ApiClientError } from '$lib/server/api/client';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

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
      if (err instanceof ApiClientError && err.status >= 400 && err.status < 500) {
        return fail(err.status, { action: 'validate', error: err.errorResponse.message });
      }
      handleLoadError(err);
    }

    return { action: 'validate', success: true };
  },

  compile: async ({ params }) => {
    try {
      await compileCampaign({ campaign_id: params.campaign_id });
    } catch (err) {
      if (err instanceof ApiClientError && err.status >= 400 && err.status < 500) {
        return fail(err.status, { action: 'compile', error: err.errorResponse.message });
      }
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
