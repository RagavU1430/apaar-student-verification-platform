import { useEffect, useState } from "react";
import { api, type ValidationIssue } from "../api/client";
import StatusBadge from "../components/StatusBadge";
import { AlertTriangle, CheckCircle2, XCircle, FileText, Check, Ban } from "lucide-react";

export default function IssuesPage() {
  const [issues, setIssues] = useState<ValidationIssue[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notes, setNotes] = useState<Record<string, string>>({});
  const [message, setMessage] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  async function load() {
    try {
      setIssues(await api.openIssues());
    } catch (err: any) {
      setError(err?.message || "Could not load issues.");
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function resolve(issue: ValidationIssue, status: "RESOLVED" | "DISMISSED") {
    const note = (notes[issue.id] ?? "").trim();
    if (!note) {
      setError("A recorded reason/note is required before an issue can be closed.");
      return;
    }
    setError(null);
    setBusyId(issue.id);
    try {
      await api.resolveIssue(issue.id, status, note);
      setMessage(`Issue ${issue.id.slice(0, 8)} successfully marked as ${status}.`);
      await load();
    } catch (err: any) {
      setError(err?.message || "Could not resolve issue.");
    } finally {
      setBusyId(null);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
          <AlertTriangle className="w-7 h-7 text-amber-500" />
          Validation &amp; Exception Review Queue
        </h1>
        <p className="text-sm text-slate-500 mt-1">
          Review, resolve, or dismiss outstanding validation issues and capacity exceptions. Every decision requires a recorded reason and creates an attributable audit record.
        </p>
      </div>

      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-xl text-sm text-red-800 flex items-center gap-2">
          <XCircle className="w-5 h-5 text-red-600 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {message && (
        <div className="p-4 bg-emerald-50 border border-emerald-200 rounded-xl text-sm text-emerald-800 flex items-center gap-2">
          <CheckCircle2 className="w-5 h-5 text-emerald-600 shrink-0" />
          <span>{message}</span>
        </div>
      )}

      <section className="panel bg-white border border-slate-200 p-0 overflow-hidden">
        <div className="p-4 border-b border-slate-100 flex items-center justify-between">
          <h2 className="text-base font-semibold text-slate-800 m-0 flex items-center gap-2">
            <FileText className="w-5 h-5 text-blue-600" />
            Open Issues Requiring Action ({issues.length})
          </h2>
          <span className="text-xs text-slate-500 font-mono">
            Audit trailing enabled
          </span>
        </div>

        {issues.length === 0 ? (
          <div className="p-12 text-center text-slate-500">
            <CheckCircle2 className="w-10 h-10 text-emerald-500 mx-auto mb-2" />
            <p className="font-semibold text-slate-700">No open issues.</p>
            <p className="text-xs text-slate-400 mt-1">All batches are clean or all flagged issues have been resolved.</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse m-0">
              <thead>
                <tr className="bg-slate-50 text-xs font-semibold text-slate-600 uppercase">
                  <th className="py-3 px-4">Location</th>
                  <th className="py-3 px-4">Severity</th>
                  <th className="py-3 px-4">Rule Code</th>
                  <th className="py-3 px-4">Finding &amp; Evidence</th>
                  <th className="py-3 px-4">Reviewer Reason (Mandatory)</th>
                  <th className="py-3 px-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 text-sm">
                {issues.map((issue) => (
                  <tr key={issue.id} className="hover:bg-slate-50/80">
                    <td className="py-3 px-4 font-mono font-medium text-slate-800 text-xs whitespace-nowrap">
                      {issue.row_reference}
                    </td>
                    <td className="py-3 px-4">
                      <StatusBadge label={issue.severity} />
                    </td>
                    <td className="py-3 px-4 font-mono text-xs text-slate-700">
                      <code>{issue.rule_code}</code>
                      <div className="text-[11px] text-slate-400 font-sans mt-0.5">rule: {issue.rule_version}</div>
                    </td>
                    <td className="py-3 px-4 max-w-sm">
                      <div className="text-xs text-slate-800 font-medium">{issue.message}</div>
                      {issue.evidence && (
                        <div className="text-[11px] font-mono text-slate-500 mt-1 bg-slate-50 p-1.5 rounded border border-slate-100">
                          Evidence: {issue.evidence}
                        </div>
                      )}
                    </td>
                    <td className="py-3 px-4 min-w-[220px]">
                      <input
                        id={`note-${issue.id}`}
                        value={notes[issue.id] ?? ""}
                        placeholder="State reason for resolution/dismissal..."
                        onChange={(e) =>
                          setNotes((prev) => ({ ...prev, [issue.id]: e.target.value }))
                        }
                        className="w-full px-3 py-1.5 text-xs border border-slate-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:outline-none"
                      />
                    </td>
                    <td className="py-3 px-4 text-right whitespace-nowrap">
                      <div className="inline-flex gap-2">
                        <button
                          type="button"
                          onClick={() => resolve(issue, "RESOLVED")}
                          disabled={busyId === issue.id}
                          className="px-2.5 py-1 bg-emerald-600 hover:bg-emerald-700 text-white rounded text-xs font-semibold inline-flex items-center gap-1 transition disabled:opacity-50"
                        >
                          <Check className="w-3.5 h-3.5" />
                          Resolve
                        </button>
                        <button
                          type="button"
                          onClick={() => resolve(issue, "DISMISSED")}
                          disabled={busyId === issue.id}
                          className="px-2.5 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded text-xs font-semibold inline-flex items-center gap-1 transition disabled:opacity-50"
                        >
                          <Ban className="w-3.5 h-3.5" />
                          Dismiss
                        </button>
                      </div>
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
