import type { Actions, PageServerLoad } from './$types';
import {
  getResolution,
  declareIntent,
  resolveCheck,
  produceEffects,
  archiveResolution,
} from '$lib/server/api/rules';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const resolution = await getResolution(params.resolution_id);
    return { resolution };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  declareIntent: async ({ request, params }) => {
    const formData = await request.formData();
    const actionType = formData.get('action_type');
    const skill = formData.get('skill');
    const targetId = formData.get('target_id');
    const difficultyClassRaw = formData.get('difficulty_class');
    const modifierRaw = formData.get('modifier');

    if (!actionType || typeof actionType !== 'string' || actionType.trim().length === 0) {
      return fail(400, { action: 'declareIntent', error: 'Action type is required.' });
    }

    const difficultyClass = parseInt(String(difficultyClassRaw), 10);
    if (isNaN(difficultyClass)) {
      return fail(400, { action: 'declareIntent', error: 'Difficulty class must be a number.' });
    }

    const modifier = parseInt(String(modifierRaw), 10);
    if (isNaN(modifier)) {
      return fail(400, { action: 'declareIntent', error: 'Modifier must be a number.' });
    }

    try {
      await declareIntent({
        resolution_id: params.resolution_id,
        intent_id: crypto.randomUUID(),
        action_type: actionType.trim(),
        skill: skill && typeof skill === 'string' && skill.trim().length > 0 ? skill.trim() : null,
        target_id: targetId && typeof targetId === 'string' && targetId.trim().length > 0 ? targetId.trim() : null,
        difficulty_class: difficultyClass,
        modifier,
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'declareIntent', success: true };
  },

  resolveCheck: async ({ params }) => {
    try {
      await resolveCheck({ resolution_id: params.resolution_id });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'resolveCheck', success: true };
  },

  produceEffects: async ({ request, params }) => {
    const formData = await request.formData();
    const effectType = formData.get('effect_type');
    const targetId = formData.get('target_id');

    if (!effectType || typeof effectType !== 'string' || effectType.trim().length === 0) {
      return fail(400, { action: 'produceEffects', error: 'Effect type is required.' });
    }

    try {
      await produceEffects({
        resolution_id: params.resolution_id,
        effects: [
          {
            effect_type: effectType.trim(),
            target_id: targetId && typeof targetId === 'string' && targetId.trim().length > 0 ? targetId.trim() : null,
            payload: {},
          },
        ],
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'produceEffects', success: true };
  },

  archive: async ({ params }) => {
    try {
      await archiveResolution(params.resolution_id);
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/rules');
  },
};
