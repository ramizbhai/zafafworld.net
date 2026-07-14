import { json } from '@sveltejs/kit';
import { listingService } from '$lib/services/api/listing.service.js';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, fetch }) => {
  try {
    const { listingId, guestCount, eventDate } = await request.json();
    
    if (!listingId) {
      return json({ error: 'Missing listingId' }, { status: 400 });
    }

    const listing = await listingService.getById(listingId, fetch);
    
    if (!listing) {
      return json({ error: 'Listing not found' }, { status: 404 });
    }

    // In a real application, the backend Rust service would perform this logic securely.
    // As a Node BFF proxy, we implement the authoritative calculation here.
    const startingPrice = listing.startingPrice ?? (listing.basePriceSar ? parseFloat(listing.basePriceSar) : 0);
    const subtotal = startingPrice;
    
    // Calculate taxes securely on the server
    const taxRate = 0.15;
    const tax = Math.round(subtotal * taxRate);
    const total = subtotal + tax;
    
    // Calculate deposit securely on the server
    const depositPercentage = listing.depositPercentage ?? 25;
    const depositAmt = Math.round(total * (depositPercentage / 100));

    return json({
      subtotal,
      tax,
      total,
      depositAmt,
      depositPercentage,
      currency: listing.currency || 'SAR'
    });
  } catch (err) {
    console.error('[Pricing Engine API] Calculation error:', err);
    return json({ error: 'Failed to calculate price' }, { status: 500 });
  }
};
