import { useTranslation } from 'react-i18next';
import { Globe, Route, Activity, AlertCircle } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import api from '@/lib/api';
import type { MetricsData } from '@/types';

export default function Dashboard() {
  const { t } = useTranslation();

  const { data } = useQuery<{ code: number; data: MetricsData }>({
    queryKey: ['metrics'],
    queryFn: async () => {
      const { data } = await api.get('/v1/metrics');
      return data;
    },
    refetchInterval: 30000,
  });

  const metrics = data?.data;

  const cards = [
    {
      label: t('dashboard.totalRequests'),
      value: metrics?.total_requests ?? '-',
      icon: Activity,
      color: 'text-blue-500',
    },
    {
      label: t('dashboard.domainsCount'),
      value: metrics?.domains_count ?? '-',
      icon: Globe,
      color: 'text-green-500',
    },
    {
      label: t('dashboard.activeRoutes'),
      value: metrics?.active_routes ?? '-',
      icon: Route,
      color: 'text-orange-500',
    },
    {
      label: 'Circuit Breaker',
      value: metrics?.circuit_breaker_state ?? '-',
      icon: AlertCircle,
      color: metrics?.circuit_breaker_state === 'Open' ? 'text-red-500' : 'text-green-500',
    },
  ];

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">{t('dashboard.title')}</h1>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {cards.map(({ label, value, icon: Icon, color }) => (
          <div
            key={label}
            className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]"
          >
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm text-[var(--text-secondary)]">{label}</span>
              <Icon size={20} className={color} />
            </div>
            <div className="text-2xl font-semibold">{value}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
