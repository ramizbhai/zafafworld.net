import { eventBus } from './eventBus';

let received = false;

eventBus.on('402_PAYMENT_REQUIRED', (payload) => {
  console.log('Received 402 event with payload:', payload);
  if (payload.limitType === 'listing_limit') {
    received = true;
  }
});

eventBus.emit('402_PAYMENT_REQUIRED', {
  limitType: 'listing_limit',
  currentTier: 'Free',
  message: 'Upgrade to premium to add more listings'
});

if (received) {
  console.log('Test PASSED: event bus successfully emitted and received 402_PAYMENT_REQUIRED');
} else {
  console.error('Test FAILED');
  process.exit(1);
}
