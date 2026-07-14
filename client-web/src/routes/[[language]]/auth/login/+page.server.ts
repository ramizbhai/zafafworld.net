import { fail, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';

import type { Actions, PageServerLoad } from './$types';
import { z } from 'zod';

const loginSchema = z.object({
  email: z.string().email('Invalid email address format.'),
  password: z.string().min(8, 'Password must be at least 8 characters long.')
});

export const load: PageServerLoad = async ({ cookies }) => {
  const sessionToken = cookies.get('zafaf_client_session');
  if (sessionToken) {
    throw redirect(303, '/dashboard');
  }
};

export const actions: Actions = {
  default: async ({ request, cookies, fetch, url }) => {
    const rawData = await request.formData();
    const formData = Object.fromEntries(rawData);
    const parseResult = loginSchema.safeParse(formData);

    if (!parseResult.success) {
      return fail(400, { success: false, message: parseResult.error.issues[0].message });
    }

    const { email, password } = parseResult.data;

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    try {
      const response = await fetch(`${API_BASE}/api/v1/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, domain_type: 'Client' }),
      });

      const result = await response.json();

      if (!response.ok || result.status !== 'success') {
        return fail(401, { success: false, message: 'Invalid ID or Password' });
      }

      if (result.user?.role?.toLowerCase() !== 'client') {
        return fail(403, { success: false, message: 'Invalid ID or Password' });
      }

      if (result.token) {
        const isSecure = url.protocol === 'https:' || request.headers.get('x-forwarded-proto') === 'https';
        cookies.set('zafaf_client_session', result.token, {
          path: '/',
          httpOnly: true,
          secure: isSecure,
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 // 1 day
        });
      }

      return { success: true, user: result.user };
    } catch (e) {
      console.error('Login error:', e);
      return fail(500, { success: false, message: 'Invalid ID or Password' });
    }
  }
};
