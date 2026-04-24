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

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('bws_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
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
      localStorage.removeItem('bws_token');
      toast.error(message || 'Unauthorized');
      window.location.href = '/login';
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
