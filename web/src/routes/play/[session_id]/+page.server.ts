import { getNarrativeSession, advanceBeat, selectChoice, enterScene } from '$lib/server/api/narrative';
import { handleLoadError, mapApiError } from '$lib/server/api';
import { fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const session = await getNarrativeSession(params.session_id);
    return { session };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  selectChoice: async ({ params, request }) => {
    const formData = await request.formData();
    const choiceIndexRaw = formData.get('choice_index');
    const sceneId = formData.get('scene_id');
    const narrativeText = formData.get('narrative_text');
    const choicesJson = formData.get('choices');
    const npcRefsJson = formData.get('npc_refs');

    if (
      choiceIndexRaw === null ||
      sceneId === null ||
      narrativeText === null ||
      choicesJson === null
    ) {
      return fail(400, { error: 'Missing required fields for choice selection.' });
    }

    const choiceIndex = parseInt(choiceIndexRaw.toString(), 10);
    if (isNaN(choiceIndex)) {
      return fail(400, { error: 'Invalid choice index.' });
    }

    let choices: Array<{ label: string; target_scene_id: string }>;
    try {
      choices = JSON.parse(choicesJson.toString());
    } catch {
      return fail(400, { error: 'Invalid choices data.' });
    }

    let npcRefs: string[] | undefined;
    if (npcRefsJson) {
      try {
        npcRefs = JSON.parse(npcRefsJson.toString());
      } catch {
        return fail(400, { error: 'Invalid NPC refs data.' });
      }
    }

    try {
      await selectChoice({
        session_id: params.session_id,
        choice_index: choiceIndex,
        target_scene: {
          scene_id: sceneId.toString(),
          narrative_text: narrativeText.toString(),
          choices,
          npc_refs: npcRefs,
        },
      });
      return { success: true };
    } catch (err) {
      mapApiError(err);
    }
  },

  advanceBeat: async ({ params }) => {
    try {
      await advanceBeat({ session_id: params.session_id });
      return { success: true };
    } catch (err) {
      mapApiError(err);
    }
  },

  enterScene: async ({ params, request }) => {
    const formData = await request.formData();
    const sceneId = formData.get('scene_id');
    const narrativeText = formData.get('narrative_text');
    const choicesJson = formData.get('choices');
    const npcRefsJson = formData.get('npc_refs');

    if (sceneId === null || narrativeText === null || choicesJson === null) {
      return fail(400, { error: 'Missing required fields for entering scene.' });
    }

    let choices: Array<{ label: string; target_scene_id: string }>;
    try {
      choices = JSON.parse(choicesJson.toString());
    } catch {
      return fail(400, { error: 'Invalid choices data.' });
    }

    let npcRefs: string[] | undefined;
    if (npcRefsJson) {
      try {
        npcRefs = JSON.parse(npcRefsJson.toString());
      } catch {
        return fail(400, { error: 'Invalid NPC refs data.' });
      }
    }

    try {
      await enterScene({
        session_id: params.session_id,
        scene_id: sceneId.toString(),
        narrative_text: narrativeText.toString(),
        choices,
        npc_refs: npcRefs,
      });
      return { success: true };
    } catch (err) {
      mapApiError(err);
    }
  },
};
