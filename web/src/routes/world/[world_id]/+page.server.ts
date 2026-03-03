import type { Actions, PageServerLoad } from './$types';
import {
  getWorldSnapshot,
  applyEffect,
  setFlag,
  updateDisposition,
  archiveWorldSnapshot,
} from '$lib/server/api/world-state';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const snapshot = await getWorldSnapshot(params.world_id);
    return { snapshot };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  applyEffect: async ({ request, params }) => {
    const formData = await request.formData();
    const factKey = formData.get('fact_key');

    if (!factKey || typeof factKey !== 'string' || factKey.trim().length === 0) {
      return fail(400, { action: 'applyEffect', error: 'Fact key is required.' });
    }

    try {
      await applyEffect({
        world_id: params.world_id,
        fact_key: factKey.trim(),
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'applyEffect', success: true };
  },

  setFlag: async ({ request, params }) => {
    const formData = await request.formData();
    const flagKey = formData.get('flag_key');
    const valueRaw = formData.get('value');

    if (!flagKey || typeof flagKey !== 'string' || flagKey.trim().length === 0) {
      return fail(400, { action: 'setFlag', error: 'Flag key is required.' });
    }

    if (valueRaw === null || typeof valueRaw !== 'string') {
      return fail(400, { action: 'setFlag', error: 'Flag value is required.' });
    }

    const value = valueRaw === 'true';

    try {
      await setFlag({
        world_id: params.world_id,
        flag_key: flagKey.trim(),
        value,
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'setFlag', success: true };
  },

  updateDisposition: async ({ request, params }) => {
    const formData = await request.formData();
    const entityId = formData.get('entity_id');

    if (!entityId || typeof entityId !== 'string' || entityId.trim().length === 0) {
      return fail(400, { action: 'updateDisposition', error: 'Entity ID is required.' });
    }

    try {
      await updateDisposition({
        world_id: params.world_id,
        entity_id: entityId.trim(),
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'updateDisposition', success: true };
  },

  archive: async ({ params }) => {
    try {
      await archiveWorldSnapshot(params.world_id);
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/world');
  },
};
