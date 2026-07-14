import type { AppEvent } from './types';

type EventCallback<T extends AppEvent['type']> = (
  payload: Omit<Extract<AppEvent, { type: T }>, 'type'>
) => void;

class EventBus {
  private listeners: { [K in AppEvent['type']]?: EventCallback<K>[] } = {};

  public on<T extends AppEvent['type']>(event: T, handler: EventCallback<T>): void {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event]!.push(handler);
  }

  public off<T extends AppEvent['type']>(event: T, handler: EventCallback<T>): void {
    if (!this.listeners[event]) return;
    this.listeners[event] = this.listeners[event]!.filter((h) => h !== handler) as any;
  }

  public emit<T extends AppEvent['type']>(
    event: T, 
    payload: Omit<Extract<AppEvent, { type: T }>, 'type'>
  ): void {
    if (!this.listeners[event]) return;
    this.listeners[event]!.forEach((handler) => {
      try {
        handler(payload);
      } catch (err) {
        console.error(`Error in event handler for ${event}:`, err);
      }
    });
  }
}

export const eventBus = new EventBus();
