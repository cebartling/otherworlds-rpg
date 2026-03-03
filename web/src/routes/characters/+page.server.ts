import type { Actions, PageServerLoad } from './$types';
import { listCharacters, createCharacter } from '$lib/server/api/character';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async () => {
  try {
    const characters = await listCharacters();
    return { characters };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  create: async ({ request }) => {
    const formData = await request.formData();
    const name = formData.get('name');

    if (!name || typeof name !== 'string' || name.trim().length === 0) {
      return fail(400, { error: 'Character name is required.' });
    }

    try {
      await createCharacter({ name: name.trim() });
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/characters');
  },
};
