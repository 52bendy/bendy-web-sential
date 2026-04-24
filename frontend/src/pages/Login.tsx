import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
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

  return (
    <div className="min-h-screen flex items-center justify-center bg-[var(--bg-primary)]">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-sm p-8 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]"
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
        </div>
      </form>
    </div>
  );
}
