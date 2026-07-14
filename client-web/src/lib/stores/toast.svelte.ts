export interface ToastMessage {
  id: string;
  type: 'success' | 'error';
  message: string;
}

class ToastEngine {
  messages = $state<ToastMessage[]>([]);

  push(type: 'success' | 'error', message: string) {
    const id = crypto.randomUUID();
    this.messages.push({ id, type, message });
    setTimeout(() => {
      this.messages = this.messages.filter((m) => m.id !== id);
    }, 4000);
  }
}

export const toasts = new ToastEngine();
