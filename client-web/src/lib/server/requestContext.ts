import { AsyncLocalStorage } from 'node:async_hooks';

export interface RequestContext {
  countryCode: string;
}

export const requestContextStore = new AsyncLocalStorage<RequestContext>();
