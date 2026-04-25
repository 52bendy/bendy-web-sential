import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { Plus, Pencil, Trash2, RefreshCw } from 'lucide-react';
import { getRewrites, createRewrite, deleteRewrite } from '@/lib/api';
import type { RewriteRule } from '@/types';

const RULE_TYPES = [
  { value: 'header_add', label: 'Add Header' },
  { value: 'header_replace', label: 'Replace Header' },
  { value: 'header_remove', label: 'Remove Header' },
] as const;

export default function Rewrites() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [editRule, setEditRule] = useState<RewriteRule | null>(null);
  const [form, setForm] = useState({
    name: '',
    rule_type: 'header_add' as 'header_add' | 'header_replace' | 'header_remove',
    pattern: '',
    replacement: '',
    enabled: true,
  });

  const { data: rules, isLoading, refetch } = useQuery({
    queryKey: ['rewrites'],
    queryFn: getRewrites,
  });

  const createMut = useMutation({
    mutationFn: createRewrite,
    onSuccess: () => {
      toast.success(t('common.save'));
      qc.invalidateQueries({ queryKey: ['rewrites'] });
      closeModal();
    },
    onError: () => {
      toast.error('Failed to create rewrite rule');
    },
  });

  const deleteMut = useMutation({
    mutationFn: deleteRewrite,
    onSuccess: () => {
      toast.success(t('common.delete'));
      qc.invalidateQueries({ queryKey: ['rewrites'] });
    },
    onError: () => {
      toast.error('Failed to delete rewrite rule');
    },
  });

  const openCreate = () => {
    setEditRule(null);
    setForm({ name: '', rule_type: 'header_add', pattern: '', replacement: '', enabled: true });
    setModalOpen(true);
  };

  const openEdit = (r: RewriteRule) => {
    setEditRule(r);
    setForm({
      name: r.name,
      rule_type: r.rule_type,
      pattern: r.pattern,
      replacement: r.replacement,
      enabled: r.enabled,
    });
    setModalOpen(true);
  };

  const closeModal = () => {
    setModalOpen(false);
    setEditRule(null);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMut.mutate(form);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-semibold">{t('rewrites.title') || 'Rewrite Rules'}</h1>
          <button onClick={() => refetch()} className="p-2 rounded hover:bg-[var(--bg-secondary)]" title="Refresh">
            <RefreshCw size={16} />
          </button>
        </div>
        <button onClick={openCreate} className="flex items-center gap-2 px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm hover:opacity-90 transition-opacity">
          <Plus size={16} />
          {t('rewrites.addRule') || 'Add Rule'}
        </button>
      </div>

      <div className="border border-[var(--border-default)] rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-[var(--bg-secondary)]">
            <tr>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">Name</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">Type</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">Pattern</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">Replacement</th>
              <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">Status</th>
              <th className="text-right px-4 py-3 font-medium text-[var(--text-secondary)]">{t('common.actions')}</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr><td colSpan={6} className="text-center py-8 text-[var(--text-muted)]">{t('common.loading')}</td></tr>
            ) : rules?.length === 0 ? (
              <tr><td colSpan={6} className="text-center py-8 text-[var(--text-muted)]">{t('common.noData')}</td></tr>
            ) : (
              rules?.map((r) => (
                <tr key={r.id} className="border-t border-[var(--border-default)]">
                  <td className="px-4 py-3 font-medium">{r.name}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                      r.rule_type === 'header_add' ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' :
                      r.rule_type === 'header_replace' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300' :
                      'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'
                    }`}>{r.rule_type.replace('header_', '')}</span>
                  </td>
                  <td className="px-4 py-3 font-mono text-xs">{r.pattern}</td>
                  <td className="px-4 py-3 font-mono text-xs text-[var(--text-secondary)]">
                    {r.rule_type === 'header_remove' ? '-' : r.replacement}
                  </td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-0.5 rounded text-xs ${
                      r.enabled ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-gray-100 text-gray-500'
                    }`}>{r.enabled ? 'Enabled' : 'Disabled'}</span>
                  </td>
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
            <h2 className="text-lg font-semibold mb-4">{editRule ? 'Edit Rule' : t('rewrites.addRule') || 'Add Rule'}</h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm mb-1">Name</label>
                <input type="text" value={form.name} onChange={e => setForm({...form, name: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="e.g., Add X-Custom-Header" required />
              </div>
              <div>
                <label className="block text-sm mb-1">Type</label>
                <select value={form.rule_type} onChange={e => setForm({...form, rule_type: e.target.value as typeof form.rule_type})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]">
                  {RULE_TYPES.map(rt => <option key={rt.value} value={rt.value}>{rt.label}</option>)}
                </select>
              </div>
              <div>
                <label className="block text-sm mb-1">Header Name (Pattern)</label>
                <input type="text" value={form.pattern} onChange={e => setForm({...form, pattern: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="e.g., X-Forwarded-For or X-Api-Version" required />
              </div>
              {form.rule_type !== 'header_remove' && (
                <div>
                  <label className="block text-sm mb-1">Header Value (Replacement)</label>
                  <input type="text" value={form.replacement} onChange={e => setForm({...form, replacement: e.target.value})} className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]" placeholder="e.g., v2 or ${original}" />
                </div>
              )}
              <label className="flex items-center gap-2">
                <input type="checkbox" checked={form.enabled} onChange={e => setForm({...form, enabled: e.target.checked})} />
                <span className="text-sm">Enabled</span>
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