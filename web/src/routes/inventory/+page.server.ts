import type { PageServerLoad } from './$types';
import { listInventories } from '$lib/server/api/inventory';
import { handleLoadError } from '$lib/server/api/errors';

export const load: PageServerLoad = async () => {
  try {
    const inventories = await listInventories();
    return { inventories };
  } catch (err) {
    handleLoadError(err);
  }
};
