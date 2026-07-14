import { fail, redirect } from '@sveltejs/kit';
import { safeFetch } from '$lib/utils/api';
import { env } from '$env/dynamic/public';

import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
  const sessionToken = cookies.get('zafaf_client_session');
  if (sessionToken) {
    throw redirect(303, '/dashboard');
  }
  
  let cities = [];
  try {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    const response = await fetch(`${API_BASE}/api/v1/public/cities`);
    if (response.ok) {
        const data = await response.json();
        if (data.status === 'success' && data.cities) {
            cities = data.cities;
        }
    }
  } catch (err) {
    console.error('Failed to fetch cities for register:', err);
  }

  return { cities };
};

export const actions: Actions = {
  default: async ({ request, cookies, fetch, url }) => {
    const data = await request.formData();
    const first_name = data.get('firstName');
    const last_name = data.get('lastName');
    const email = data.get('email');
    const phone = data.get('phone');
    const password = data.get('password');
    const roleInput = (data.get('role') as string) || 'Client';
    const domain_type = roleInput.toLowerCase() === 'vendor' ? 'Vendor' : 'Client';
    
    // As per new spec, map to full_name and city
    const full_name = `${first_name} ${last_name}`.trim();
    const city = data.get('city') as string;

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    const result = await safeFetch<any>(fetch, `${API_BASE}/api/v1/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        email, 
        password, 
        domain_type,
        full_name,
        city,
        phone
      }),
    });

    if (!result.success) {
      return fail(result.status || 400, { success: false, message: result.error?.message || 'Registration failed.' });
    }

    const resultData = result.data;
    if (resultData && resultData.token) {
      const isSecure = url.protocol === 'https:' || request.headers.get('x-forwarded-proto') === 'https';
      cookies.set('zafaf_client_session', resultData.token, {
        path: '/',
        httpOnly: true,
        secure: isSecure,
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 // 1 day
      });
    }

    return { success: true, user: resultData?.user };
  }
};
