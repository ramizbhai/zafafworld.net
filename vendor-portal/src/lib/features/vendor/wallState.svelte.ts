import { getApiUrl } from '$lib/utils/api';

export interface Message {
    id: string;
    sender: 'vendor' | 'admin';
    body: string;
    file_url?: string;
    is_read: boolean;
    created_at: string;
}

export class WallState {
    messages = $state<Message[]>([]);
    newMessage = $state('');
    loadingMessages = $state(true);
    sendingMessage = $state(false);
    
    attachedFile = $state<File | null>(null);
    attachedPreviewUrl = $state('');
    lightboxUrl = $state('');
    
    sessionToken = $state('');
    pollingInterval: any;
    chatContainer: HTMLDivElement | null = $state(null);

    constructor() {}
    
    setSessionToken(token: string) {
        this.sessionToken = token;
    }

    handleFileChange(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            this.attachedFile = target.files[0];
            this.attachedPreviewUrl = URL.createObjectURL(this.attachedFile);
        }
    }

    removeAttachment(fileInputEl: HTMLInputElement | null) {
        this.attachedFile = null;
        if (this.attachedPreviewUrl) {
            URL.revokeObjectURL(this.attachedPreviewUrl);
            this.attachedPreviewUrl = '';
        }
        if (fileInputEl) {
            fileInputEl.value = '';
        }
    }

    async fetchMessages(silent = false) {
        if (!silent) this.loadingMessages = true;
        try {
            const response = await fetch(getApiUrl('/api/v1/vendor/tickets/messages'), {
                headers: {
                    'Authorization': `Bearer ${this.sessionToken}`
                }
            });
            if (response.ok) {
                const data = await response.json();
                if (data.status === 'success') {
                    this.messages = data.messages || [];
                    this.scrollToBottom();
                }
            }
        } catch (err) {
            console.error('Failed to load chat messages:', err);
        } finally {
            if (!silent) this.loadingMessages = false;
        }
    }

    async sendMessage(fileInputEl: HTMLInputElement | null) {
        if ((!this.newMessage.trim() && !this.attachedFile) || this.sendingMessage) return;
        this.sendingMessage = true;
        try {
            const formData = new FormData();
            if (this.newMessage.trim()) {
                formData.append('body', this.newMessage.trim());
            }
            if (this.attachedFile) {
                formData.append('file', this.attachedFile);
            }

            const response = await fetch(getApiUrl('/api/v1/vendor/tickets/reply'), {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${this.sessionToken}`
                },
                body: formData
            });

            if (response.ok) {
                this.newMessage = '';
                this.removeAttachment(fileInputEl);
                await this.fetchMessages(true);
            }
        } catch (err) {
            console.error('Failed to send message:', err);
        } finally {
            this.sendingMessage = false;
        }
    }

    scrollToBottom() {
        setTimeout(() => {
            if (this.chatContainer) {
                this.chatContainer.scrollTop = this.chatContainer.scrollHeight;
            }
        }, 50);
    }
    
    startPolling() {
        this.fetchMessages();
        this.pollingInterval = setInterval(() => {
            this.fetchMessages(true);
        }, 4000);
    }
    
    stopPolling() {
        if (this.pollingInterval) clearInterval(this.pollingInterval);
        if (this.attachedPreviewUrl) URL.revokeObjectURL(this.attachedPreviewUrl);
    }
}
