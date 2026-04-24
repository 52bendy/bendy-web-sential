import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';
import { format } from 'date-fns';
import type { TrafficData } from '@/types';

interface TrafficChartProps {
  data: TrafficData | undefined;
  loading?: boolean;
  error?: unknown;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

function formatTime(timestamp: string): string {
  try {
    return format(new Date(timestamp), 'HH:mm:ss');
  } catch {
    return timestamp;
  }
}

export default function TrafficChart({ data, loading, error }: TrafficChartProps) {
  const { t } = useTranslation();

  const chartData = useMemo(() => {
    if (!data) return [];

    const ingressMap = new Map(data.ingress.map((p) => [p.time, p.bytes]));
    const egressMap = new Map(data.egress.map((p) => [p.time, p.bytes]));

    const allTimes = Array.from(
      new Set([...data.ingress.map((p) => p.time), ...data.egress.map((p) => p.time)])
    ).sort();

    return allTimes.map((time) => ({
      time,
      timeLabel: formatTime(time),
      ingress: ingressMap.get(time) || 0,
      egress: egressMap.get(time) || 0,
    }));
  }, [data]);

  if (loading) {
    return (
      <div className="h-80 flex items-center justify-center">
        <div className="text-[var(--text-muted)]">{t('common.loading')}</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-80 flex items-center justify-center">
        <div className="text-[var(--text-muted)] text-red-500">Failed to load traffic data. Please refresh.</div>
      </div>
    );
  }

  if (!data || chartData.length === 0) {
    return (
      <div className="h-80 flex items-center justify-center">
        <div className="text-[var(--text-muted)]">{t('dashboard.noTrafficData')}</div>
      </div>
    );
  }

  return (
    <div className="bg-[var(--bg-secondary)] border border-[var(--border-default)] rounded-lg p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-base font-medium">{t('dashboard.trafficChart')}</h3>
        <div className="flex gap-4 text-xs">
          <div className="flex items-center gap-1.5">
            <div className="w-3 h-3 rounded-full bg-blue-500" />
            <span>{t('dashboard.ingress')}</span>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-3 h-3 rounded-full bg-emerald-500" />
            <span>{t('dashboard.egress')}</span>
          </div>
        </div>
      </div>

      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="colorIngress" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="colorEgress" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--border-default)" />
            <XAxis
              dataKey="timeLabel"
              tick={{ fontSize: 11 }}
              stroke="var(--text-muted)"
            />
            <YAxis
              tick={{ fontSize: 11 }}
              stroke="var(--text-muted)"
              tickFormatter={(value) => formatBytes(value)}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'var(--bg-secondary)',
                border: '1px solid var(--border-default)',
                borderRadius: '0.5rem',
              }}
              labelStyle={{ color: 'var(--text-primary)' }}
              formatter={(value: number) => [formatBytes(value), '']}
            />
            <Legend />
            <Area
              type="monotone"
              dataKey="ingress"
              stroke="#3b82f6"
              fillOpacity={1}
              fill="url(#colorIngress)"
              name={t('dashboard.ingress')}
            />
            <Area
              type="monotone"
              dataKey="egress"
              stroke="#10b981"
              fillOpacity={1}
              fill="url(#colorEgress)"
              name={t('dashboard.egress')}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 gap-4 mt-4">
        <div className="text-center">
          <div className="text-xs text-[var(--text-muted)]">{t('dashboard.totalIngress')}</div>
          <div className="text-lg font-semibold text-blue-500">
            {formatBytes(data.total_ingress_bytes)}
          </div>
        </div>
        <div className="text-center">
          <div className="text-xs text-[var(--text-muted)]">{t('dashboard.totalEgress')}</div>
          <div className="text-lg font-semibold text-emerald-500">
            {formatBytes(data.total_egress_bytes)}
          </div>
        </div>
      </div>
    </div>
  );
}
