import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { api, type SubmissionBatch, type ValidationIssue } from '../api/client';
import StatusBadge from '../components/StatusBadge';
import { CheckCircle2, Download, AlertTriangle, ArrowLeft, Layers, ShieldCheck, Check } from 'lucide-react';

export default function BatchDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [batch, setBatch] = useState<SubmissionBatch | null>(null);
  const [issues, setIssues] = useState<ValidationIssue[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!id) return;
    api
      .batch(id)
      .then(setBatch)
      .catch((err: any) => setError(err.message));
    api
      .batchIssues(id)
      .then(setIssues)
      .catch(() => setIssues([]));
  }, [id]);

  async function onCommit() {
    if (!id) return;
    setBusy(true);
    setError(null);
    setMessage(null);
    try {
      const result = await api.commitBatch(id);
      const exceptionText = result.intake_exceptions.length
        ? ` Capacity Exception: ${result.intake_exceptions.map((e) => e.programme_code).join(', ')}.`
        : '';
      setMessage(
        `Successfully committed ${result.committed} records, skipped ${result.skipped}.${exceptionText}`
      );
      const refreshed = await api.batch(id);
      setBatch(refreshed);
      setIssues(await api.batchIssues(id));
    } catch (err: any) {
      setError(err?.message || 'Transactional commit failed.');
    } finally {
      setBusy(false);
    }
  }

  function downloadReport() {
    if (issues.length === 0) return;
    const header = 'row_reference,rule_code,severity,message,field,evidence,rule_version,status';
    const body = issues
      .map((issue) =>
        [
          issue.row_reference,
          issue.rule_code,
          issue.severity,
          `"${issue.message.replace(/"/g, '""')}"`,
          issue.field ?? '',
          issue.evidence ? `"${issue.evidence.replace(/"/g, '""')}"` : '',
          issue.rule_version,
          issue.status
        ].join(',')
      )
      .join('\n');
    const blob = new Blob([`${header}\n${body}\n`], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `batch-${id}-issues.csv`;
    link.click();
    URL.revokeObjectURL(url);
  }

  if (error && !batch) {
    return (
      <div className="panel bg-white border border-red-200">
        <h2 className="text-red-700">Batch Unavailable</h2>
        <p className="text-sm text-red-600 mt-2">{error}</p>
        <Link to="/dashboard" className="btn-secondary mt-4 inline-flex items-center gap-2">
          <ArrowLeft className="w-4 h-4" /> Return to Dashboard
        </Link>
      </div>
    );
  }

  if (!batch) {
    return <div className="p-12 text-center text-slate-500">Loading batch details...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <Link to="/dashboard" className="text-xs text-blue-600 hover:underline flex items-center gap-1 mb-2">
            <ArrowLeft className="w-3.5 h-3.5" /> Back to dashboard
          </Link>
          <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2.5">
            <Layers className="w-6 h-6 text-blue-600" />
            Batch {batch.id.slice(0, 8)}...
          </h1>
          <p className="text-xs text-slate-500 mt-1">
            File: <strong>{batch.file_name}</strong> &middot; Template: <code>{batch.schema_version}</code> &middot; Rule: <code>{batch.rule_version}</code>
          </p>
        </div>

        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={downloadReport}
            disabled={issues.length === 0}
            className="btn-secondary text-xs"
          >
            <Download className="w-4 h-4" />
            Export Issues ({issues.length})
          </button>

          <button
            type="button"
            onClick={onCommit}
            disabled={busy || batch.status === 'COMMITTED'}
            className="btn-primary text-xs"
          >
            <Check className="w-4 h-4" />
            {busy ? 'Committing...' : batch.status === 'COMMITTED' ? 'Committed to Registry' : 'Commit Accepted Rows'}
          </button>
        </div>
      </div>

      {/* Metrics Row */}
      <section className="metric-grid" aria-label="Batch counts">
        <div className="metric">
          <span className="metric-label">Total Rows</span>
          <span className="metric-value">{batch.row_count}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Accepted</span>
          <span className="metric-value text-emerald-600">{batch.accepted_count}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Warnings</span>
          <span className="metric-value text-amber-600">{batch.warning_count}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Needs Review</span>
          <span className="metric-value text-purple-600">{batch.needs_review_count}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Duplicates</span>
          <span className="metric-value text-red-600">{batch.duplicate_count}</span>
        </div>
        <div className="metric">
          <span className="metric-label">Batch Status</span>
          <div className="mt-2">
            <StatusBadge label={batch.status} />
          </div>
        </div>
      </section>

      {message && (
        <div className="p-4 bg-emerald-50 border border-emerald-200 rounded-xl text-sm text-emerald-800 flex items-center gap-2">
          <CheckCircle2 className="w-5 h-5 text-emerald-600 shrink-0" />
          <span>{message}</span>
        </div>
      )}

      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-xl text-sm text-red-800 flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-red-600 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Findings Table */}
      <section className="panel p-0 overflow-hidden bg-white border border-slate-200">
        <div className="p-4 border-b border-slate-100 flex items-center justify-between">
          <h2 className="text-base font-semibold text-slate-800 flex items-center gap-2 m-0">
            <AlertTriangle className="w-4 h-4 text-amber-500" />
            Row-Level Validation Findings ({issues.length})
          </h2>
          <span className="text-xs text-slate-500">Every finding is traceable to a specific rule code</span>
        </div>

        {issues.length === 0 ? (
          <div className="p-8 text-center text-slate-500">
            <ShieldCheck className="w-8 h-8 text-emerald-500 mx-auto mb-2" />
            <p className="font-medium text-slate-700">No validation issues detected.</p>
            <p className="text-xs text-slate-400 mt-1">All rows satisfy the configured validation rules.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse m-0">
              <thead>
                <tr className="bg-slate-50 text-xs font-semibold text-slate-600 uppercase">
                  <th className="py-3 px-4">Row</th>
                  <th className="py-3 px-4">Severity</th>
                  <th className="py-3 px-4">Rule Code</th>
                  <th className="py-3 px-4">Message</th>
                  <th className="py-3 px-4">Evidence</th>
                  <th className="py-3 px-4">Status</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {issues.map((issue) => (
                  <tr key={issue.id} className="hover:bg-slate-50/80 transition-colors">
                    <td className="py-3 px-4 font-mono font-medium text-slate-800 text-xs">{issue.row_reference}</td>
                    <td className="py-3 px-4">
                      <StatusBadge label={issue.severity} />
                    </td>
                    <td className="py-3 px-4 font-mono text-xs text-slate-700">
                      <code>{issue.rule_code}</code>
                    </td>
                    <td className="py-3 px-4 text-slate-800 text-xs">
                      {issue.message}
                      {issue.field && <span className="text-slate-400 ml-1">({issue.field})</span>}
                    </td>
                    <td className="py-3 px-4 font-mono text-xs text-slate-500">{issue.evidence ?? '-'}</td>
                    <td className="py-3 px-4">
                      <StatusBadge label={issue.status} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}
