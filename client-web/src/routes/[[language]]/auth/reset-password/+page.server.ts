import { fail } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { Actions } from './$types';

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const data = await request.formData();
    const token = data.get('token')?.toString().trim();
    const password = data.get('password')?.toString();
    const confirmPassword = data.get('confirmPassword')?.toString();

    if (!token) {
      return fail(400, { success: false, message: 'Invalid or missing recovery token.' });
    }

    if (!password || password.length < 8) {
      return fail(400, { success: false, message: 'Password must be at least 8 characters long.' });
    }

    if (password !== confirmPassword) {
      return fail(400, { success: false, message: 'Passwords do not match.' });
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    try {
      const response = await fetch(`${API_BASE}/api/v1/auth/reset-password`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token, password }),
      });

      const result = await response.json();

      if (!response.ok || result.status !== 'success') {
        return fail(response.status === 400 ? 400 : 500, {
          success: false,
          message: result.message || 'An error occurred while resetting your password.'
        });
      }

      return { success: true, message: result.message };
    } catch (e) {
      console.error('Reset password error:', e);
      return fail(500, { success: false, message: 'An unexpected system error occurred.' });
    }
  }
};
