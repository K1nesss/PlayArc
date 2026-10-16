import { dev } from '$app/environment';
import { error } from '@sveltejs/kit';

// Component gallery for reviewing the UI kit. Not available in production builds.
export const load = () => {
	if (!dev && !import.meta.env.VITE_UI_GALLERY) {
		error(404, 'Not found');
	}
};
