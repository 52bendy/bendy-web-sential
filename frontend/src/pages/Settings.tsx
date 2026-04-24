import { useTranslation } from 'react-i18next';
import { useThemeStore } from '@/store';
import { Moon, Sun } from 'lucide-react';

export default function Settings() {
  const { t } = useTranslation();
  const { dark, toggle } = useThemeStore();

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">{t('nav.settings')}</h1>
      <div className="max-w-lg space-y-6">
        <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
          <h2 className="text-sm font-medium mb-4">Appearance</h2>
          <label className="flex items-center justify-between cursor-pointer">
            <div>
              <div className="text-sm font-medium">{dark ? 'Dark Mode' : 'Light Mode'}</div>
              <div className="text-xs text-[var(--text-muted)]">Switch between light and dark theme</div>
            </div>
            <button
              onClick={toggle}
              className="p-2 rounded hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              {dark ? <Sun size={20} /> : <Moon size={20} />}
            </button>
          </label>
        </div>
      </div>
    </div>
  );
}
