import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { Plus, Pencil, Trash2, Globe, Cloud } from 'lucide-react';
import api from '@/lib/api';
import type { Domain } from '@/types';

type Tab = 'domains' | 'dns';

interface CfDnsRecord {
  id: string;
  type: string;
  name: string;
  content: string;
  proxied?: boolean;
  ttl?: number;
  priority?: number;
}

interface DnsFormData {
  type: string;
  name: string;
  content: string;
  proxied: boolean;
  ttl: number;
  priority: number;
}

export default function Domains() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [tab, setTab] = useState<Tab>('domains');
  const [modalOpen, setModalOpen] = useState(false);
  const [editDomain, setEditDomain] = useState<Domain | null>(null);
  const [dnsModalOpen, setDnsModalOpen] = useState(false);
  const [editDns, setEditDns] = useState<CfDnsRecord | null>(null);
  const [dnsForm, setDnsForm] = useState<DnsFormData>({
    type: 'A',
    name: '',
    content: '',
    proxied: true,
    ttl: 1,
    priority: 0,
  });

  const [form, setForm] = useState({ domain: '', description: '', hosting_service: '', active: true });

  const { data, isLoading } = useQuery<{ code: number; data: Domain[] }>({
    queryKey: ['domains'],
    queryFn: async () => {
      const { data } = await api.get('/v1/domains');
      return data;
    },
  });

  const {
    data: dnsData,
    isLoading: dnsLoading,
    isError: dnsError,
  } = useQuery<{ code: number; data: CfDnsRecord[] }>({
    queryKey: ['cloudflare-dns'],
    queryFn: async () => {
      const { data } = await api.get('/v1/cloudflare/dns');
      return data;
    },
    enabled: tab === 'dns',
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

  const createDnsMut = useMutation({
    mutationFn: (payload: DnsFormData) => api.post('/v1/cloudflare/dns', payload),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.save'));
        qc.invalidateQueries({ queryKey: ['cloudflare-dns'] });
        closeDnsModal();
      }
    },
    onError: (err: any) => {
      toast.error(err?.response?.data?.message || t('domains.dnsError'));
    },
  });

  const updateDnsMut = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Partial<DnsFormData> }) =>
      api.put(`/v1/cloudflare/dns/${id}`, payload),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.save'));
        qc.invalidateQueries({ queryKey: ['cloudflare-dns'] });
        closeDnsModal();
      }
    },
    onError: (err: any) => {
      toast.error(err?.response?.data?.message || t('domains.dnsError'));
    },
  });

  const deleteDnsMut = useMutation({
    mutationFn: (id: string) => api.delete(`/v1/cloudflare/dns/${id}`),
    onSuccess: (res) => {
      if (res.data.code === 0) {
        toast.success(t('common.delete'));
        qc.invalidateQueries({ queryKey: ['cloudflare-dns'] });
      }
    },
    onError: (err: any) => {
      toast.error(err?.response?.data?.message || t('domains.dnsError'));
    },
  });

  const openCreate = () => {
    setEditDomain(null);
    setForm({ domain: '', description: '', hosting_service: '', active: true });
    setModalOpen(true);
  };

  const openEdit = (d: Domain) => {
    setEditDomain(d);
    setForm({ domain: d.domain, description: d.description || '', hosting_service: d.hosting_service || '', active: d.active });
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

  const openDnsCreate = () => {
    setEditDns(null);
    setDnsForm({ type: 'A', name: '', content: '', proxied: true, ttl: 1, priority: 0 });
    setDnsModalOpen(true);
  };

  const openDnsEdit = (record: CfDnsRecord) => {
    setEditDns(record);
    setDnsForm({
      type: record.type,
      name: record.name,
      content: record.content,
      proxied: record.proxied ?? true,
      ttl: record.ttl ?? 1,
      priority: record.priority ?? 0,
    });
    setDnsModalOpen(true);
  };

  const closeDnsModal = () => {
    setDnsModalOpen(false);
    setEditDns(null);
  };

  const handleDnsSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editDns) {
      updateDnsMut.mutate({
        id: editDns.id,
        payload: { name: dnsForm.name, content: dnsForm.content, proxied: dnsForm.proxied },
      });
    } else {
      createDnsMut.mutate(dnsForm);
    }
  };

  const recordTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      A: 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300',
      AAAA: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900 dark:text-indigo-300',
      CNAME: 'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300',
      MX: 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300',
      TXT: 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300',
      NS: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300',
      SOA: 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300',
    };
    return colors[type] || 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300';
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-semibold">{t('domains.title')}</h1>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 border-b border-[var(--border-default)]">
        <button
          onClick={() => setTab('domains')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            tab === 'domains'
              ? 'border-blue-500 text-blue-500'
              : 'border-transparent text-[var(--text-muted)] hover:text-[var(--text-primary)]'
          }`}
        >
          {t('domains.domains')}
        </button>
        <button
          onClick={() => setTab('dns')}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors flex items-center gap-1.5 ${
            tab === 'dns'
              ? 'border-blue-500 text-blue-500'
              : 'border-transparent text-[var(--text-muted)] hover:text-[var(--text-primary)]'
          }`}
        >
          <Globe size={14} />
          {t('domains.cloudflareDns')}
        </button>
      </div>

      {/* Domains Tab */}
      {tab === 'domains' && (
        <>
          <div className="flex items-center justify-end mb-4">
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
                        {d.hosting_service && (
                          <button
                            onClick={() => setTab('dns')}
                            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] mr-1"
                            title="DNS Management"
                          >
                            <Cloud size={14} className="text-orange-500" />
                          </button>
                        )}
                        {d.hosting_service === 'cloudflare' && !dnsData?.data && (
                          <button
                            onClick={() => setTab('dns')}
                            className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] mr-1"
                            title="DNS Not Configured"
                          >
                            <Cloud size={14} className="text-gray-400" />
                          </button>
                        )}
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
        </>
      )}

      {/* CloudFlare DNS Tab */}
      {tab === 'dns' && (
        <>
          <div className="flex items-center justify-end mb-4">
            <button
              onClick={openDnsCreate}
              className="flex items-center gap-2 px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm hover:opacity-90 transition-opacity"
            >
              <Plus size={16} />
              {t('domains.addDnsRecord')}
            </button>
          </div>

          {dnsError ? (
            <div className="p-4 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)] text-[var(--text-muted)] text-sm">
              {t('domains.dnsNotConfigured')}
            </div>
          ) : (
            <div className="border border-[var(--border-default)] rounded-lg overflow-hidden">
              <table className="w-full text-sm">
                <thead className="bg-[var(--bg-secondary)]">
                  <tr>
                    <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.dnsType')}</th>
                    <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.dnsName')}</th>
                    <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.dnsContent')}</th>
                    <th className="text-left px-4 py-3 font-medium text-[var(--text-secondary)]">{t('domains.dnsProxied')}</th>
                    <th className="text-right px-4 py-3 font-medium text-[var(--text-secondary)]">{t('common.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {dnsLoading ? (
                    <tr><td colSpan={5} className="text-center py-8 text-[var(--text-muted)]">{t('common.loading')}</td></tr>
                  ) : dnsData?.data?.length === 0 ? (
                    <tr><td colSpan={5} className="text-center py-8 text-[var(--text-muted)]">{t('common.noData')}</td></tr>
                  ) : (
                    dnsData?.data?.map((r) => (
                      <tr key={r.id} className="border-t border-[var(--border-default)]">
                        <td className="px-4 py-3">
                          <span className={`px-2 py-0.5 rounded text-xs font-medium ${recordTypeColor(r.type)}`}>
                            {r.type}
                          </span>
                        </td>
                        <td className="px-4 py-3 font-mono text-xs">{r.name}</td>
                        <td className="px-4 py-3 font-mono text-xs text-[var(--text-secondary)]">{r.content}</td>
                        <td className="px-4 py-3">
                          {r.type === 'CNAME' || r.type === 'A' || r.type === 'AAAA' ? (
                            <span className={`px-2 py-0.5 rounded text-xs ${r.proxied ? 'bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300' : 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'}`}>
                              {r.proxied ? 'Proxied' : 'DNS Only'}
                            </span>
                          ) : '-'}
                        </td>
                        <td className="px-4 py-3 text-right">
                          <button onClick={() => openDnsEdit(r)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)]">
                            <Pencil size={14} />
                          </button>
                          <button onClick={() => deleteDnsMut.mutate(r.id)} className="p-1.5 rounded hover:bg-[var(--bg-tertiary)] text-red-500 ml-1">
                            <Trash2 size={14} />
                          </button>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          )}
        </>
      )}

      {/* Domain Modal */}
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
              <div>
                <label className="block text-sm mb-1">{t('domains.hostingService')}</label>
                <select
                  value={form.hosting_service}
                  onChange={(e) => setForm({ ...form, hosting_service: e.target.value })}
                  className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]"
                >
                  <option value="">{t('domains.noHosting')}</option>
                  <option value="cloudflare">CloudFlare</option>
                </select>
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

      {/* DNS Record Modal */}
      {dnsModalOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="w-full max-w-md p-6 bg-[var(--bg-primary)] border border-[var(--border-default)] rounded-lg">
            <h2 className="text-lg font-semibold mb-4">{editDns ? t('domains.editDnsRecord') : t('domains.addDnsRecord')}</h2>
            <form onSubmit={handleDnsSubmit} className="space-y-4">
              <div>
                <label className="block text-sm mb-1">{t('domains.dnsType')}</label>
                <select
                  value={dnsForm.type}
                  onChange={(e) => setDnsForm({ ...dnsForm, type: e.target.value })}
                  className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]"
                  disabled={!!editDns}
                >
                  {['A', 'AAAA', 'CNAME', 'MX', 'TXT', 'NS', 'SOA'].map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm mb-1">{t('domains.dnsName')}</label>
                <input
                  type="text"
                  value={dnsForm.name}
                  onChange={(e) => setDnsForm({ ...dnsForm, name: e.target.value })}
                  className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]"
                  placeholder="example.com"
                  required
                />
              </div>
              <div>
                <label className="block text-sm mb-1">{t('domains.dnsContent')}</label>
                <input
                  type="text"
                  value={dnsForm.content}
                  onChange={(e) => setDnsForm({ ...dnsForm, content: e.target.value })}
                  className="w-full px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-secondary)]"
                  placeholder={dnsForm.type === 'CNAME' ? 'target.example.com' : '1.2.3.4'}
                  required
                />
              </div>
              {(dnsForm.type === 'A' || dnsForm.type === 'AAAA' || dnsForm.type === 'CNAME') && (
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={dnsForm.proxied}
                    onChange={(e) => setDnsForm({ ...dnsForm, proxied: e.target.checked })}
                  />
                  <span className="text-sm">{t('domains.dnsProxiedLabel')}</span>
                </label>
              )}
              <div className="flex gap-3 pt-2">
                <button
                  type="submit"
                  disabled={createDnsMut.isPending || updateDnsMut.isPending}
                  className="px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm"
                >
                  {t('common.save')}
                </button>
                <button type="button" onClick={closeDnsModal} className="px-4 py-2 rounded border border-[var(--border-default)] text-sm">
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
