import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Moon, Sun, Monitor, AlignLeft, AlignCenter } from 'lucide-react';
import { toast } from 'sonner';
import { useThemeStore, useLayoutStore } from '@/store';
import api from '@/lib/api';
import type { MenuPosition } from '@/types';

export default function Settings() {
  const { t } = useTranslation();
  const { dark, toggle } = useThemeStore();
  const { menuPosition, setMenuPosition } = useLayoutStore();

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">{t('nav.settings')}</h1>
      <div className="max-w-lg space-y-6">
        {/* Appearance */}
        <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
          <h2 className="text-sm font-medium mb-4">{t('settings.appearance')}</h2>
          <label className="flex items-center justify-between cursor-pointer">
            <div>
              <div className="text-sm font-medium">{dark ? t('settings.darkMode') : t('settings.lightMode')}</div>
              <div className="text-xs text-[var(--text-muted)]">{t('settings.appearanceDesc')}</div>
            </div>
            <button
              onClick={toggle}
              className="p-2 rounded hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              {dark ? <Sun size={20} /> : <Moon size={20} />}
            </button>
          </label>
        </div>

        {/* Menu Layout */}
        <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
          <h2 className="text-sm font-medium mb-4">{t('settings.menuLayout')}</h2>
          <div className="grid grid-cols-3 gap-3">
            <MenuPositionButton
              position="top"
              current={menuPosition}
              onClick={setMenuPosition}
              icon={<Monitor size={20} />}
              label={t('settings.menuTop')}
            />
            <MenuPositionButton
              position="left"
              current={menuPosition}
              onClick={setMenuPosition}
              icon={<AlignLeft size={20} />}
              label={t('settings.menuLeft')}
            />
            <MenuPositionButton
              position="bottom"
              current={menuPosition}
              onClick={setMenuPosition}
              icon={<AlignCenter size={20} />}
              label={t('settings.menuBottom')}
            />
          </div>
        </div>

        {/* User Profile */}
        <UserProfileForm />
      </div>
    </div>
  );
}

interface MenuPositionButtonProps {
  position: MenuPosition;
  current: MenuPosition;
  onClick: (position: MenuPosition) => void;
  icon: React.ReactNode;
  label: string;
}

function MenuPositionButton({ position, current, onClick, icon, label }: MenuPositionButtonProps) {
  const isActive = current === position;
  return (
    <button
      onClick={() => onClick(position)}
      className={`flex flex-col items-center gap-2 p-4 rounded-lg border transition-colors ${
        isActive
          ? 'border-blue-500 bg-blue-500/10 text-blue-500'
          : 'border-[var(--border-default)] text-[var(--text-secondary)] hover:border-[var(--text-muted)] hover:text-[var(--text-primary)]'
      }`}
    >
      {icon}
      <span className="text-xs font-medium">{label}</span>
    </button>
  );
}

function UserProfileForm() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const { data, isLoading } = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const { data } = await api.get('/v1/auth/me');
      return data;
    },
    staleTime: 5 * 60 * 1000,
  });

  // Handle both old API (authenticated: true) and new API (User object)
  const user = data?.data?.username ? data.data : null;

  const [username, setUsername] = useState('');
  const [avatar, setAvatar] = useState('');

  // Initialize form when user data loads
  useEffect(() => {
    if (user) {
      setUsername(user.username);
      setAvatar(user.avatar || '');
    }
  }, [user]);

  const mutation = useMutation({
    mutationFn: async (payload: { username?: string; avatar?: string }) => {
      const { data } = await api.put('/v1/auth/me', payload);
      return data;
    },
    onSuccess: () => {
      toast.success(t('settings.profileUpdated'));
      queryClient.invalidateQueries({ queryKey: ['user'] });
    },
    onError: () => {
      toast.error(t('settings.profileUpdateFailed'));
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    mutation.mutate({
      username: username || undefined,
      avatar: avatar || undefined,
    });
  };

  if (isLoading) {
    return (
      <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
        <h2 className="text-sm font-medium mb-4">{t('settings.userProfile')}</h2>
        <div className="animate-pulse space-y-3">
          <div className="h-10 bg-[var(--bg-tertiary)] rounded" />
          <div className="h-10 bg-[var(--bg-tertiary)] rounded" />
        </div>
      </div>
    );
  }

  return (
    <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
      <h2 className="text-sm font-medium mb-4">{t('settings.userProfile')}</h2>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm text-[var(--text-secondary)] mb-1">
            {t('settings.username')}
          </label>
          <input
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-sm"
            placeholder={user?.username || t('settings.usernamePlaceholder')}
          />
        </div>
        <div>
          <label className="block text-sm text-[var(--text-secondary)] mb-1">
            {t('settings.avatarUrl')}
          </label>
          <input
            type="url"
            value={avatar}
            onChange={(e) => setAvatar(e.target.value)}
            className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-sm"
            placeholder={t('settings.avatarUrlPlaceholder')}
          />
          {avatar && (
            <div className="mt-2">
              <img
                src={avatar}
                alt="Avatar preview"
                className="w-12 h-12 rounded-full object-cover border border-[var(--border-default)]"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = 'none';
                }}
              />
            </div>
          )}
        </div>
        <button
          type="submit"
          disabled={mutation.isPending}
          className="px-4 py-2 bg-blue-500 text-white text-sm rounded hover:bg-blue-600 disabled:opacity-50 transition-colors"
        >
          {mutation.isPending ? t('common.saving') : t('settings.saveProfile')}
        </button>
      </form>
    </div>
  );
}
