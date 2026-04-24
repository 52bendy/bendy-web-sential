import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { toast } from 'sonner';
import { ArrowLeft, Play, Loader2 } from 'lucide-react';
import api from '@/lib/api';
import type { Route, Domain } from '@/types';

export default function RouteTest() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [testUrl, setTestUrl] = useState('');
  const [method, setMethod] = useState('GET');
  const [requestHeaders, setRequestHeaders] = useState('');
  const [requestBody, setRequestBody] = useState('');
  const [response, setResponse] = useState<{
    status: number;
    statusText: string;
    headers: Record<string, string>;
    body: string;
    duration: number;
    error?: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);

  const { data: routesData } = useQuery<{ code: number; data: Route[] }>({
    queryKey: ['routes'],
    queryFn: async () => {
      const { data } = await api.get('/v1/routes');
      return data;
    },
  });

  const { data: domainsData } = useQuery<{ code: number; data: Domain[] }>({
    queryKey: ['domains'],
    queryFn: async () => {
      const { data } = await api.get('/v1/domains');
      return data;
    },
  });

  const route = routesData?.data?.find((r) => r.id === Number(id));
  const domain = domainsData?.data?.find((d) => d.id === route?.domain_id);

  const handleTest = async () => {
    if (!testUrl.trim()) {
      toast.error(t('routeTest.enterUrl'));
      return;
    }

    setLoading(true);
    setResponse(null);
    const start = Date.now();

    try {
      // Parse headers
      const headersObj: Record<string, string> = {};
      if (requestHeaders.trim()) {
        requestHeaders.split('\n').forEach((line) => {
          const colonIdx = line.indexOf(':');
          if (colonIdx > 0) {
            const key = line.slice(0, colonIdx).trim();
            const val = line.slice(colonIdx + 1).trim();
            if (key) headersObj[key] = val;
          }
        });
      }

      const fetchOptions: RequestInit = {
        method,
        headers: headersObj,
      };

      if (['POST', 'PUT', 'PATCH'].includes(method) && requestBody.trim()) {
        fetchOptions.body = requestBody;
      }

      const res = await fetch(testUrl, fetchOptions);
      const duration = Date.now() - start;

      const resHeaders: Record<string, string> = {};
      res.headers.forEach((val, key) => {
        resHeaders[key] = val;
      });

      let bodyText = '';
      try {
        const contentType = res.headers.get('content-type') || '';
        if (contentType.includes('json')) {
          const json = await res.json();
          bodyText = JSON.stringify(json, null, 2);
        } else {
          bodyText = await res.text();
        }
      } catch {
        bodyText = '(could not parse response body)';
      }

      setResponse({
        status: res.status,
        statusText: res.statusText,
        headers: resHeaders,
        body: bodyText,
        duration,
      });
    } catch (err) {
      const duration = Date.now() - start;
      setResponse({
        status: 0,
        statusText: 'Network Error',
        headers: {},
        body: '',
        duration,
        error: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setLoading(false);
    }
  };

  const statusColor = (status: number) => {
    if (status === 0) return 'text-red-500';
    if (status < 300) return 'text-green-500';
    if (status < 400) return 'text-blue-500';
    if (status < 500) return 'text-yellow-500';
    return 'text-red-500';
  };

  return (
    <div>
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => navigate('/routes')}
          className="p-2 rounded hover:bg-[var(--bg-secondary)] border border-[var(--border-default)]"
        >
          <ArrowLeft size={18} />
        </button>
        <h1 className="text-2xl font-semibold">{t('routeTest.title')}</h1>
      </div>

      {/* Route Info */}
      {route ? (
        <div className="mb-6 p-4 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4 text-sm">
            <div>
              <div className="text-[var(--text-muted)] mb-1">{t('routes.pathPattern')}</div>
              <div className="font-mono font-medium">{route.path_pattern}</div>
            </div>
            <div>
              <div className="text-[var(--text-muted)] mb-1">{t('routes.action')}</div>
              <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                route.action === 'proxy' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300' :
                route.action === 'redirect' ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' :
                'bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300'
              }`}>{route.action}</span>
            </div>
            <div>
              <div className="text-[var(--text-muted)] mb-1">{t('routes.target')}</div>
              <div className="font-mono text-xs truncate">{route.target}</div>
            </div>
            <div>
              <div className="text-[var(--text-muted)] mb-1">{t('routes.domain')}</div>
              <div className="font-medium">{domain?.domain ?? '-'}</div>
            </div>
            <div>
              <div className="text-[var(--text-muted)] mb-1">{t('routes.priority')}</div>
              <div>{route.priority}</div>
            </div>
          </div>
        </div>
      ) : (
        <div className="mb-6 p-4 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)] text-[var(--text-muted)]">
          Route not found
        </div>
      )}

      {/* Test Form */}
      <div className="p-5 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
        <h2 className="text-sm font-medium mb-4">{t('routeTest.testRequest')}</h2>
        <div className="flex gap-2 mb-4">
          <select
            value={method}
            onChange={(e) => setMethod(e.target.value)}
            className="px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-sm font-mono"
          >
            {['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'].map((m) => (
              <option key={m} value={m}>{m}</option>
            ))}
          </select>
          <input
            type="text"
            value={testUrl}
            onChange={(e) => setTestUrl(e.target.value)}
            placeholder="https://api.example.com/your/path"
            className="flex-1 px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-sm font-mono"
          />
          <button
            onClick={handleTest}
            disabled={loading}
            className="px-4 py-2 rounded bg-black text-white dark:bg-white dark:text-black text-sm hover:opacity-90 disabled:opacity-50 flex items-center gap-2"
          >
            {loading ? <Loader2 size={16} className="animate-spin" /> : <Play size={16} />}
            {t('routeTest.send')}
          </button>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm mb-1">{t('routeTest.requestHeaders')}</label>
            <textarea
              value={requestHeaders}
              onChange={(e) => setRequestHeaders(e.target.value)}
              placeholder="Content-Type: application/json&#10;Authorization: Bearer token"
              className="w-full h-24 px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-sm font-mono resize-none"
            />
          </div>
          <div>
            <label className="block text-sm mb-1">{t('routeTest.requestBody')}</label>
            <textarea
              value={requestBody}
              onChange={(e) => setRequestBody(e.target.value)}
              placeholder='{"key": "value"}'
              className="w-full h-24 px-3 py-2 border border-[var(--border-default)] rounded bg-[var(--bg-primary)] text-sm font-mono resize-none"
            />
          </div>
        </div>
      </div>

      {/* Response */}
      {response && (
        <div className="mt-6">
          <h2 className="text-sm font-medium mb-4">{t('routeTest.response')}</h2>
          <div className="p-4 border border-[var(--border-default)] rounded-lg bg-[var(--bg-secondary)]">
            {response.error ? (
              <div className="text-red-500 font-mono text-sm">{response.error}</div>
            ) : (
              <>
                <div className="flex items-center gap-4 mb-4">
                  <span className={`font-mono font-bold text-lg ${statusColor(response.status)}`}>
                    {response.status} {response.statusText}
                  </span>
                  <span className="text-sm text-[var(--text-muted)]">
                    {response.duration}ms
                  </span>
                </div>
                <div className="mb-4">
                  <div className="text-sm font-medium mb-2">{t('routeTest.responseHeaders')}</div>
                  <div className="font-mono text-xs space-y-1 max-h-40 overflow-y-auto">
                    {Object.entries(response.headers).map(([k, v]) => (
                      <div key={k}>
                        <span className="text-blue-600">{k}</span>
                        <span className="text-[var(--text-muted)]">: </span>
                        <span>{v}</span>
                      </div>
                    ))}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium mb-2">{t('routeTest.responseBody')}</div>
                  <pre className="font-mono text-xs bg-[var(--bg-primary)] border border-[var(--border-default)] rounded p-3 overflow-x-auto max-h-96">
                    {response.body}
                  </pre>
                </div>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
