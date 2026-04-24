import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { Plus, Pencil, Trash2 } from 'lucide-react';
import api from '@/lib/api';
import type { Domain } from '@/types';

export default function Domains() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [editDomain, setEditDomain] = useState<Domain | null>(null);
  const [form, setForm] = useState({ domain: '', description: '', active: true });

  const { data, isLoading } = useQuery<{ code: number; data: Domain[] }>({
    queryKey: ['domains'],
    queryFn: async () => {
      const { data } = await api.get('/v1/domains');
      return data;
    },
  });

  const createMut = useMutation({
    mutationFn: (payload: typeof form) => api.post('/v1/domains', payload),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.save'));
        qc.invalidateQueries({ queryKey: ['domains'] });
        closeModal();
      }
    },
  });

  const updateMut = useMutation({
    mutationFn: ({ id, payload }: { id: number; payload: typeof form }) =>
      api.put(`/v1/domains/${id}`, payload),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.save'));
        qc.invalidateQueries({ queryKey: ['domains'] });
        closeModal();
      }
    },
  });

  const deleteMut = useMutation({
    mutationFn: (id: number) => api.delete(`/v1/domains/${id}`),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.delete'));
        qc.invalidateQueries({ queryKey: ['domains'] });
      }
    },
  });

  const openCreate = () => {
    setEditDomain(null);
    setForm({ domain: '', description: '', active: true });
    setModalOpen(true);
  };

  const openEdit = (d: Domain) => {
    setEditDomain(d);
    setForm({ domain: d.domain, description: d.description || '', active: d.active });
    setModalOpen(true);
  };

  const closeModal = () => {
    setModalOpen(false);
    setEditDomain(null);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editDomain) {
      updateMut.mutate({ id: editDomain.id, payload: form });
    } else {
      createMut.mutate(form);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold">{t('domains.title')}</h1>
        <button
          onClick={openCreate}
          className="flex items-center gap-2 px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm hover:opacity-90 transition-opacity"
        >
          <Plus size={16} />
          {t('domains.addDomain')}
        </button>
      </div>

      <div className="border border-[var(--border-default)] rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-[var(--bg-secondary)]">
            <tr>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.domainName')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.description')}</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.status')}</th>
              <th className="text-right px-4 py-3 font-medium text-[var(--text-secondary)]">{t('common.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr><td colSpan={4} className="text-center py-8 text-[var(--text-muted)]">{t('common.loading')}</td></tr>
            ) : data?.data?.length === 0 ? (
              <tr><td colSpan={4} className="text-center py-8 text-[var(--text-muted)]">{t('common.noData')}</td></tr>
            ) : (
              data?.data?.map((d) => (
                <tr key={d.id} className="border-t border-[var(--border-default)]">
                  <td className="px-4 py-3">{d.domain}</td>
                  <td className="px-4 py-3 text-[var(--text-secondary)]">{d.description || '-'}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-0.5 rounded text-xs ${d.active ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'}`}>
                      {d.active ? t('domains.active') : t('domains.inactive')}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <button onClick={() => openEdit(d)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)]">
                      <Pencil size={14} />
                    </button>
                    <button onClick={() => deleteMut.mutate(d.id)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-red-500 ml-1">
                      <Trash2 size={14} />
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {modalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="w-full max-w-md p-6 bg-[var(--bg-primary)] border border-[var(--border-default)] rounded-lg">
            <h2 className="text-lg font-semibold mb-4">{editDomain ? t('domains.editDomain') : t('domains.addDomain')}</h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm mb-1">{t('domains.domainName')}</label>
                <input
                  type="text"
                  value={form.domain}
                  onChange={(e) => setForm({ ...form, domain: e.target.value })}
                  className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]"
                  required
                />
              </div>
              <div>
                <label className="block text-sm mb-1">{t('domains.description')}</label>
                <input
                  type="text"
                  value={form.description}
                  onChange={(e) => setForm({ ...form, description: e.target.value })}
                  className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]"
                />
              </div>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={form.active}
                  onChange={(e) => setForm({ ...form, active: e.target.checked })}
                />
                <span className="text-sm">{t('domains.active')}</span>
              </label>
              <div className="flex gap-3 pt-2">
                <button type="submit" className="px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm">
                  {t('common.save')}
                </button>
                <button type="button" onClick={closeModal} className="px-4 py-2 rounded border border-[var(--border-default)] text-sm">
                  {t('common.cancel')}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
