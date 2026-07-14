import { getApiUrl, safeFetch, type ApiResponse } from '../utils/api';

// ── Types & Interfaces ────────────────────────────────────────────────────────

export interface UserDto {
  id: string;
  email: string;
  first_name?: string;
  last_name?: string;
  role: string;
  status: string;
  scopes?: string[];
}

export interface LoginResponse {
  status: string;
  token: string;
  user: UserDto;
}

export interface RegisterResponse {
  status: string;
  token?: string;
  message?: string;
}

export interface StaffDto {
  id: string;
  name: string;
  email: string;
  role: string;
  status: string;
}

export interface TemplateDto {
  id: string;
  name?: string;
  template_name?: string;
  body?: string;
  body_text_ar?: string;
  body_text_en?: string;
  created_at?: string;
}

export interface SubscriptionRequestDto {
  id: string;
  requested_tier_id: string;
  status: string;
  created_at: string;
}

export interface ReviewDto {
  id: string;
  reviewerName?: string;
  couple_name?: string;
  rating: number;
  comment: string;
  date?: string;
  created_at?: string;
  status: 'pending' | 'approved' | 'rejected' | string;
}

export interface TaskDto {
  id: string;
  title: string;
  done: boolean;
  category: string;
}

export interface LeadInquiryDto {
  id: string;
  vendor_id: string;
  customer_name: string;
  phone?: string;
  customer_phone?: string;
  wedding_date?: string;
  message?: string;
  status: string;
  created_at: string;
}

export interface VendorInquiryDto {
  id: string;
  vendor_id: string;
  client_name: string;
  email?: string;
  phone?: string;
  event_date?: string;
  message?: string;
  status: string;
  created_at: string;
}

export interface CompetitorDto {
  rank: number;
  page: string;
  city: string;
  district?: string;
  reviews: number;
  couples: number;
  conversion: number;
}

// ── API Client Implementation ──────────────────────────────────────────────────

class ApiClient {
  private fetchFn: typeof fetch;

  constructor(fetchFn: typeof fetch = fetch) {
    this.fetchFn = fetchFn;
  }

  /**
   * Return a new instance of ApiClient bound to a custom fetch function.
   * Useful in SvelteKit server-side load functions to pass down the correct fetch hook.
   */
  public withFetch(customFetch: typeof fetch): ApiClient {
    return new ApiClient(customFetch);
  }

  // ── Auth Endpoints ──────────────────────────────────────────────────────────

  public auth = {
    login: async (payload: any): Promise<ApiResponse<LoginResponse>> => {
      return safeFetch<LoginResponse>(this.fetchFn, getApiUrl('/api/v1/auth/login'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...payload, domain_type: 'Vendor' })
      });
    },

    register: async (payload: any): Promise<ApiResponse<RegisterResponse>> => {
      return safeFetch<RegisterResponse>(this.fetchFn, getApiUrl('/api/v1/public/vendor/register'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
    },

    forgotPassword: async (email: string): Promise<ApiResponse<{ status: string; message: string }>> => {
      return safeFetch<{ status: string; message: string }>(this.fetchFn, getApiUrl('/api/v1/auth/forgot-password'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email })
      });
    },

    resetPassword: async (payload: any): Promise<ApiResponse<{ status: string; message: string }>> => {
      return safeFetch<{ status: string; message: string }>(this.fetchFn, getApiUrl('/api/v1/auth/reset-password'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
    },

    logout: async (token: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl('/api/v1/auth/logout'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    me: async (token: string): Promise<ApiResponse<{ status: string; user: UserDto }>> => {
      return safeFetch<{ status: string; user: UserDto }>(this.fetchFn, getApiUrl('/api/v1/auth/me'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    }
  };

  // ── Vendor Endpoints ────────────────────────────────────────────────────────

  public vendor = {
    // ── Media Upload & Processing Pipeline ──
    uploadMedia: async (token: string, formData: FormData): Promise<ApiResponse<{
      status: string;
      id: string;
      status_state: string;
      url: string;
      file_path: string;
      media_type: string;
      thumbnail_url?: string;
      file_size: number;
      duration_seconds?: number;
    }>> => {
      return safeFetch<{
        status: string;
        id: string;
        status_state: string;
        url: string;
        file_path: string;
        media_type: string;
        thumbnail_url?: string;
        file_size: number;
        duration_seconds?: number;
      }>(this.fetchFn, getApiUrl('/api/v1/vendor/upload'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: formData,
        timeoutMs: 600000 // 10 minutes timeout for large video uploads
      });
    },

    getUploadStatus: async (token: string, id: string): Promise<ApiResponse<{
      status: string;
      data: {
        id: string;
        status: string;
        file_name: string;
        file_url: string;
        mime_type: string;
        file_size: number;
        error_message?: string;
      }
    }>> => {
      return safeFetch<{
        status: string;
        data: {
          id: string;
          status: string;
          file_name: string;
          file_url: string;
          mime_type: string;
          file_size: number;
          error_message?: string;
        }
      }>(this.fetchFn, getApiUrl(`/api/v1/vendor/upload/status/${id}`), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    // ── Staff ──
    getStaff: async (token: string): Promise<ApiResponse<{ status: string; staff: StaffDto[] }>> => {
      return safeFetch<{ status: string; staff: StaffDto[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/staff'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    createStaff: async (token: string, payload: any): Promise<ApiResponse<{ status: string; staff: StaffDto }>> => {
      return safeFetch<{ status: string; staff: StaffDto }>(this.fetchFn, getApiUrl('/api/v1/vendor/staff'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    updateStaff: async (token: string, id: string, payload: any): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/staff/${id}`), {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    updateStaffStatus: async (token: string, id: string, status: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/staff/${id}/status`), {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify({ status })
      });
    },

    deleteStaff: async (token: string, id: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/staff/${id}`), {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    // ── Templates ──
    getTemplates: async (token: string): Promise<ApiResponse<{ status: string; templates: TemplateDto[] }>> => {
      return safeFetch<{ status: string; templates: TemplateDto[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/templates'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    createTemplate: async (token: string, payload: any): Promise<ApiResponse<{ status: string; template: TemplateDto }>> => {
      return safeFetch<{ status: string; template: TemplateDto }>(this.fetchFn, getApiUrl('/api/v1/vendor/templates'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    updateTemplate: async (token: string, id: string, payload: any): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/templates/${id}`), {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    deleteTemplate: async (token: string, id: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/templates/${id}`), {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    // ── Subscriptions ──
    getSubscriptionRequests: async (token: string): Promise<ApiResponse<{ status: string; requests: SubscriptionRequestDto[] }>> => {
      return safeFetch<{ status: string; requests: SubscriptionRequestDto[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/subscription-requests'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    createSubscriptionRequest: async (token: string, requested_tier_id: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl('/api/v1/vendor/subscription-requests'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify({ requested_tier_id })
      });
    },

    // ── Reviews ──
    getReviews: async (token: string, queryParams?: string): Promise<ApiResponse<{ status: string; reviews: ReviewDto[]; total_count?: number }>> => {
      const path = `/api/v1/vendor/reviews${queryParams ? '?' + queryParams : ''}`;
      return safeFetch<{ status: string; reviews: ReviewDto[]; total_count?: number }>(this.fetchFn, getApiUrl(path), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    updateReviewStatus: async (token: string, id: string, status: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/reviews/${id}/status`), {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify({ status })
      });
    },

    // ── Tasks ──
    getTasks: async (token: string): Promise<ApiResponse<{ status: string; tasks: TaskDto[] }>> => {
      return safeFetch<{ status: string; tasks: TaskDto[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/tasks'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    createTask: async (token: string, payload: any): Promise<ApiResponse<{ status: string; task: TaskDto }>> => {
      return safeFetch<{ status: string; task: TaskDto }>(this.fetchFn, getApiUrl('/api/v1/vendor/tasks'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    updateTask: async (token: string, id: string, payload: any): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/tasks/${id}`), {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    deleteTask: async (token: string, id: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/tasks/${id}`), {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    // ── Stats ──
    getDashboardStats: async (token: string): Promise<ApiResponse<{ status: string; data: any }>> => {
      return safeFetch<{ status: string; data: any }>(this.fetchFn, getApiUrl('/api/v1/vendor/stats/dashboard'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    // Mocked competitors API client layer resolving empty state elegantly
    getCompetitors: async (token: string): Promise<ApiResponse<{ status: string; competitorsCity: CompetitorDto[]; competitorsService: CompetitorDto[] }>> => {
      console.warn('Backend API `/competitors` not implemented. Gracefully resolving empty array dataset.');
      return {
        success: true,
        status: 200,
        data: {
          status: 'success',
          competitorsCity: [],
          competitorsService: []
        },
        error: null
      };
    },

    // ── Inquiries (Lead Inquiries) ──
    getInquiries: async (token: string): Promise<ApiResponse<{ status: string; inquiries: LeadInquiryDto[] }>> => {
      return safeFetch<{ status: string; inquiries: LeadInquiryDto[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/inquiries'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    updateInquiryStatus: async (token: string, id: string, status: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/inquiries/${id}/status`), {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify({ status })
      });
    },

    // ── Direct Vendor Inquiries ──
    getVendorInquiries: async (token: string): Promise<ApiResponse<{ status: string; inquiries: VendorInquiryDto[] }>> => {
      return safeFetch<{ status: string; inquiries: VendorInquiryDto[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/vendor_inquiries'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    updateVendorInquiryStatus: async (token: string, id: string, status: string): Promise<ApiResponse<{ status: string }>> => {
      return safeFetch<{ status: string }>(this.fetchFn, getApiUrl(`/api/v1/vendor/vendor_inquiries/${id}/status`), {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify({ status })
      });
    },

    // ── Products (Halls) ──
    getProducts: async (token: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/products'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getProductEditContext: async (token: string, id: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/products/${id}/edit-context`), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    deleteProduct: async (token: string, id: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/products/${id}`), {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getProductAvailability: async (token: string, id: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/products/${id}/availability`), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    linkProductImage: async (token: string, productId: string, payload: any): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/products/${productId}/images`), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    deleteProductImage: async (token: string, productId: string, imageId: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/products/${productId}/images/${imageId}`), {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    setProductCoverImage: async (token: string, productId: string, imageId: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/products/${productId}/images/${imageId}/cover`), {
        method: 'PATCH',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    updateProfile: async (token: string, payload: any): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/profile'), {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    // ── Promotions (Offers) ──
    getPromotions: async (token: string, queryParams?: string): Promise<ApiResponse<any>> => {
      const path = `/api/v1/vendor/promotions${queryParams ? '?' + queryParams : ''}`;
      return safeFetch<any>(this.fetchFn, getApiUrl(path), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getPromotion: async (token: string, id: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/promotions/${id}`), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    createPromotion: async (token: string, payload: any): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/promotions'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    updatePromotion: async (token: string, id: string, payload: any): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl(`/api/v1/vendor/promotions/${id}`), {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify(payload)
      });
    },

    uploadPromoBanner: async (token: string, formData: FormData): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/promotions/upload-banner'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`
        },
        body: formData,
        timeoutMs: 600000
      });
    },

    cleanupPromoBanner: async (token: string, filename: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/promotions/cleanup-banner'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        },
        body: JSON.stringify({ filename })
      });
    },

    getPricing: async (token: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/pricing'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getConversations: async (token: string): Promise<ApiResponse<any>> => {
      return safeFetch<any>(this.fetchFn, getApiUrl('/api/v1/vendor/conversations'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getNotifications: async (token: string): Promise<ApiResponse<{ status: string; notifications: any[] }>> => {
      return safeFetch<{ status: string; notifications: any[] }>(this.fetchFn, getApiUrl('/api/v1/vendor/notifications'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getMetadataConfig: async (token: string): Promise<ApiResponse<{ data: any }>> => {
      return safeFetch<{ data: any }>(this.fetchFn, getApiUrl('/api/v1/metadata/vendor-config'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    },

    getTiers: async (token: string): Promise<ApiResponse<{ status: string; tiers: any[] }>> => {
      return safeFetch<{ status: string; tiers: any[] }>(this.fetchFn, getApiUrl('/api/v1/public/subscription/tiers'), {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Cookie': `zafaf_vendor_session=${token}`
        }
      });
    }
  };
}

export const apiClient = new ApiClient();
