export interface NewsletterSubscriptionResult {
  success: boolean;
  message: string;
}

export interface NewsletterService {
  /**
   * Subscribes a user's email address to the newsletter list.
   * This is an interface-only definition for Phase 5b.
   * The actual implementation is deferred to a later integration phase.
   * 
   * @param email The validated email string to subscribe.
   */
  subscribe(email: string): Promise<NewsletterSubscriptionResult>;
}
