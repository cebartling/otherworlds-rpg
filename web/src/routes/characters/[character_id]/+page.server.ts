import type { Actions, PageServerLoad } from './$types';
import {
  getCharacter,
  modifyAttribute,
  awardExperience,
  archiveCharacter,
} from '$lib/server/api/character';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const character = await getCharacter(params.character_id);
    return { character };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  modifyAttribute: async ({ request, params }) => {
    const formData = await request.formData();
    const attribute = formData.get('attribute');
    const newValueRaw = formData.get('new_value');

    if (!attribute || typeof attribute !== 'string' || attribute.trim().length === 0) {
      return fail(400, { action: 'modifyAttribute', error: 'Attribute name is required.' });
    }

    if (!newValueRaw || typeof newValueRaw !== 'string') {
      return fail(400, { action: 'modifyAttribute', error: 'Attribute value is required.' });
    }

    const newValue = parseInt(newValueRaw, 10);
    if (isNaN(newValue)) {
      return fail(400, { action: 'modifyAttribute', error: 'Attribute value must be a number.' });
    }

    try {
      await modifyAttribute({
        character_id: params.character_id,
        attribute: attribute.trim(),
        new_value: newValue,
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'modifyAttribute', success: true };
  },

  awardExperience: async ({ request, params }) => {
    const formData = await request.formData();
    const amountRaw = formData.get('amount');

    if (!amountRaw || typeof amountRaw !== 'string') {
      return fail(400, { action: 'awardExperience', error: 'Amount is required.' });
    }

    const amount = parseInt(amountRaw, 10);
    if (isNaN(amount) || amount <= 0) {
      return fail(400, { action: 'awardExperience', error: 'Amount must be a positive number.' });
    }

    try {
      await awardExperience({
        character_id: params.character_id,
        amount,
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'awardExperience', success: true };
  },

  archive: async ({ params }) => {
    try {
      await archiveCharacter(params.character_id);
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/characters');
  },
};
