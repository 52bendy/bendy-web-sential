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

// SSO Token Auto-Login: handles token in URL for any page
function SsoAutoLogin() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { setToken, isAuthenticated } = useAuthStore();
  const [processed, setProcessed] = useState(false);

  useEffect(() => {
    // Skip if already authenticated or already processed
    if (isAuthenticated || processed) return;

    const token = searchParams.get('token');
    if (!token) return;

    // Check if it's an SSO token (contains dots) that needs exchange
    if (token.includes('.') && token.split('.').length === 3) {
      // It's an SSO token, exchange it for a JWT
      api.post('/v1/auth/sso/exchange', { token })
        .then(({ data }) => {
          if (data.code === 0 && data.data?.token) {
            setToken(data.data.token);
          }
        })
        .catch(() => {
          // If exchange fails, try setting as direct JWT (for GitHub OAuth flow)
          setToken(token);
        })
        .finally(() => {
          setProcessed(true);
          // Remove token from URL without navigation
          const url = new URL(window.location.href);
          url.searchParams.delete('token');
          url.searchParams.delete('jti');
          window.history.replaceState({}, '', url.toString());
        });
    } else {
      // Direct JWT token
      setToken(token);
      const url = new URL(window.location.href);
      url.searchParams.delete('token');
      url.searchParams.delete('jti');
      window.history.replaceState({}, '', url.toString());
      setProcessed(true);
    }
  }, [searchParams, setToken, isAuthenticated, processed, navigate]);

  return null; // Silent component, doesn't render anything
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <SsoAutoLogin />
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
