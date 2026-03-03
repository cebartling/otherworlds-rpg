import { redirect } from '@sveltejs/kit';
import {
  archiveCampaignRun,
  branchTimeline,
  createCheckpoint,
  getCampaignRun,
} from '$lib/server/api/session';
import { handleLoadError } from '$lib/server/api/errors';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const run = await getCampaignRun(params.run_id);
    return { run };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  checkpoint: async ({ params }) => {
    try {
      await createCheckpoint({ run_id: params.run_id });
      return { success: true, action: 'checkpoint' };
    } catch (err) {
      handleLoadError(err);
    }
  },

  branch: async ({ request, params }) => {
    const formData = await request.formData();
    const fromCheckpointId = formData.get('from_checkpoint_id');

    if (!fromCheckpointId || typeof fromCheckpointId !== 'string' || fromCheckpointId.trim() === '') {
      return { success: false, error: 'Checkpoint ID is required for branching.' };
    }

    try {
      const result = await branchTimeline({
        source_run_id: params.run_id,
        from_checkpoint_id: fromCheckpointId.trim(),
      });
      redirect(303, `/sessions/${result.aggregate_id}`);
    } catch (err) {
      handleLoadError(err);
    }
  },

  archive: async ({ params }) => {
    try {
      await archiveCampaignRun(params.run_id);
      redirect(303, '/sessions');
    } catch (err) {
      handleLoadError(err);
    }
  },
};
