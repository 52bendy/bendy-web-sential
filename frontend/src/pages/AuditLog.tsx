import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { format } from 'date-fns';
import api from '@/lib/api';
import type { AuditLog } from '@/types';

export default function AuditLogPage() {
  const { t } = useTranslation();

  const { data, isLoading } = useQuery<{ code: number; data: AuditLog[] }>({
    queryKey: ['audit-logs'],
    queryFn: async () => {
      const { data } = await api.get('/v1/audit-logs');
      return data;
    },
    refetchInterval: 30000,
  });

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">{t('auditLog.title')}</h1>
      <div className="border border-[var(--border-default)] rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-[var(--bg-secondary)]">
            <tr>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('auditLog.time')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('auditLog.user')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('auditLog.action')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('auditLog.resource')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('auditLog.ipAddress')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('auditLog.details')}</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr><td colSpan={6} className="text-center py-8 text-[var(--text-muted)]">{t('common.loading')}</td></tr>
            ) : data?.data?.length === 0 ? (
              <tr><td colSpan={6} className="text-center py-8 text-[var(--text-muted)]">{t('common.noData')}</td></tr>
            ) : (
              data?.data?.map((log) => (
                <tr key={log.id} className="border-t border-[var(--border-default)]">
                  <td className="px-4 py-3 text-[var(--text-secondary)]">{format(new Date(log.created_at), 'yyyy-MM-dd HH:mm:ss')}</td>
                  <td className="px-4 py-3">{log.username || '-'}</td>
                  <td className="px-4 py-3">
                    <span className="px-2 py-0.5 rounded text-xs bg-[var(--bg-tertiary)]">{log.action}</span>
                  </td>
                  <td className="px-4 py-3 text-[var(--text-secondary)]">{log.resource}</td>
                  <td className="px-4 py-3 text-[var(--text-secondary)] font-mono text-xs">{log.ip_address || '-'}</td>
                  <td className="px-4 py-3 text-[var(--text-secondary)] text-xs max-w-xs truncate">{log.details || '-'}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
