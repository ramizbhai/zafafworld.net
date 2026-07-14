import { fail } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { Actions } from './$types';

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const data = await request.formData();
    const email = data.get('email')?.toString().trim().toLowerCase();

    if (!email || !email.includes('@')) {
      return fail(400, { success: false, message: 'A valid email address is required.' });
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    try {
      const response = await fetch(`${API_BASE}/api/v1/auth/forgot-password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, domain_type: 'Client' }),
      });

      const result = await response.json();

      if (!response.ok || result.status !== 'success') {
        return fail(response.status === 400 ? 400 : 500, {
          success: false,
          message: result.message || 'An unexpected error occurred during password recovery.'
        });
      }

      return { success: true, message: result.message };
    } catch (e) {
      console.error('Forgot password error:', e);
      return fail(500, { success: false, message: 'An unexpected system error occurred.' });
    }
  }
};
