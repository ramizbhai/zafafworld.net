export interface CoupleInquiry {
  id: number;
  name: string;
  page: string;
  eventDate: string;
  receivedDate: string;
  waitTime: string;
  status: string;
}

export interface Subscription {
  id: number;
  page: string;
  plan: string;
  date: string;
  amount: string;
  status: string;
  canRenew: boolean;
}

export interface Competitor {
  rank: number;
  page: string;
  city: string;
  district: string;
  reviews: number;
  couples: number;
  conversion: number;
  calls: number;
  visits: number;
  convRate: number;
  avgReply: number;
  comments: number;
  offers: number;
  quality: number;
}

export interface Offer {
  id: number;
  title: string;
  discount: number;
  startDate: string;
  endDate: string;
  status: 'active' | 'expired' | 'pending';
  page: string;
}

export interface Review {
  id: number;
  reviewerName: string;
  rating: number;
  comment: string;
  date: string;
  status: 'pending' | 'approved' | 'rejected';
  weddingDate: string;
}

export interface VendorPage {
  id: number;
  title: string;
  category: string;
  status: 'published' | 'draft';
  description: string;
  visits: number;
  couples: number;
  rating: number;
}

export interface User {
  id: number;
  name: string;
  email: string;
  role: 'admin' | 'editor' | 'viewer';
  status: 'active' | 'inactive';
}

export interface Task {
  id: number;
  title: string;
  done: boolean;
  category: string;
}

export interface PricingPlan {
  id: string;
  name: string;
  name_en?: string;
  price: number;
  desc: string;
  popular?: boolean;
  current?: boolean;
  features: string[];
}
