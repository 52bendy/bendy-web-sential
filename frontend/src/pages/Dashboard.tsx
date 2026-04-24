import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Globe, Route, Activity, AlertCircle, ArrowDownCircle, ArrowUpCircle } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import api from '@/lib/api';
import type { MetricsData, TrafficData } from '@/types';
import TrafficChart from '@/components/TrafficChart';

export default function Dashboard() {
  const { t } = useTranslation();
  const [refreshKey] = useState(0);

  const { data: metricsData } = useQuery<{ code: number; data: MetricsData }>({
    queryKey: ['metrics'],
    queryFn: async () => {
      const { data } = await api.get('/v1/metrics');
      return data;
    },
    refetchInterval: 30000,
    retry: 1,
  });

  const { data: trafficData, isLoading: trafficLoading, error: trafficError } = useQuery<{ code: number; data: TrafficData }>({
    queryKey: ['traffic', refreshKey],
    queryFn: async () => {
      const { data } = await api.get('/v1/traffic');
      return data;
    },
    retry: 1,
  });

  const metrics = metricsData?.data;

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

      {/* Traffic Stats Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-6">
        <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
          <div className="flex items-center justify-between mb-3">
            <span className="text-sm text-[var(--text-secondary)]">{t('dashboard.totalIngress')}</span>
            <ArrowDownCircle size={20} className="text-blue-500" />
          </div>
          <div className="text-2xl font-semibold text-blue-500">
            {trafficData?.data?.total_ingress_bytes
              ? formatBytes(trafficData.data.total_ingress_bytes)
              : '-'}
          </div>
        </div>
        <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
          <div className="flex items-center justify-between mb-3">
            <span className="text-sm text-[var(--text-secondary)]">{t('dashboard.totalEgress')}</span>
            <ArrowUpCircle size={20} className="text-emerald-500" />
          </div>
          <div className="text-2xl font-semibold text-emerald-500">
            {trafficData?.data?.total_egress_bytes
              ? formatBytes(trafficData.data.total_egress_bytes)
              : '-'}
          </div>
        </div>
      </div>

      {/* Traffic Chart */}
      <div className="mt-6">
        <TrafficChart data={trafficData?.data} loading={trafficLoading} error={trafficError} />
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}
