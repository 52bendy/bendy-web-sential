import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { Plus, Pencil, Trash2, FlaskConical, ChevronDown, ChevronUp } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import api from '@/lib/api';
import type { Route, Domain } from '@/types';

export default function Routes() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const navigate = useNavigate();
  const [modalOpen, setModalOpen] = useState(false);
  const [editRoute, setEditRoute] = useState<Route | null>(null);
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set());

  const [form, setForm] = useState({
    domain_id: 0,
    path_pattern: '',
    action: 'proxy',
    target: '',
    description: '',
    priority: 0,
    active: true,
    auth_strategy: 'none',
    min_role: null as string | null,
    ratelimit_window: null as number | null,
    ratelimit_limit: null as number | null,
    ratelimit_dimension: 'ip',
    health_check_path: null as string | null,
    health_check_interval_secs: 30,
    transform_rules: null as string | null,
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
    setForm({
      domain_id: domains?.data?.[0]?.id || 0,
      path_pattern: '',
      action: 'proxy',
      target: '',
      description: '',
      priority: 0,
      active: true,
      auth_strategy: 'none',
      min_role: null,
      ratelimit_window: null,
      ratelimit_limit: null,
      ratelimit_dimension: 'ip',
      health_check_path: null,
      health_check_interval_secs: 30,
      transform_rules: null,
    });
    setModalOpen(true);
  };

  const openEdit = (r: Route) => {
    setEditRoute(r);
    setForm({
      domain_id: r.domain_id,
      path_pattern: r.path_pattern,
      action: r.action as 'proxy' | 'redirect' | 'static',
      target: r.target,
      description: r.description || '',
      priority: r.priority,
      active: r.active,
      auth_strategy: r.auth_strategy || 'none',
      min_role: r.min_role || '',
      ratelimit_window: r.ratelimit_window,
      ratelimit_limit: r.ratelimit_limit,
      ratelimit_dimension: r.ratelimit_dimension || 'ip',
      health_check_path: r.health_check_path,
      health_check_interval_secs: r.health_check_interval_secs || 30,
      transform_rules: r.transform_rules,
    });
    setModalOpen(true);
  };

  const closeModal = () => {
    setModalOpen(false);
    setEditRoute(null);
  };

  const toggleRow = (id: number) => {
    setExpandedRows(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editRoute) {
      updateMut.mutate({ id: editRoute.id, payload: form });
    } else {
      createMut.mutate(form);
    }
  };

  const getAuthBadge = (strategy: string) => {
    const colors: Record<string, string> = {
      none: 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400',
      jwt: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
      apikey: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300',
      bearer: 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300',
    };
    return colors[strategy] || colors.none;
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
              <th className="w-8"></th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.pathPattern')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.action')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.target')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('routes.auth')}</th>
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
                <>
                  <tr key={r.id} className="border-t border-[var(--border-default)]">
                    <td className="px-2 py-3">
                      <button onClick={() => toggleRow(r.id)} className="p-1 hover:bg-[var(--bg-tertiary)] rounded">
                        {expandedRows.has(r.id) ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                      </button>
                    </td>
                    <td className="px-4 py-3 font-mono text-xs">{r.path_pattern}</td>
                    <td className="px-4 py-3">
                      <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                        r.action === 'proxy' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300' :
                        r.action === 'redirect' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' :
                        'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
                      }`}>{r.action}</span>
                    </td>
                    <td className="px-4 py-3 text-[var(--text-secondary)] font-mono text-xs max-w-[200px] truncate">{r.target}</td>
                    <td className="px-4 py-3">
                      <span className={`px-2 py-0.5 rounded text-xs font-medium ${getAuthBadge(r.auth_strategy)}`}>
                        {r.auth_strategy || 'none'}
                      </span>
                      {r.ratelimit_limit && (
                        <span className="ml-1 text-xs text-[var(--text-muted)]">{r.ratelimit_limit}/s</span>
                      )}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <button onClick={() => navigate(`/routes/test/${r.id}`)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-blue-500" title={t('routeTest.test')}>
                        <FlaskConical size={14} />
                      </button>
                      <button onClick={() => openEdit(r)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] ml-1"><Pencil size={14} /></button>
                      <button onClick={() => deleteMut.mutate(r.id)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-red-500 ml-1"><Trash2 size={14} /></button>
                    </td>
                  </tr>
                  {expandedRows.has(r.id) && (
                    <tr className="border-t border-[var(--border-default)] bg-[var(--bg-secondary)]">
                      <td colSpan={6} className="px-6 py-4">
                        <div className="grid grid-cols-4 gap-4 text-xs">
                          <div>
                            <span className="text-[var(--text-muted)]">{t('routes.priority')}:</span>
                            <span className="ml-2">{r.priority}</span>
                          </div>
                          <div>
                            <span className="text-[var(--text-muted)]">{t('routes.domain')}:</span>
                            <span className="ml-2">{domains?.data?.find(d => d.id === r.domain_id)?.domain ?? '-'}</span>
                          </div>
                          <div>
                            <span className="text-[var(--text-muted)]">{t('routes.minRole')}:</span>
                            <span className="ml-2">{r.min_role || '-'}</span>
                          </div>
                          <div>
                            <span className="text-[var(--text-muted)]">{t('routes.rateLimit')}:</span>
                            <span className="ml-2">
                              {r.ratelimit_limit ? `${r.ratelimit_limit}/${r.ratelimit_dimension}` : '-'}
                            </span>
                          </div>
                          <div>
                            <span className="text-[var(--text-muted)]">{t('routes.healthCheck')}:</span>
                            <span className="ml-2">{r.health_check_path || '-'}</span>
                          </div>
                          <div>
                            <span className="text-[var(--text-muted)]">Status:</span>
                            <span className={`ml-2 ${r.active ? 'text-green-600' : 'text-red-500'}`}>
                              {r.active ? 'Active' : 'Inactive'}
                            </span>
                          </div>
                          <div className="col-span-2">
                            <span className="text-[var(--text-muted)]">{t('domains.description')}:</span>
                            <span className="ml-2">{r.description || '-'}</span>
                          </div>
                          {r.transform_rules && (
                            <div className="col-span-4">
                              <span className="text-[var(--text-muted)]">{t('routes.transformRules')}:</span>
                              <pre className="mt-1 p-2 bg-[var(--bg-tertiary)] rounded text-xs overflow-x-auto">
                                {r.transform_rules}
                              </pre>
                            </div>
                          )}
                        </div>
                      </td>
                    </tr>
                  )}
                </>
              ))
            )}
          </tbody>
        </table>
      </div>

      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto">
          <div className="w-full max-w-2xl p-6 bg-[var(--bg-primary)] border border-[var(--border-default)] rounded-lg my-8">
            <h2 className="text-lg font-semibold mb-4">{editRoute ? t('routes.editRoute') : t('routes.addRoute')}</h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
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
              </div>

              <div className="grid grid-cols-3 gap-4">
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
                <div>
                  <label className="block text-sm mb-1">{t('routes.auth')}</label>
                  <select value={form.auth_strategy} onChange={e => setForm({...form, auth_strategy: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]">
                    <option value="none">{t('routes.noAuth')}</option>
                    <option value="jwt">JWT</option>
                    <option value="apikey">API Key</option>
                    <option value="bearer">Bearer Token</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-sm mb-1">{t('routes.target')}</label>
                <input type="text" value={form.target} onChange={e => setForm({...form, target: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="https://api.example.com" required />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm mb-1">{t('routes.minRole')}</label>
                  <select value={form.min_role ?? ''} onChange={e => setForm({...form, min_role: e.target.value || null})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]">
                    <option value="">-</option>
                    <option value="user">user</option>
                    <option value="admin">admin</option>
                    <option value="superadmin">superadmin</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm mb-1">{t('routes.description')}</label>
                  <input type="text" value={form.description} onChange={e => setForm({...form, description: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" />
                </div>
              </div>

              {/* Rate Limiting Section */}
              <div className="border-t border-[var(--border-default)] pt-4">
                <h3 className="text-sm font-medium mb-3">{t('routes.rateLimit')} ({t('routes.optional')})</h3>
                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <label className="block text-sm mb-1">{t('routes.rateLimitWindow')} (s)</label>
                    <input type="number" value={form.ratelimit_window || ''} onChange={e => setForm({...form, ratelimit_window: e.target.value ? +e.target.value : null})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="60" />
                  </div>
                  <div>
                    <label className="block text-sm mb-1">{t('routes.rateLimitMax')}</label>
                    <input type="number" value={form.ratelimit_limit || ''} onChange={e => setForm({...form, ratelimit_limit: e.target.value ? +e.target.value : null})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="100" />
                  </div>
                  <div>
                    <label className="block text-sm mb-1">{t('routes.rateLimitDim')}</label>
                    <select value={form.ratelimit_dimension} onChange={e => setForm({...form, ratelimit_dimension: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]">
                      <option value="ip">IP</option>
                      <option value="user">User</option>
                      <option value="global">Global</option>
                    </select>
                  </div>
                </div>
              </div>

              {/* Health Check Section */}
              <div className="border-t border-[var(--border-default)] pt-4">
                <h3 className="text-sm font-medium mb-3">{t('routes.healthCheck')} ({t('routes.optional')})</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm mb-1">{t('routes.healthCheckPath')}</label>
                    <input type="text" value={form.health_check_path || ''} onChange={e => setForm({...form, health_check_path: e.target.value || null})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="/health" />
                  </div>
                  <div>
                    <label className="block text-sm mb-1">{t('routes.healthCheckInterval')} (s)</label>
                    <input type="number" value={form.health_check_interval_secs} onChange={e => setForm({...form, health_check_interval_secs: +e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" />
                  </div>
                </div>
              </div>

              {/* Transform Rules */}
              <div className="border-t border-[var(--border-default)] pt-4">
                <h3 className="text-sm font-medium mb-3">{t('routes.transformRules')} ({t('routes.optional')})</h3>
                <textarea value={form.transform_rules || ''} onChange={e => setForm({...form, transform_rules: e.target.value || null})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)] font-mono text-xs" rows={3} placeholder='{"header_transforms": []}' />
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
