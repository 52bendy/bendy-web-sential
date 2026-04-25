import { Link, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { useLayoutStore, useAuthStore } from '@/store';
import {
  LayoutDashboard,
  Globe,
  Route,
  RefreshCw,
  ScrollText,
  Settings,
  ChevronLeft,
  ChevronRight,
  LogOut,
} from 'lucide-react';
import api from '@/lib/api';

const navItems = [
  { path: '/', label: 'nav.dashboard', icon: LayoutDashboard },
  { path: '/domains', label: 'nav.domains', icon: Globe },
  { path: '/routes', label: 'nav.routes', icon: Route },
  { path: '/rewrites', label: 'nav.rewrites', icon: RefreshCw },
  { path: '/audit-log', label: 'nav.auditLog', icon: ScrollText },
  { path: '/settings', label: 'nav.settings', icon: Settings },
];

function UserAvatar() {
  const { data } = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const { data } = await api.get('/v1/auth/me');
      return data;
    },
    staleTime: 5 * 60 * 1000,
  });

  // Handle both old API (authenticated: true) and new API (User object)
  const user = data?.data?.username ? data.data : null;

  const getInitials = (name: string) => {
    return name
      .split(' ')
      .map((n) => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  if (!user) {
    return (
      <div className="w-8 h-8 rounded-full bg-[var(--bg-tertiary)] flex items-center justify-center">
        <div className="w-6 h-6 rounded-full bg-[var(--border-default)] animate-pulse" />
      </div>
    );
  }

  if (user.avatar) {
    return (
      <img
        src={user.avatar}
        alt={user.username}
        className="w-8 h-8 rounded-full object-cover border border-[var(--border-default)]"
        onError={(e) => {
          (e.target as HTMLImageElement).style.display = 'none';
          (e.target as HTMLImageElement).nextElementSibling?.classList.remove('hidden');
        }}
      />
    );
  }

  return (
    <div className="w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-white text-xs font-medium">
      {getInitials(user.username)}
    </div>
  );
}

export default function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const { sidebarCollapsed, toggleSidebar } = useLayoutStore();
  const { setToken } = useAuthStore();

  const handleLogout = async () => {
    try {
      await api.post('/v1/auth/logout');
    } catch (_) {}
    setToken(null);
    window.location.href = '/login';
  };

  return (
    <aside
      className={`fixed left-0 top-0 h-full border-r border-[var(--border-default)] bg-[var(--bg-secondary)] transition-all duration-300 z-40 flex flex-col ${
        sidebarCollapsed ? 'w-16' : 'w-64'
      }`}
    >
      {/* Logo */}
      <div className="h-14 flex items-center justify-center border-b border-[var(--border-default)]">
        <Link to="/" className="font-semibold text-lg">
          {sidebarCollapsed ? 'BWS' : 'BWS Admin'}
        </Link>
      </div>

      {/* Nav Items */}
      <nav className="flex-1 py-4 overflow-y-auto">
        {navItems.map(({ path, label, icon: Icon }) => (
          <Link
            key={path}
            to={path}
            className={`flex items-center gap-3 px-4 py-3 text-sm transition-colors ${
              location.pathname === path
                ? 'bg-[var(--bg-tertiary)] text-[var(--text-primary)] border-r-2 border-blue-500'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
            }`}
          >
            <Icon size={20} className="flex-shrink-0" />
            {!sidebarCollapsed && <span>{t(label)}</span>}
          </Link>
        ))}
      </nav>

      {/* User Info */}
      {!sidebarCollapsed && (
        <div className="px-4 py-3 border-t border-[var(--border-default)] flex items-center gap-3">
          <UserAvatar />
          <button
            onClick={handleLogout}
            className="flex items-center gap-1.5 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          >
            <LogOut size={16} />
            <span>{t('auth.logout')}</span>
          </button>
        </div>
      )}

      {/* Collapse Toggle */}
      <button
        onClick={toggleSidebar}
        className="h-12 flex items-center justify-center border-t border-[var(--border-default)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
      >
        {sidebarCollapsed ? <ChevronRight size={20} /> : <ChevronLeft size={20} />}
      </button>
    </aside>
  );
}
