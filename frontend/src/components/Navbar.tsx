import { Link, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import i18n from '@/i18n';
import { Moon, Sun, LogOut, LayoutDashboard, Globe, Route, ScrollText, Settings } from 'lucide-react';
import { useThemeStore, useAuthStore } from '@/store';

const navItems = [
  { path: '/', label: 'nav.dashboard', icon: LayoutDashboard },
  { path: '/domains', label: 'nav.domains', icon: Globe },
  { path: '/routes', label: 'nav.routes', icon: Route },
  { path: '/audit-log', label: 'nav.auditLog', icon: ScrollText },
  { path: '/settings', label: 'nav.settings', icon: Settings },
];

export default function Navbar() {
  const { t } = useTranslation();
  const location = useLocation();
  const { dark, toggle } = useThemeStore();
  const { setToken } = useAuthStore();

  const handleLogout = () => {
    setToken(null);
    window.location.href = '/login';
  };

  return (
    <nav className="border-b border-[var(--border-default)] bg-[var(--bg-secondary)] px-6 py-3 flex items-center justify-between">
      <div className="flex items-center gap-1">
        <Link to="/" className="font-semibold text-lg mr-6">
          BWS Admin
        </Link>
        {navItems.map(({ path, label, icon: Icon }) => (
          <Link
            key={path}
            to={path}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors ${
              location.pathname === path
                ? 'bg-[var(--bg-tertiary)] text-[var(--text-primary)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Icon size={16} />
            <span>{t(label)}</span>
          </Link>
        ))}
      </div>
      <div className="flex items-center gap-3">
        <select
          className="bg-transparent text-sm border border-[var(--border-default)] rounded px-2 py-1"
          defaultValue={navigator.language.split('-')[0]}
          onChange={(e) => {
            i18n.changeLanguage(e.target.value);
          }}
        >
          <option value="en">EN</option>
          <option value="zh">中文</option>
        </select>
        <button
          onClick={toggle}
          className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] transition-colors"
          title={dark ? 'Light mode' : 'Dark mode'}
        >
          {dark ? <Sun size={18} /> : <Moon size={18} />}
        </button>
        <button
          onClick={handleLogout}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
        >
          <LogOut size={16} />
          <span>{t('auth.logout')}</span>
        </button>
      </div>
    </nav>
  );
}
