import { Link, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import {
  LayoutDashboard,
  Globe,
  Route,
  ScrollText,
  Settings,
} from 'lucide-react';
import api from '@/lib/api';

const navItems = [
  { path: '/', label: 'nav.dashboard', icon: LayoutDashboard },
  { path: '/domains', label: 'nav.domains', icon: Globe },
  { path: '/routes', label: 'nav.routes', icon: Route },
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
      <div className="w-6 h-6 rounded-full bg-[var(--bg-tertiary)] flex items-center justify-center">
        <div className="w-4 h-4 rounded-full bg-[var(--border-default)] animate-pulse" />
      </div>
    );
  }

  if (user.avatar) {
    return (
      <img
        src={user.avatar}
        alt={user.username}
        className="w-6 h-6 rounded-full object-cover border border-[var(--border-default)]"
        onError={(e) => {
          (e.target as HTMLImageElement).style.display = 'none';
          (e.target as HTMLImageElement).nextElementSibling?.classList.remove('hidden');
        }}
      />
    );
  }

  return (
    <div className="w-6 h-6 rounded-full bg-blue-500 flex items-center justify-center text-white text-xs font-medium">
      {getInitials(user.username)}
    </div>
  );
}

export default function BottomNavbar() {
  const { t } = useTranslation();
  const location = useLocation();

  return (
    <nav className="fixed bottom-0 left-0 right-0 h-12 border-t border-[var(--border-default)] bg-[var(--bg-secondary)] flex items-center z-40">
      {/* Navigation items - evenly spaced */}
      <div className="flex-1 flex items-center justify-around">
        {navItems.map(({ path, label, icon: Icon }) => (
          <Link
            key={path}
            to={path}
            className={`flex flex-col items-center justify-center px-3 py-1 text-xs transition-colors ${
              location.pathname === path
                ? 'text-[var(--text-primary)]'
                : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
            }`}
          >
            <Icon size={16} />
            <span className="mt-0.5 truncate max-w-[60px]">{t(label)}</span>
          </Link>
        ))}
      </div>

      {/* Right side - user avatar */}
      <div className="pr-4">
        <UserAvatar />
      </div>
    </nav>
  );
}
