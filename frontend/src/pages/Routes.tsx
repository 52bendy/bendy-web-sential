import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { Plus, Pencil, Trash2 } from 'lucide-react';
import api from '@/lib/api';
import type { Route, Domain } from '@/types';

export default function Routes() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [editRoute, setEditRoute] = useState<Route | null>(null);
  const [form, setForm] = useState({
    domain_id: 0,
    path_pattern: '',
    action: 'proxy',
    target: '',
    description: '',
    priority: 0,
    active: true,
  });

  const { data: domains } = useQuery<{ code: number; data: Domain[] }>({
    queryKey: ['domains'],
    queryFn: async () => {
      const { data } = await api.get('/v1/domains');
      return data;
    },
  });

  const { data, isLoading } = useQuery<{ code: number; data: Route[] }>({
    queryKey: ['routes'],
    queryFn: async () => {
      const { data } = await api.get('/v1/routes');
      return data;
    },
  });

  const createMut = useMutation({
    mutationFn: (payload: typeof form) => api.post('/v1/routes', payload),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.save'));
        qc.invalidateQueries({ queryKey: ['routes'] });
        closeModal();
      }
    },
  });

  const updateMut = useMutation({
    mutationFn: ({ id, payload }: { id: number; payload: typeof form }) =>
      api.put(`/v1/routes/${id}`, payload),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.save'));
        qc.invalidateQueries({ queryKey: ['routes'] });
        closeModal();
      }
    },
  });

  const deleteMut = useMutation({
    mutationFn: (id: number) => api.delete(`/v1/routes/${id}`),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.delete'));
        qc.invalidateQueries({ queryKey: ['routes'] });
      }
    },
  });

  const openCreate = () => {
    setEditRoute(null);
    setForm({ domain_id: domains?.data?.[0]?.id || 0, path_pattern: '', action: 'proxy', target: '', description: '', priority: 0, active: true });
    setModalOpen(true);
  };

  const openEdit = (r: Route) => {
    setEditRoute(r);
    setForm({
      domain_id: r.domain_id,
      path_pattern: r.path_pattern,
      action: r.action,
      target: r.target,
      description: r.description || '',
      priority: r.priority,
      active: r.active,
    });
    setModalOpen(true);
  };

  const closeModal = () => {
    setModalOpen(false);
    setEditRoute(null);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editRoute) {
      updateMut.mutate({ id: editRoute.id, payload: form });
    } else {
      createMut.mutate(form);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold">{t('routes.title')}</h1>
        <button onClick={openCreate} className="flex items-center gap-2 px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm hover:opacity-90 transition-opacity">
          <Plus size={16} />
          {t('routes.addRoute')}
        </button>
      </div>

      <div className="border border-[var(--border-default)] rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-[var(--bg-secondary)]">
            <tr>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.pathPattern')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.action')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.target')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.priority')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.domain')}</th>
              <th className="text-right px-4 py-3 font-medium text-[var(--text-secondary)]">{t('common.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr><td colSpan={6} className="text-center py-8 text-[var(--text-muted)]">{t('common.loading')}</td></tr>
            ) : data?.data?.length === 0 ? (
              <tr><td colSpan={6} className="text-center py-8 text-[var(--text-muted)]">{t('common.noData')}</td></tr>
            ) : (
              data?.data?.map((r) => (
                <tr key={r.id} className="border-t border-[var(--border-default)]">
                  <td className="px-4 py-3 font-mono text-xs">{r.path_pattern}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                      r.action === 'proxy' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300' :
                      r.action === 'redirect' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' :
                      'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
                    }`}>{r.action}</span>
                  </td>
                  <td className="px-4 py-3 text-[var(--text-secondary)] font-mono text-xs">{r.target}</td>
                  <td className="px-4 py-3 text-[var(--text-secondary)]">{r.priority}</td>
                  <td className="px-4 py-3 text-[var(--text-secondary)]">{domains?.data?.find(d => d.id === r.domain_id)?.domain ?? '-'}</td>
                  <td className="px-4 py-3 text-right">
                    <button onClick={() => openEdit(r)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)]"><Pencil size={14} /></button>
                    <button onClick={() => deleteMut.mutate(r.id)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-red-500 ml-1"><Trash2 size={14} /></button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="w-full max-w-lg p-6 bg-[var(--bg-primary)] border border-[var(--border-default)] rounded-lg">
            <h2 className="text-lg font-semibold mb-4">{editRoute ? t('routes.editRoute') : t('routes.addRoute')}</h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm mb-1">{t('routes.domain')}</label>
                <select value={form.domain_id} onChange={e => setForm({...form, domain_id: +e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]">
                  {domains?.data?.map(d => <option key={d.id} value={d.id}>{d.domain}</option>)}
                </select>
              </div>
              <div>
                <label className="block text-sm mb-1">{t('routes.pathPattern')}</label>
                <input type="text" value={form.path_pattern} onChange={e => setForm({...form, path_pattern: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="/api/*" required />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm mb-1">{t('routes.action')}</label>
                  <select value={form.action} onChange={e => setForm({...form, action: e.target.value as typeof form.action})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]">
                    <option value="proxy">{t('routes.proxy')}</option>
                    <option value="redirect">{t('routes.redirect')}</option>
                    <option value="static">{t('routes.static')}</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm mb-1">{t('routes.priority')}</label>
                  <input type="number" value={form.priority} onChange={e => setForm({...form, priority: +e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" />
                </div>
              </div>
              <div>
                <label className="block text-sm mb-1">{t('routes.target')}</label>
                <input type="text" value={form.target} onChange={e => setForm({...form, target: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="https://api.example.com" required />
              </div>
              <div>
                <label className="block text-sm mb-1">{t('domains.description')}</label>
                <input type="text" value={form.description} onChange={e => setForm({...form, description: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" />
              </div>
              <label className="flex items-center gap-2">
                <input type="checkbox" checked={form.active} onChange={e => setForm({...form, active: e.target.checked})} />
                <span className="text-sm">{t('domains.active')}</span>
              </label>
              <div className="flex gap-3 pt-2">
                <button type="submit" className="px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm">{t('common.save')}</button>
                <button type="button" onClick={closeModal} className="px-4 py-2 rounded border border-[var(--border-default)] text-sm">{t('common.cancel')}</button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
