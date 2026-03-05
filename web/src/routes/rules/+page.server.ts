import type { PageServerLoad } from './$types';
import { listResolutions } from '$lib/server/api/rules';
import { handleLoadError } from '$lib/server/api/errors';

export const load: PageServerLoad = async () => {
  try {
    const resolutions = await listResolutions();
    return { resolutions };
  } catch (err) {
    handleLoadError(err);
  }
};
