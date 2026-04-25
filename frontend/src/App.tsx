import { BrowserRouter, Routes, Route, Navigate, useSearchParams, useNavigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'sonner';
import Layout from '@/components/Layout';
import Login from '@/pages/Login';
import Dashboard from '@/pages/Dashboard';
import Domains from '@/pages/Domains';
import RoutesPage from '@/pages/Routes';
import AuditLog from '@/pages/AuditLog';
import Settings from '@/pages/Settings';
import RouteTest from '@/pages/RouteTest';
import { useAuthStore } from '@/store';
import api from '@/lib/api';
import { useEffect, useState } from 'react';
import '@/i18n';
import { useAuthHandler } from '@/store';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 5000,
    },
  },
});

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore();
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

function cleanUrl() {
  const url = new URL(window.location.href);
  url.searchParams.delete('token');
  url.searchParams.delete('jti');
  url.searchParams.delete('error');
  url.searchParams.delete('error_description');
  window.history.replaceState({}, '', url.toString());
}

// GitHub OAuth Token Handler: ONLY component that processes OAuth tokens
function GitHubTokenHandler() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { setToken, isAuthenticated } = useAuthStore();
  const [status, setStatus] = useState<'idle' | 'verifying' | 'success' | 'error'>('idle');
  const [processed, setProcessed] = useState(false);

  useEffect(() => {
    // Only process once per token value
    const token = searchParams.get('token');
    const error = searchParams.get('error');

    // Handle error from OAuth provider
    if (error) {
      console.log('[GitHubTokenHandler] OAuth error:', error);
      // Don't process the error here - Login.tsx handles it
      setProcessed(true);
      return;
    }

    // No token to process
    if (!token) {
      setProcessed(true);
      return;
    }

    // Already authenticated and this token is already processed
    if (processed && isAuthenticated) {
      return;
    }

    // Skip if we're already processing
    if (status === 'verifying') {
      return;
    }

    // Skip if already processed this token (avoid re-processing)
    const urlToken = window.location.search.match(/token=([^&]*)/)?.[1];
    if (processed && urlToken !== token) {
      setProcessed(false);
    }

    setStatus('verifying');
    console.log('[GitHubTokenHandler] Starting token verification for:', token.substring(0, 20) + '...');

    // Verify token with backend - directly validate JWT with /me endpoint
    api.get('/v1/auth/me', {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    })
      .then(({ data }) => {
        console.log('[GitHubTokenHandler] Token verified successfully, user:', data.data);
        // Clean URL first
        cleanUrl();
        // Set token after URL is cleaned
        setToken(token);
        setStatus('success');
        // Navigate after a tiny delay to ensure state updates
        setTimeout(() => {
          navigate('/', { replace: true });
        }, 50);
      })
      .catch((err) => {
        console.error('[GitHubTokenHandler] Token verification failed:', err);
        setStatus('error');
        // Stay on login page, don't set token
      });
  }, [searchParams, setToken, isAuthenticated, navigate, status, processed]);

  return null;
}

function AuthEventWrapper() {
  useAuthHandler();
  return null;
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AuthEventWrapper />
        <GitHubTokenHandler />
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <Layout />
              </ProtectedRoute>
            }
          >
            <Route index element={<Dashboard />} />
            <Route path="domains" element={<Domains />} />
            <Route path="routes" element={<RoutesPage />} />
            <Route path="audit-log" element={<AuditLog />} />
            <Route path="routes/test/:id" element={<RouteTest />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
      <Toaster position="top-right" />
    </QueryClientProvider>
  );
}