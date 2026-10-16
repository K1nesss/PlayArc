import { redirect } from '@sveltejs/kit';

// "Dashboard" became Home (DOIT.md §2). Kept so old links and bookmarks still land.
export const load = () => {
	redirect(301, '/home');
};
