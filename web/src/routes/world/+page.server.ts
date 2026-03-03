import type { PageServerLoad } from './$types';
import { listWorldSnapshots } from '$lib/server/api/world-state';
import { handleLoadError } from '$lib/server/api/errors';

export const load: PageServerLoad = async () => {
  try {
    const snapshots = await listWorldSnapshots();
    return { snapshots };
  } catch (err) {
    handleLoadError(err);
  }
};
