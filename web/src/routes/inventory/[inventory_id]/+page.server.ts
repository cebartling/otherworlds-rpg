import type { Actions, PageServerLoad } from './$types';
import {
  getInventory,
  addItem,
  removeItem,
  equipItem,
  archiveInventory,
} from '$lib/server/api/inventory';
import { handleLoadError } from '$lib/server/api/errors';
import { fail, redirect } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const inventory = await getInventory(params.inventory_id);
    return { inventory };
  } catch (err) {
    handleLoadError(err);
  }
};

export const actions: Actions = {
  addItem: async ({ request, params }) => {
    const formData = await request.formData();
    const itemId = formData.get('item_id');

    if (!itemId || typeof itemId !== 'string' || itemId.trim().length === 0) {
      return fail(400, { action: 'addItem', error: 'Item ID is required.' });
    }

    try {
      await addItem({
        inventory_id: params.inventory_id,
        item_id: itemId.trim(),
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'addItem', success: true };
  },

  removeItem: async ({ request, params }) => {
    const formData = await request.formData();
    const itemId = formData.get('item_id');

    if (!itemId || typeof itemId !== 'string' || itemId.trim().length === 0) {
      return fail(400, { action: 'removeItem', error: 'Item ID is required.' });
    }

    try {
      await removeItem({
        inventory_id: params.inventory_id,
        item_id: itemId.trim(),
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'removeItem', success: true };
  },

  equipItem: async ({ request, params }) => {
    const formData = await request.formData();
    const itemId = formData.get('item_id');

    if (!itemId || typeof itemId !== 'string' || itemId.trim().length === 0) {
      return fail(400, { action: 'equipItem', error: 'Item ID is required.' });
    }

    try {
      await equipItem({
        inventory_id: params.inventory_id,
        item_id: itemId.trim(),
      });
    } catch (err) {
      handleLoadError(err);
    }

    return { action: 'equipItem', success: true };
  },

  archive: async ({ params }) => {
    try {
      await archiveInventory(params.inventory_id);
    } catch (err) {
      handleLoadError(err);
    }

    redirect(303, '/inventory');
  },
};
