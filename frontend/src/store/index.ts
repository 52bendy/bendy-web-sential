import { create } from 'zustand';
import { useEffect } from 'react';
import type { MenuPosition } from '@/types';
import { toast } from 'sonner';
import { useNavigate } from 'react-router-dom';

interface AuthState {
  token: string | null;
  setToken: (token: string | null) => void;
  isAuthenticated: boolean;
  logout: () => void;
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
  logout: () => {
    localStorage.removeItem('bws_token');
    set({ token: null, isAuthenticated: false });
  },
}));

// Hook to handle auth:unauthorized events - use in App component
export function useAuthHandler() {
  const logout = useAuthStore((s) => s.logout);
  const navigate = useNavigate();

  useEffect(() => {
    const handleUnauthorized = (e: Event) => {
      const customEvent = e as CustomEvent<{ message?: string }>;
      console.log('[AuthHandler] Unauthorized event:', customEvent.detail);
      logout();
      // Only navigate if not already on login page
      if (!window.location.pathname.includes('/login')) {
        toast.error(customEvent.detail?.message || 'Session expired');
        navigate('/login', { replace: true });
      }
    };

    window.addEventListener('auth:unauthorized', handleUnauthorized);
    return () => window.removeEventListener('auth:unauthorized', handleUnauthorized);
  }, [logout, navigate]);
}

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

interface LayoutState {
  menuPosition: MenuPosition;
  setMenuPosition: (position: MenuPosition) => void;
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
}

export const useLayoutStore = create<LayoutState>((set) => ({
  menuPosition: (localStorage.getItem('bws_menu_position') as MenuPosition) || 'top',
  setMenuPosition: (position) => {
    localStorage.setItem('bws_menu_position', position);
    set({ menuPosition: position });
  },
  sidebarCollapsed: localStorage.getItem('bws_sidebar_collapsed') === 'true',
  toggleSidebar: () => {
    set((state) => {
      const next = !state.sidebarCollapsed;
      localStorage.setItem('bws_sidebar_collapsed', String(next));
      return { sidebarCollapsed: next };
    });
  },
}));