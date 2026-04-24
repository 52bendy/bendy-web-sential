import { create } from 'zustand';

interface AuthState {
  token: string | null;
  setToken: (token: string | null) => void;
  isAuthenticated: boolean;
}

export const useAuthStore = create<AuthState>((set) => ({
  token: localStorage.getItem('bws_token'),
  setToken: (token) => {
    if (token) {
      localStorage.setItem('bws_token', token);
    } else {
      localStorage.removeItem('bws_token');
    }
    set({ token, isAuthenticated: !!token });
  },
  isAuthenticated: !!localStorage.getItem('bws_token'),
}));

interface ThemeState {
  dark: boolean;
  toggle: () => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  dark: localStorage.getItem('bws_theme') === 'dark',
  toggle: () => {
    set((state) => {
      const next = !state.dark;
      localStorage.setItem('bws_theme', next ? 'dark' : 'light');
      document.documentElement.classList.toggle('dark', next);
      return { dark: next };
    });
  },
}));
