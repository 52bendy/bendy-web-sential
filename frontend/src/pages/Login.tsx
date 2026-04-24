import { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import api from '@/lib/api';
import { useAuthStore } from '@/store';

export default function Login() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { setToken } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [form, setForm] = useState({ username: '', password: '' });
  const [searchParams] = useSearchParams();

  // Handle OAuth callback token or SSO token exchange
  useEffect(() => {
    const token = searchParams.get('token');
    if (token) {
      // Check if it's an SSO token (contains dots) that needs exchange
      if (token.includes('.') && token.split('.').length === 3) {
        // It's an SSO token, exchange it for a JWT
        setLoading(true);
        api.post('/v1/auth/sso/exchange', { token })
          .then(({ data }) => {
            if (data.code === 0 && data.data?.token) {
              setToken(data.data.token);
              toast.success(t('auth.login'));
              navigate('/', { replace: true });
            } else {
              toast.error(data.message || t('auth.loginFailed'));
            }
          })
          .catch(() => {
            toast.error(t('auth.loginFailed'));
          })
          .finally(() => {
            setLoading(false);
          });
      } else {
        // It's already a JWT token from GitHub OAuth
        setToken(token);
        toast.success(t('auth.login'));
        navigate('/', { replace: true });
      }
    }
  }, [searchParams, setToken, navigate, t]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      const { data } = await api.post('/v1/auth/login', form);
      if (data.code === 0 && data.data?.token) {
        setToken(data.data.token);
        toast.success(t('auth.login'));
        navigate('/');
      } else {
        toast.error(data.message || t('auth.loginFailed'));
      }
    } catch {
      toast.error(t('auth.loginFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleGithubLogin = () => {
    window.location.href = '/api/auth/github/login';
  };

  return (
    <div className="min-h-screen flex items-center justify-center">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-sm p-8 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)] shadow-sm"
      >
        <h1 className="text-xl font-semibold mb-6 text-center">{t('auth.login')}</h1>
        <div className="space-y-4">
          <div>
            <label className="block text-sm mb-1">{t('auth.username')}</label>
            <input
              type="text"
              value={form.username}
              onChange={(e) => setForm({ ...form, username: e.target.value })}
              className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
              required
            />
          </div>
          <div>
            <label className="block text-sm mb-1">{t('auth.password')}</label>
            <input
              type="password"
              value={form.password}
              onChange={(e) => setForm({ ...form, password: e.target.value })}
              className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)]"
              required
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="w-full py-2 rounded bg-[var(--accent)] text-[var(--accent)] dark:bg-[var(--accent)] dark:text-[var(--bg-primary)] bg-black text-white hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {loading ? t('common.loading') : t('auth.loginButton')}
          </button>

          <div className="relative flex items-center justify-center my-4">
            <div className="border-t border-[var(--border-default)] w-full"></div>
            <span className="px-3 text-xs text-[var(--text-muted)] bg-[var(--bg-secondary)] absolute left-1/2 -translate-x-1/2 bg-[var(--bg-secondary)]">or</span>
            <div className="border-t border-[var(--border-default)] w-full"></div>
          </div>

          <button
            type="button"
            onClick={handleGithubLogin}
            disabled={loading}
            className="w-full py-2 rounded border border-[var(--border-default)] hover:bg-[var(--bg-tertiary)] transition-colors flex items-center justify-center gap-2 text-sm"
          >
            <svg height="16" width="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
            </svg>
            {t('auth.loginWithGithub')}
          </button>
        </div>
      </form>
    </div>
  );
}
