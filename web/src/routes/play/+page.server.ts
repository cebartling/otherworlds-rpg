import { listNarrativeSessions } from '$lib/server/api/narrative';
import { handleLoadError } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
  try {
    const sessions = await listNarrativeSessions();
    return { sessions };
  } catch (err) {
    handleLoadError(err);
  }
};
