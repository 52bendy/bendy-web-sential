import axios, { AxiosError } from 'axios';
import { toast } from 'sonner';
import type { ApiResponse } from '@/types';

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Track if we've shown the unauthorized toast to avoid duplicates
let unauthorizedToastShown = false;

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('bws_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  // Reset toast flag on new requests (except for auth endpoints)
  if (!config.url?.includes('/auth/')) {
    unauthorizedToastShown = false;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error: AxiosError<ApiResponse>) => {
    const status = error.response?.status;
    const data = error.response?.data;
    const message = data?.message || error.message;

    if (status === 401) {
      // Clear token but DON'T do hard redirect - let React handle it
      localStorage.removeItem('bws_token');

      // Only show toast once per session and not for initial auth checks
      const isAuthEndpoint = error.config?.url?.includes('/auth/me');
      if (!unauthorizedToastShown && !isAuthEndpoint) {
        toast.error(message || 'Unauthorized');
        unauthorizedToastShown = true;
      }

      // Dispatch custom event that components can listen to
      window.dispatchEvent(new CustomEvent('auth:unauthorized', {
        detail: { message }
      }));

      return Promise.reject(error);
    }

    if (status === 429) {
      toast.error(message || 'Rate limited');
      return Promise.reject(error);
    }

    if (data?.code) {
      toast.error(message);
    } else {
      toast.error('Network error');
    }

    return Promise.reject(error);
  }
);

export default api;