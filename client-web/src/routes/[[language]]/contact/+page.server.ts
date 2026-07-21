import { env } from '$env/dynamic/public';
import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const formData = await request.formData();
    const name = (formData.get('name') as string || '').trim();
    const email = (formData.get('email') as string || '').trim();
    const phone = (formData.get('phone') as string || '').trim();
    const subject = (formData.get('subject') as string || '').trim();
    const message = (formData.get('message') as string || '').trim();

    if (!name || !email || !subject || !message) {
      return fail(400, {
        error: 'Please fill out all required fields.',
        values: { name, email, phone, subject, message }
      });
    }

    try {
      // Use internal container endpoint when running server-side for zero-latency, SSL/CORS-free submission
      const apiBase = process.env.NODE_ENV === 'production' 
        ? 'http://backend:8080' 
        : (env.PUBLIC_API_URL || 'http://localhost:8080');

      const response = await fetch(`${apiBase}/api/v1/public/support`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          name,
          email,
          phone: phone ? phone : null,
          subject,
          message
        })
      });

      const resData = await response.json();

      if (response.ok && resData.status === 'success') {
        return { success: true };
      }

      return fail(400, {
        error: resData.message || 'Failed to submit support message. Please try again.',
        values: { name, email, phone, subject, message }
      });
    } catch (err) {
      console.error('[Contact Form SSR Action] Error submitting support message:', err);
      return fail(500, {
        error: 'Server connection error. Please try again later.',
        values: { name, email, phone, subject, message }
      });
    }
  }
};
