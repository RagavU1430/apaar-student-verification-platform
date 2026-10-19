import { useEffect, useState } from 'react';
import { api, type AuditEvent } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { History, Filter, Clock } from 'lucide-react';

export default function AuditPage() {
  const [logs, setLogs] = useState<AuditEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [actionFilter, setActionFilter] = useState('');

  const fetchLogs = () => {
    setLoading(true);
    api
      .auditLogs()
      .then((data) => setLogs(data))
      .catch((err) => console.error(err))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchLogs();
  }, []);

  const filtered = actionFilter
    ? logs.filter((log) => log.action.toLowerCase().includes(actionFilter.toLowerCase()))
    : logs;

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
            <History className="w-7 h-7 text-indigo-600" />
            Attributable Audit Trail
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            Tamper-evident, append-only log of all security, data modification, review, and verification actions.
          </p>
        </div>
        <button onClick={fetchLogs} className="btn-secondary self-start md:self-auto">
          Refresh Trail
        </button>
      </div>

      {/* Filter toolbar */}
      <div className="panel p-4 bg-white border border-slate-200">
        <div className="flex items-center gap-3">
          <Filter className="w-4 h-4 text-slate-400" />
          <span className="text-xs uppercase font-semibold text-slate-600">Action Filter:</span>
          <select
            value={actionFilter}
            onChange={(e) => setActionFilter(e.target.value)}
            className="px-3 py-1.5 text-sm border border-slate-200 rounded-lg focus:ring-2 focus:ring-blue-500 bg-white"
          >
            <option value="">All Actions</option>
            <option value="batch.upload">batch.upload</option>
            <option value="batch.commit">batch.commit</option>
            <option value="issue.resolve">issue.resolve</option>
            <option value="verification.request">verification.request</option>
            <option value="auth.login">auth.login</option>
          </select>
          <span className="text-xs text-slate-500 ml-auto font-mono">
            Showing {filtered.length} events
          </span>
        </div>
      </div>

      {/* Audit Log Table */}
      <div className="panel p-0 overflow-hidden">
        {loading ? (
          <div className="p-8 text-center text-slate-500">Loading audit history...</div>
        ) : filtered.length === 0 ? (
          <div className="p-12 text-center text-slate-500">
            <Clock className="w-10 h-10 text-slate-300 mx-auto mb-2" />
            <p>No audit events recorded yet.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-slate-50 border-b border-slate-200 text-xs font-semibold text-slate-600 uppercase tracking-wider">
                  <th className="py-3 px-4">Timestamp (UTC)</th>
                  <th className="py-3 px-4">Action</th>
                  <th className="py-3 px-4">Actor Role</th>
                  <th className="py-3 px-4">Target Entity</th>
                  <th className="py-3 px-4">Outcome</th>
                  <th className="py-3 px-4">Summary</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {filtered.map((event) => (
                  <tr key={event.id} className="hover:bg-slate-50/80 transition-colors">
                    <td className="py-3 px-4 font-mono text-xs text-slate-500 whitespace-nowrap">
                      {new Date(event.created_at).toLocaleString()}
                    </td>
                    <td className="py-3 px-4">
                      <span className="font-mono text-xs px-2 py-0.5 rounded bg-slate-100 text-slate-800 font-semibold">
                        {event.action}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-xs font-medium text-slate-700">
                      {event.actor_role || 'SYSTEM'}
                    </td>
                    <td className="py-3 px-4 font-mono text-xs text-slate-600">
                      {event.entity_type}
                    </td>
                    <td className="py-3 px-4">
                      <StatusBadge label={event.outcome} />
                    </td>
                    <td className="py-3 px-4 text-xs text-slate-700 max-w-md">
                      {event.summary || '-'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
