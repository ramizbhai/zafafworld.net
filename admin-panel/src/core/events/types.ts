export type AppEvent = {
  type: '402_PAYMENT_REQUIRED';
  limitType: string;
  currentTier: string;
  message: string;
};
